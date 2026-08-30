use crate::cli::{Color, Format, Resource, validate_render_options};
use crate::cli_output::write_output;
use arnes::Roots;
use arnes::commands;
use arnes::config;
use arnes::diagnostic::{ColorMode, Diagnostic, HumanContext, HumanOptions, Report, State};
use arnes::instructions;
use arnes::manifest::{self, Agent, Scope};
use arnes::prompts;
use arnes::rules;
use arnes::skills;
use std::io::{self, IsTerminal};
use std::process::ExitCode;

pub(super) fn run(
    resource: Option<Resource>,
    agent: Option<Agent>,
    scope: Option<Scope>,
    format: Format,
    color: Color,
    verbose: bool,
) -> ExitCode {
    if let Err(error) = validate_render_options(format, verbose, color) {
        eprintln!("{error}");
        return ExitCode::from(2);
    }

    let diagnostics = diagnose(resource, agent, scope);
    let human_options = (if verbose {
        HumanOptions::verbose()
    } else {
        HumanOptions::normal()
    })
    .with_color(
        ColorMode::from(color),
        io::stdout().is_terminal(),
        std::env::var_os("NO_COLOR").as_deref(),
    );
    let (output, exit_code) = match format {
        Format::Human => render_human(diagnostics, resource, agent, scope, human_options),
        Format::Json => {
            let report = Report::new(diagnostics);
            (
                report.json().expect("diagnostics are JSON serializable"),
                report.exit_code(),
            )
        }
    };

    if let Err(error) = write_output(&output) {
        eprintln!("output: could not write diagnostics: {error}");
        return ExitCode::from(2);
    }
    ExitCode::from(exit_code)
}

fn render_human(
    diagnostics: Vec<Diagnostic>,
    resource: Option<Resource>,
    agent: Option<Agent>,
    scope: Option<Scope>,
    options: HumanOptions,
) -> (String, u8) {
    if resource.is_some() {
        let report = Report::new(diagnostics);
        return (
            report.human(&human_context(resource, agent, scope), options),
            report.exit_code(),
        );
    }

    let (manifest, skills): (Vec<_>, Vec<_>) = diagnostics
        .into_iter()
        .partition(|diagnostic| diagnostic.resource == "manifest");
    let manifest_report = Report::new(manifest);
    let mut output = manifest_report.human(
        &human_context(Some(Resource::Manifest), agent, scope),
        options,
    );
    let mut exit_code = manifest_report.exit_code();
    if !skills.is_empty() {
        let skills_report = Report::new(skills);
        output.push_str("\n\n");
        output.push_str(&skills_report.human(
            &human_context(Some(Resource::Skills), agent, scope),
            options,
        ));
        exit_code = exit_code.max(skills_report.exit_code());
    }
    (output, exit_code)
}

impl Resource {
    fn heading(self) -> &'static str {
        match self {
            Self::Manifest => "Manifest",
            Self::Config => "Config",
            Self::Instructions => "Instructions",
            Self::Skills => "Skills",
            Self::Prompts => "Prompts",
            Self::Commands => "Commands",
            Self::Rules => "Rules",
            Self::Hooks => "Hooks",
            Self::Mcp => "MCP",
            Self::Statusline => "Statusline",
        }
    }
}

fn human_context(
    resource: Option<Resource>,
    agent: Option<Agent>,
    scope: Option<Scope>,
) -> HumanContext {
    let resource = resource.unwrap_or(Resource::Manifest);
    let mut context = HumanContext::new(resource.heading());
    if resource != Resource::Manifest {
        if let Some(scope) = scope {
            context = context.with_qualifier(format!("{scope} scope"));
        }
        if let Some(agent) = agent {
            context = context.with_qualifier(format!("{agent} agent"));
        } else if resource == Resource::Skills {
            context = context.with_section_count("agent", "agents", "all agents");
        }
    }
    context
}

