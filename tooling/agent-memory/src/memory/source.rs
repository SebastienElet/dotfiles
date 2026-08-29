mod oracle;

use super::{Fingerprint, MemoryError, ProcessOutput, ProcessRunner, SourceKind, ValidatedDraft};
use rustix::fs::{Mode, OFlags, open};
use sha2::{Digest, Sha256};
use std::ffi::{OsStr, OsString};
use std::fs::{self, File};
use std::io::{self, Read};
use std::net::IpAddr;
use std::path::{Component, Path, PathBuf};
use tempfile::{Builder, NamedTempFile};
use url::{Host, Url};

const MAX_SOURCE_BYTES: u64 = 1024 * 1024;
const CURL_WRITE_OUT: &str = "%{http_code}\n%{url_effective}\n%{remote_ip}";

pub struct SourceContext<'a> {
    cwd: &'a Path,
    git: &'a dyn ProcessRunner,
    curl: &'a dyn ProcessRunner,
    temporary_directory: Option<&'a Path>,
}

impl<'a> SourceContext<'a> {
    pub fn new(cwd: &'a Path, git: &'a dyn ProcessRunner, curl: &'a dyn ProcessRunner) -> Self {
        Self {
            cwd,
            git,
            curl,
            temporary_directory: None,
        }
    }

    pub fn with_temporary_directory(mut self, directory: &'a Path) -> Self {
        self.temporary_directory = Some(directory);
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedSource {
    kind: SourceKind,
    locator: String,
    fingerprint: Fingerprint,
}

impl ResolvedSource {
    pub fn kind(&self) -> SourceKind {
        self.kind
    }

    pub fn locator(&self) -> &str {
        &self.locator
    }

    pub fn fingerprint(&self) -> &Fingerprint {
        &self.fingerprint
    }
}

#[derive(Debug)]
pub struct ResolvedDraft {
    draft: ValidatedDraft,
    sources: Vec<ResolvedSource>,
}

impl ResolvedDraft {
    pub fn draft(&self) -> &ValidatedDraft {
        &self.draft
    }

    pub fn sources(&self) -> &[ResolvedSource] {
        &self.sources
    }

    pub fn recheck_sources(&self, context: &SourceContext<'_>) -> Result<(), MemoryError> {
        for expected in &self.sources {
            let actual = resolve_source(expected.kind, &expected.locator, context)?;
            if actual.fingerprint != expected.fingerprint {
                return Err(MemoryError::new("source_changed", "proof.sources"));
            }
        }
        Ok(())
    }

    pub fn into_parts(self) -> (ValidatedDraft, Vec<ResolvedSource>) {
        (self.draft, self.sources)
    }
}

pub fn resolve_sources(
    draft: ValidatedDraft,
    context: &SourceContext<'_>,
) -> Result<ResolvedDraft, MemoryError> {
    let proof_sources = draft.proof().sources();
    let has_official_url = proof_sources
        .iter()
        .any(|source| source.kind() == SourceKind::OfficialUrl);
    let has_user_decision = proof_sources
        .iter()
        .any(|source| source.kind() == SourceKind::UserDecision);
    if has_official_url && !has_user_decision {
        return Err(source_invalid());
    }
    let sources = draft
        .proof()
        .sources()
        .iter()
        .map(|source| resolve_source(source.kind(), source.locator(), context))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ResolvedDraft { draft, sources })
}

fn resolve_source(
    kind: SourceKind,
    locator: &str,
    context: &SourceContext<'_>,
) -> Result<ResolvedSource, MemoryError> {
    let bytes = match kind {
        SourceKind::GitFile => resolve_git_file(locator, context)?,
        SourceKind::LocalFile => resolve_local_file(locator)?,
        SourceKind::OfficialUrl => resolve_official_url(locator, context)?,
        SourceKind::UserDecision => locator.as_bytes().to_vec(),
    };
    Ok(ResolvedSource {
        kind,
        locator: locator.to_owned(),
        fingerprint: Fingerprint::from_validated(format!("sha256:{:x}", Sha256::digest(bytes))),
    })
}

fn resolve_git_file(locator: &str, context: &SourceContext<'_>) -> Result<Vec<u8>, MemoryError> {
    let relative = Path::new(locator);
    if relative.as_os_str().is_empty()
        || relative.is_absolute()
        || !relative
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
    {
        return Err(source_invalid());
    }
    let repository = context
        .cwd
        .canonicalize()
        .map_err(|_| source_unavailable())?;
    let path = resolved_regular_path(&repository.join(relative))?;
    if !path.starts_with(&repository) {
        return Err(source_invalid());
    }
    validate_tracked(relative, context)?;
    read_bounded_regular_file(&path)
}

fn validate_tracked(relative: &Path, context: &SourceContext<'_>) -> Result<(), MemoryError> {
    let arguments = [
        OsString::from("-C"),
        context.cwd.as_os_str().to_owned(),
        OsString::from("ls-files"),
        OsString::from("--error-unmatch"),
        OsString::from("--"),
        relative.as_os_str().to_owned(),
    ];
    let output = context
        .git
        .run(OsStr::new("git"), &arguments, None)
        .map_err(|_| source_unavailable())?;
    if !output.success() {
        return if output.code() == Some(1) {
            Err(source_invalid())
        } else {
            Err(source_unavailable())
        };
    }
    let stdout = std::str::from_utf8(output.stdout()).map_err(|_| source_unavailable())?;
    if stdout.trim().is_empty() {
        return Err(source_unavailable());
    }
    Ok(())
}

fn resolve_local_file(locator: &str) -> Result<Vec<u8>, MemoryError> {
    let path = Path::new(locator);
    if !path.is_absolute() {
        return Err(source_invalid());
    }
    let path = resolved_regular_path(path)?;
    read_bounded_regular_file(&path)
}

fn resolved_regular_path(path: &Path) -> Result<PathBuf, MemoryError> {
    let parent = path.parent().ok_or_else(source_invalid)?;
    let name = path.file_name().ok_or_else(source_invalid)?;
    let parent = parent.canonicalize().map_err(classify_local_error)?;
    let resolved = parent.join(name);
    let metadata = fs::symlink_metadata(&resolved).map_err(classify_local_error)?;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
        return Err(source_invalid());
    }
    Ok(resolved)
}

