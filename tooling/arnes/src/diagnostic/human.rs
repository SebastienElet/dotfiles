use super::{Diagnostic, State, human_field};

#[derive(Clone, Debug, Eq, PartialEq)]
struct SectionCount {
    singular: &'static str,
    plural: &'static str,
    empty: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HumanContext {
    parts: Vec<String>,
    section_count: Option<SectionCount>,
}

impl HumanContext {
    pub fn new(heading: impl Into<String>) -> Self {
        Self {
            parts: vec![heading.into()],
            section_count: None,
        }
    }

    pub fn with_qualifier(mut self, qualifier: impl Into<String>) -> Self {
        self.parts.push(qualifier.into());
        self
    }

    pub fn with_section_count(
        mut self,
        singular: &'static str,
        plural: &'static str,
        empty: &'static str,
    ) -> Self {
        self.section_count = Some(SectionCount {
            singular,
            plural,
            empty,
        });
        self
    }

    pub(super) fn heading(&self, count: Option<usize>) -> String {
        let mut parts = self.parts.clone();
        if let Some(labels) = &self.section_count {
            parts.push(match count {
                Some(1) => format!("1 {}", labels.singular),
                Some(count) => format!("{count} {}", labels.plural),
                None => labels.empty.to_owned(),
            });
        }
        parts.join(" · ")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HumanOptions {
    verbose: bool,
}

impl HumanOptions {
    pub fn normal() -> Self {
        Self { verbose: false }
    }

    pub fn verbose() -> Self {
        Self { verbose: true }
    }

    pub fn includes_healthy(self) -> bool {
        self.verbose
    }
}

pub(super) fn render(
    diagnostics: &[Diagnostic],
    context: &HumanContext,
    options: HumanOptions,
) -> String {
    if diagnostics.is_empty() {
        return "No diagnostics".to_owned();
    }
    let mut lines = vec![
        context.heading(None),
        format!("✓ {} healthy", state_count(diagnostics, State::Healthy)),
    ];
    let unsupported = state_count(diagnostics, State::Unsupported);
    if unsupported > 0 {
        lines.push(format!("! {unsupported} unsupported (non-blocking)"));
    }
    lines.push(String::new());
    lines.extend(render_flat(diagnostics, options));
    trim_trailing_empty(&mut lines);
    lines.join("\n")
}

fn state_count(diagnostics: &[Diagnostic], state: State) -> usize {
    diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.state == state)
        .count()
}

fn render_flat(diagnostics: &[Diagnostic], options: HumanOptions) -> Vec<String> {
    let mut lines = Vec::new();
    let mut group = None;
    for diagnostic in diagnostics
        .iter()
        .filter(|diagnostic| options.includes_healthy() || diagnostic.state != State::Healthy)
    {
        if let Some(human) = diagnostic.human() {
            if group != Some(human.group()) {
                if !lines.is_empty() {
                    lines.push(String::new());
                }
                lines.push(human_field(human.group()));
                group = Some(human.group());
            }
            lines.push(format!(
                "  {:11} {}",
                diagnostic.state.to_string(),
                human_field(human.summary())
            ));
        } else {
            group = None;
            lines.push(diagnostic.to_string());
        }
    }
    lines
}

fn trim_trailing_empty(lines: &mut Vec<String>) {
    while lines.last().is_some_and(String::is_empty) {
        lines.pop();
    }
}