fn diagnose(
    resource: Option<Resource>,
    agent: Option<Agent>,
    scope: Option<Scope>,
) -> Vec<Diagnostic> {
    match resource {
        None => diagnose_default(agent, scope),
        Some(Resource::Manifest) => match Roots::from_environment() {
            Ok(roots) => vec![diagnose_manifest(&roots)],
            Err(error) => vec![Diagnostic::new("manifest", State::Error, error.to_string())],
        },
        Some(Resource::Config) => match Roots::from_environment() {
            Ok(roots) => diagnose_config(&roots, agent, scope),
            Err(error) => vec![Diagnostic::new("config", State::Error, error.to_string())],
        },
        Some(Resource::Instructions) => match Roots::from_environment() {
            Ok(roots) => diagnose_instructions(&roots, agent, scope),
            Err(error) => vec![Diagnostic::new(
                "instructions",
                State::Error,
                error.to_string(),
            )],
        },
        Some(Resource::Skills) => match Roots::from_environment() {
            Ok(roots) => diagnose_skills(&roots, agent, scope),
            Err(error) => vec![Diagnostic::new("skills", State::Error, error.to_string())],
        },
        Some(Resource::Prompts) => match Roots::from_environment() {
            Ok(roots) => diagnose_prompts(&roots, agent, scope),
            Err(error) => vec![Diagnostic::new("prompts", State::Error, error.to_string())],
        },
        Some(Resource::Commands) => match Roots::from_environment() {
            Ok(roots) => diagnose_commands(&roots, agent, scope),
            Err(error) => vec![Diagnostic::new("commands", State::Error, error.to_string())],
        },
        Some(Resource::Rules) => match Roots::from_environment() {
            Ok(roots) => diagnose_rules(&roots, agent, scope),
            Err(error) => vec![Diagnostic::new("rules", State::Error, error.to_string())],
        },
        _ => Vec::new(),
    }
}

fn diagnose_default(agent: Option<Agent>, scope: Option<Scope>) -> Vec<Diagnostic> {
    let roots = match Roots::from_environment() {
        Ok(roots) => roots,
        Err(error) => return vec![Diagnostic::new("manifest", State::Error, error.to_string())],
    };
    let manifest = match manifest::load(roots.home()) {
        Ok(manifest) => manifest,
        Err(error) => return vec![Diagnostic::new("manifest", State::Error, error.to_string())],
    };
    let mut diagnostics = vec![Diagnostic::new(
        "manifest",
        State::Healthy,
        "manifest is valid",
    )];
    diagnostics.extend(skills::diagnose(&roots, &manifest, agent, scope));
    diagnostics
}

fn diagnose_manifest(roots: &Roots) -> Diagnostic {
    match manifest::load(roots.home()) {
        Ok(_) => Diagnostic::new("manifest", State::Healthy, "manifest is valid"),
        Err(error) => Diagnostic::new("manifest", State::Error, error.to_string()),
    }
}

fn diagnose_config(roots: &Roots, agent: Option<Agent>, scope: Option<Scope>) -> Vec<Diagnostic> {
    match manifest::load(roots.home()) {
        Ok(manifest) => config::diagnose(roots, &manifest, agent, scope),
        Err(error) => vec![Diagnostic::new("config", State::Error, error.to_string())],
    }
}

fn diagnose_instructions(
    roots: &Roots,
    agent: Option<Agent>,
    scope: Option<Scope>,
) -> Vec<Diagnostic> {
    match manifest::load(roots.home()) {
        Ok(manifest) => instructions::diagnose(roots, &manifest, agent, scope),
        Err(error) => vec![Diagnostic::new(
            "instructions",
            State::Error,
            error.to_string(),
        )],
    }
}

fn diagnose_skills(roots: &Roots, agent: Option<Agent>, scope: Option<Scope>) -> Vec<Diagnostic> {
    match manifest::load(roots.home()) {
        Ok(manifest) => skills::diagnose(roots, &manifest, agent, scope),
        Err(error) => vec![Diagnostic::new("skills", State::Error, error.to_string())],
    }
}

fn diagnose_prompts(roots: &Roots, agent: Option<Agent>, scope: Option<Scope>) -> Vec<Diagnostic> {
    match manifest::load(roots.home()) {
        Ok(manifest) => prompts::diagnose(roots, &manifest, agent, scope),
        Err(error) => vec![Diagnostic::new("prompts", State::Error, error.to_string())],
    }
}

fn diagnose_commands(roots: &Roots, agent: Option<Agent>, scope: Option<Scope>) -> Vec<Diagnostic> {
    match manifest::load(roots.home()) {
        Ok(manifest) => commands::diagnose(roots, &manifest, agent, scope),
        Err(error) => vec![Diagnostic::new("commands", State::Error, error.to_string())],
    }
}

fn diagnose_rules(roots: &Roots, agent: Option<Agent>, scope: Option<Scope>) -> Vec<Diagnostic> {
    match manifest::load(roots.home()) {
        Ok(manifest) => rules::diagnose(roots, &manifest, agent, scope),
        Err(error) => vec![Diagnostic::new("rules", State::Error, error.to_string())],
    }
}