fn read_bounded_regular_file(path: &Path) -> Result<Vec<u8>, MemoryError> {
    let descriptor = open(
        path,
        OFlags::RDONLY | OFlags::CLOEXEC | OFlags::NOFOLLOW,
        Mode::empty(),
    )
    .map_err(|error| classify_local_io_kind(error.kind()))?;
    let mut file = File::from(descriptor);
    let metadata = file.metadata().map_err(|_| source_unavailable())?;
    if !metadata.file_type().is_file() {
        return Err(source_invalid());
    }
    if metadata.len() > MAX_SOURCE_BYTES {
        return Err(source_unavailable());
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.by_ref()
        .take(MAX_SOURCE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| source_unavailable())?;
    if bytes.len() as u64 > MAX_SOURCE_BYTES {
        return Err(source_unavailable());
    }
    Ok(bytes)
}

fn resolve_official_url(
    locator: &str,
    context: &SourceContext<'_>,
) -> Result<Vec<u8>, MemoryError> {
    let url = validated_https_url(locator).map_err(|_| source_invalid())?;
    let temporary = create_private_temporary(context)?;
    let arguments = curl_arguments(temporary.path(), url.as_str());
    let output = context
        .curl
        .run(OsStr::new("curl"), &arguments, None)
        .map_err(|_| source_unavailable())?;
    let metadata = curl_metadata(&output)?;
    validate_curl_result(&output, &metadata)?;
    read_bounded_regular_file(temporary.path())
}

fn create_private_temporary(context: &SourceContext<'_>) -> Result<NamedTempFile, MemoryError> {
    let mut builder = Builder::new();
    builder.prefix(".agent-memory-source-");
    let temporary = match context.temporary_directory {
        Some(directory) => builder.tempfile_in(directory),
        None => builder.tempfile(),
    }
    .map_err(|_| source_unavailable())?;
    temporary
        .as_file()
        .set_permissions(std::os::unix::fs::PermissionsExt::from_mode(0o600))
        .map_err(|_| source_unavailable())?;
    Ok(temporary)
}

fn curl_arguments(output: &Path, locator: &str) -> Vec<OsString> {
    [
        OsString::from("--disable"),
        OsString::from("--silent"),
        OsString::from("--show-error"),
        OsString::from("--fail-with-body"),
        OsString::from("--location"),
        OsString::from("--max-redirs"),
        OsString::from("5"),
        OsString::from("--proto"),
        OsString::from("=https"),
        OsString::from("--proto-redir"),
        OsString::from("=https"),
        OsString::from("--connect-timeout"),
        OsString::from("5"),
        OsString::from("--max-time"),
        OsString::from("15"),
        OsString::from("--max-filesize"),
        OsString::from("1048576"),
        OsString::from("--output"),
        output.as_os_str().to_owned(),
        OsString::from("--write-out"),
        OsString::from(CURL_WRITE_OUT),
        OsString::from(locator),
    ]
    .into()
}

struct CurlMetadata {
    status: u16,
    final_url: Url,
    _remote_ip: IpAddr,
}

fn curl_metadata(output: &ProcessOutput) -> Result<CurlMetadata, MemoryError> {
    let stdout = std::str::from_utf8(output.stdout()).map_err(|_| source_unavailable())?;
    let lines = stdout.lines().collect::<Vec<_>>();
    if lines.len() != 3 || lines.iter().any(|line| line.is_empty()) {
        return Err(source_unavailable());
    }
    let status = lines[0].parse::<u16>().map_err(|_| source_unavailable())?;
    if !(100..=599).contains(&status) {
        return Err(source_unavailable());
    }
    let final_url = Url::parse(lines[1]).map_err(|_| source_unavailable())?;
    let remote_ip = lines[2]
        .parse::<IpAddr>()
        .map_err(|_| source_unavailable())?;
    Ok(CurlMetadata {
        status,
        final_url,
        _remote_ip: remote_ip,
    })
}

fn validate_curl_result(
    output: &ProcessOutput,
    metadata: &CurlMetadata,
) -> Result<(), MemoryError> {
    if !allowed_https_url(&metadata.final_url) {
        return Err(source_invalid());
    }
    if matches!(metadata.status, 404 | 410) && output.code() == Some(22) {
        return Err(source_invalid());
    }
    if !output.success() || !matches!(metadata.status, 200..=299) {
        return Err(source_unavailable());
    }
    Ok(())
}

fn validated_https_url(value: &str) -> Result<Url, ()> {
    let url = Url::parse(value).map_err(|_| ())?;
    allowed_https_url(&url).then_some(url).ok_or(())
}

fn allowed_https_url(url: &Url) -> bool {
    url.scheme() == "https"
        && url.username().is_empty()
        && url.password().is_none()
        && url.fragment().is_none()
        && matches!(url.host(), Some(Host::Domain(_)))
}

fn classify_local_error(error: io::Error) -> MemoryError {
    classify_local_io_kind(error.kind())
}

fn classify_local_io_kind(kind: io::ErrorKind) -> MemoryError {
    if kind == io::ErrorKind::NotFound {
        source_invalid()
    } else {
        source_unavailable()
    }
}

const fn source_invalid() -> MemoryError {
    MemoryError::new("source_invalid", "proof.sources")
}

const fn source_unavailable() -> MemoryError {
    MemoryError::new("source_unavailable", "proof.sources")
}
