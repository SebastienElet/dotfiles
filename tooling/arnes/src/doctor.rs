use crate::cli::{Color, Format, Resource, validate_render_options};
use crate::cli_output::write_output;
use arnes::Roots;
use arnes::commands;
use arnes::config;
use arnes::diagnostic::{ColorMode, Diagnostic, HumanOptions, Report, State};
use arnes::hooks;
use arnes::instructions;
use arnes::manifest::{self, Agent, Scope};
use arnes::mcp;
use arnes::prompts;
use arnes::rules;
use arnes::skills;
use arnes::statusline;
use std::io::{self, IsTerminal};
use std::process::ExitCode;

mod render;

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
        Format::Human => render::human(
            diagnostics,
            resource,
            agent,
            resource.and(scope.or(Some(Scope::User))).or(scope),
            human_options,
        ),
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
            Ok(roots) => diagnose_config(&roots, agent, scope.or(Some(Scope::User))),
            Err(error) => vec![Diagnostic::new("config", State::Error, error.to_string())],
        },
        Some(Resource::Instructions) => match Roots::from_environment() {
            Ok(roots) => diagnose_instructions(&roots, agent, scope.or(Some(Scope::User))),
            Err(error) => vec![Diagnostic::new(
                "instructions",
                State::Error,
                error.to_string(),
            )],
        },
        Some(Resource::Skills) => match Roots::from_environment() {
            Ok(roots) => diagnose_skills(&roots, agent, scope.or(Some(Scope::User))),
            Err(error) => vec![Diagnostic::new("skills", State::Error, error.to_string())],
        },
        Some(Resource::Prompts) => match Roots::from_environment() {
            Ok(roots) => diagnose_prompts(&roots, agent, scope.or(Some(Scope::User))),
            Err(error) => vec![Diagnostic::new("prompts", State::Error, error.to_string())],
        },
        Some(Resource::Commands) => match Roots::from_environment() {
            Ok(roots) => diagnose_commands(&roots, agent, scope.or(Some(Scope::User))),
            Err(error) => vec![Diagnostic::new("commands", State::Error, error.to_string())],
        },
        Some(Resource::Rules) => match Roots::from_environment() {
            Ok(roots) => diagnose_rules(&roots, agent, scope.or(Some(Scope::User))),
            Err(error) => vec![Diagnostic::new("rules", State::Error, error.to_string())],
        },
        Some(Resource::Hooks) => match Roots::from_environment() {
            Ok(roots) => diagnose_hooks(&roots, agent, scope.or(Some(Scope::User))),
            Err(error) => vec![Diagnostic::new("hooks", State::Error, error.to_string())],
        },
        Some(Resource::Mcp) => match Roots::from_environment() {
            Ok(roots) => diagnose_mcp(&roots, agent, scope.or(Some(Scope::User))),
            Err(error) => vec![Diagnostic::new("mcp", State::Error, error.to_string())],
        },
        Some(Resource::Statusline) => match Roots::from_environment() {
            Ok(roots) => diagnose_statusline(&roots, agent, scope.or(Some(Scope::User))),
            Err(error) => vec![Diagnostic::new(
                "statusline",
                State::Error,
                error.to_string(),
            )],
        },
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
    let user_scope = scope.or(Some(Scope::User));
    diagnostics.extend(skills::diagnose(&roots, &manifest, agent, user_scope));
    diagnostics.extend(hooks::diagnose(&roots, &manifest, agent, user_scope));
    diagnostics.extend(mcp::diagnose(&roots, &manifest, agent, scope));
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

fn diagnose_hooks(roots: &Roots, agent: Option<Agent>, scope: Option<Scope>) -> Vec<Diagnostic> {
    match manifest::load(roots.home()) {
        Ok(manifest) => hooks::diagnose(roots, &manifest, agent, scope),
        Err(error) => vec![Diagnostic::new("hooks", State::Error, error.to_string())],
    }
}

fn diagnose_mcp(roots: &Roots, agent: Option<Agent>, scope: Option<Scope>) -> Vec<Diagnostic> {
    match manifest::load(roots.home()) {
        Ok(manifest) => mcp::diagnose(roots, &manifest, agent, scope),
        Err(error) => vec![Diagnostic::new("mcp", State::Error, error.to_string())],
    }
}

fn diagnose_statusline(
    roots: &Roots,
    agent: Option<Agent>,
    scope: Option<Scope>,
) -> Vec<Diagnostic> {
    match manifest::load(roots.home()) {
        Ok(manifest) => statusline::diagnose(roots, &manifest, agent, scope),
        Err(error) => vec![Diagnostic::new(
            "statusline",
            State::Error,
            error.to_string(),
        )],
    }
}
