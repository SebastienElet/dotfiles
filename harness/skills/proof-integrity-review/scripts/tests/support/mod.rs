use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    error::Error,
    fs,
    path::Path,
    process::{Command, Output},
};
use tempfile::TempDir;

pub type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;

pub fn digest(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}
pub fn bound(value: &Value) -> Result<String> {
    Ok(digest(&serde_json::to_vec(value)?))
}
pub fn cli(args: &[&str]) -> Result<Output> {
    Ok(Command::new(env!("CARGO_BIN_EXE_proof-integrity"))
        .args(args)
        .output()?)
}
pub fn git(root: &Path, args: &[&str]) -> Result {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}
pub struct Fixture {
    pub directory: TempDir,
}
impl Fixture {
    pub fn new() -> Result<Self> {
        let fixture = Self {
            directory: tempfile::tempdir()?,
        };
        let root = fixture.root();
        fs::create_dir(root.join("policy"))?;
        fs::create_dir(root.join("policy/scripts"))?;
        fs::write(root.join("policy/SKILL.md"), "policy")?;
        fs::write(root.join("policy/scripts/Cargo.toml"), "manifest")?;
        let required: Vec<String> =
            serde_json::from_str(include_str!("../../policy-sources.json"))?;
        for path in required {
            let source = root.join("policy").join(path);
            if source.exists() {
                continue;
            }
            fs::create_dir_all(source.parent().ok_or("parent")?)?;
            fs::write(source, "fixture source")?;
        }
        git(root, &["init", "-b", "candidate"])?;
        git(root, &["config", "user.email", "test@example.invalid"])?;
        git(root, &["config", "user.name", "Test"])?;
        fs::write(root.join(".gitignore"), "ignored\n.proof-integrity/\n")?;
        fs::write(root.join("Makefile"), "old")?;
        git(root, &["add", "."])?;
        git(root, &["commit", "-m", "base"])?;
        git(root, &["branch", "base"])?;
        fs::write(root.join("Makefile"), "new")?;
        git(root, &["add", "Makefile"])?;
        git(root, &["commit", "-m", "candidate"])?;
        fs::create_dir(root.join(".proof-integrity"))?;
        Ok(fixture)
    }
    pub fn root(&self) -> &Path {
        self.directory.path()
    }
    pub fn epoch(&self) -> Result<Value> {
        let output = cli(&[
            "epoch",
            "--repository",
            self.root().to_str().ok_or("path")?,
            "--base",
            "base",
            "--head",
            "HEAD",
            "--policy-root",
            self.root().join("policy").to_str().ok_or("path")?,
        ])?;
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        Ok(serde_json::from_slice(&output.stdout)?)
    }
    pub fn gate(&self, epoch: &Value, receipt: &Value) -> Result<Output> {
        let epoch_path = self.root().join(".proof-integrity/epoch.json");
        let receipt_path = self.root().join(".proof-integrity/receipt.json");
        fs::write(&epoch_path, serde_json::to_vec(epoch)?)?;
        fs::write(&receipt_path, serde_json::to_vec(receipt)?)?;
        cli(&[
            "gate",
            "--epoch",
            epoch_path.to_str().ok_or("path")?,
            "--receipt",
            receipt_path.to_str().ok_or("path")?,
            "--policy-root",
            self.root().join("policy").to_str().ok_or("path")?,
        ])
    }
}
pub fn receipt(epoch: &Value) -> Result<Value> {
    let ids = [
        "proof.trigger_coverage",
        "proof.epoch_binding",
        "proof.fail_closed",
        "make.recipe",
    ];
    let mut claims = Vec::new();
    let mut witnesses = Vec::new();
    for id in ids {
        claims.push(json!({"id":id,"impact":"high","kind":"other","rationale":"invalid input reaches recipe","positive_evidence":{"origin":"reviewer","artifact":"fixture trace","input_basis":"current fixture inputs","environment":"test host"},"claim":"recipe refuses invalid input","source":"contract","enforcement":"recipe","oracle":"integration","paths":["Makefile"]}));
        let targets = json!([{"scope":"repository","path":"Makefile","before_content":"new","after_content":"bad","before_digest":digest(b"new"),"after_digest":digest(b"bad")}]);
        let mutant = json!({"description":"remove refusal", "targets":targets});
        let mutant_digest = bound(&mutant)?;
        let subject = epoch
            .get("subject_digest")
            .and_then(Value::as_str)
            .ok_or("subject")?;
        let evidence = json!({"provenance":{"origin":"reviewer","artifact":"fixture trace","input_basis":"unchanged fixture inputs","environment":"test host"},"command":"fixture-declaration-only", "expected_failure":"invalid input rejected", "red_exit_code":1,"red_stdout":format!("invalid input rejected\nPROOF_RED:{id}:{mutant_digest}\n"),"red_stderr":"","green_exit_code":0,"green_stdout":format!("PROOF_GREEN:{id}:{subject}\n"),"green_stderr":"","targets_digest":bound(&targets)?,"injection_mode":"file-overlay"});
        witnesses.push(json!({"claim_id":id,"mutant_digest":mutant_digest,"mutant":mutant,"red_exit_code":1,"green_exit_code":0,"artifact_digest":bound(&evidence)?,"evidence":evidence}));
    }
    for path in epoch
        .pointer("/subject/changed_paths")
        .and_then(Value::as_array)
        .ok_or("paths")?
    {
        if path == "Makefile" {
            continue;
        }
        let mut claim = claims.last().ok_or("claim")?.clone();
        set(&mut claim, "/id", json!(format!("assessment:{path}")))?;
        set(&mut claim, "/paths", json!([path]))?;
        set(&mut claim, "/impact", json!("low"))?;
        claims.push(claim);
    }
    Ok(
        json!({"schema_version":5,"predicate_type":"https://proof-integrity.local/receipt/v5","subject_digest":epoch.get("subject_digest").ok_or("subject")?,"policy_digest":epoch.get("policy_digest").ok_or("policy")?,"verdict":"PROOF_ADEQUATE","auditor":{"host":"test","session_id":"independent","fresh_session":true,"forked":false,"persistent_memory":false,"author_independent":true,"independent_first_pass":true,"isolation":null,"sandbox_mode":"read-only","write_tools_enabled":false},"claims":claims,"witnesses":witnesses,"limitations":[]}),
    )
}
pub fn set(value: &mut Value, pointer: &str, replacement: Value) -> Result {
    *value
        .pointer_mut(pointer)
        .ok_or_else(|| format!("missing {pointer}"))? = replacement;
    Ok(())
}

