use crate::cli::Resource;
use arnes::diagnostic::{Diagnostic, HumanContext, HumanOptions, Report};
use arnes::manifest::{Agent, Scope};

const DEFAULT_RESOURCES: [Resource; 4] = [
    Resource::Manifest,
    Resource::Skills,
    Resource::Hooks,
    Resource::Mcp,
];

pub(super) fn human(
    diagnostics: Vec<Diagnostic>,
    resource: Option<Resource>,
    agent: Option<Agent>,
    scope: Option<Scope>,
    options: HumanOptions,
) -> (String, u8) {
    if resource.is_some() {
        let report = Report::new(diagnostics);
        return (
            report.human(&context(resource, agent, scope), options),
            report.exit_code(),
        );
    }

    let report = Report::new(diagnostics);
    let exit_code = report.exit_code();
    let mut output = String::new();
    for (section, diagnostics) in default_sections(report.into_diagnostics()) {
        if section != Resource::Manifest && diagnostics.is_empty() {
            continue;
        }
        if !output.is_empty() {
            output.push_str("\n\n");
        }
        let section_scope = if section == Resource::Mcp {
            scope
        } else {
            scope.or(Some(Scope::User))
        };
        output.push_str(
            &Report::new(diagnostics).human(&context(Some(section), agent, section_scope), options),
        );
    }
    (output, exit_code)
}

fn default_sections(diagnostics: Vec<Diagnostic>) -> Vec<(Resource, Vec<Diagnostic>)> {
    let mut sections = DEFAULT_RESOURCES.map(|resource| (resource, Vec::new()));
    let mut unclaimed = Vec::new();
    for diagnostic in diagnostics {
        match sections
            .iter_mut()
            .find(|(resource, _)| resource.key() == diagnostic.resource)
        {
            Some((_, section)) => section.push(diagnostic),
            None => unclaimed.push(diagnostic),
        }
    }
    debug_assert!(
        unclaimed.is_empty(),
        "the default doctor emits only its declared resources"
    );
    sections.into_iter().collect()
}

fn context(resource: Option<Resource>, agent: Option<Agent>, scope: Option<Scope>) -> HumanContext {
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

impl Resource {
    fn key(self) -> &'static str {
        match self {
            Self::Manifest => "manifest",
            Self::Config => "config",
            Self::Instructions => "instructions",
            Self::Skills => "skills",
            Self::Prompts => "prompts",
            Self::Commands => "commands",
            Self::Rules => "rules",
            Self::Hooks => "hooks",
            Self::Mcp => "mcp",
            Self::Statusline => "statusline",
        }
    }

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