pub fn rebind(receipt: &mut Value) -> Result {
    let subject = receipt
        .get("subject_digest")
        .and_then(Value::as_str)
        .ok_or("subject")?
        .to_string();
    let witnesses = receipt
        .get_mut("witnesses")
        .and_then(Value::as_array_mut)
        .ok_or("witnesses")?;
    for witness in witnesses {
        let mutant = witness.get("mutant").ok_or("mutant")?;
        let mutant_digest = bound(mutant)?;
        let targets_digest = bound(mutant.get("targets").ok_or("targets")?)?;
        set(witness, "/mutant_digest", json!(mutant_digest))?;
        set(witness, "/evidence/targets_digest", json!(targets_digest))?;
        let id = witness
            .get("claim_id")
            .and_then(Value::as_str)
            .ok_or("id")?;
        set(
            witness,
            "/evidence/red_stdout",
            json!(format!(
                "invalid input rejected\nPROOF_RED:{id}:{mutant_digest}\n"
            )),
        )?;
        let id = witness
            .get("claim_id")
            .and_then(Value::as_str)
            .ok_or("id")?;
        set(
            witness,
            "/evidence/green_stdout",
            json!(format!("PROOF_GREEN:{id}:{subject}\n")),
        )?;
        let evidence_digest = bound(witness.get("evidence").ok_or("evidence")?)?;
        set(witness, "/artifact_digest", json!(evidence_digest))?;
    }
    Ok(())
}
