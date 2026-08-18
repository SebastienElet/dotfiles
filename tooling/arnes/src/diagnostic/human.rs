use super::{Diagnostic, State, human_field};
use std::ffi::OsStr;

mod color;
mod sections;

pub use color::ColorMode;
use color::Colorizer;

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
    color: bool,
}

impl HumanOptions {
    pub fn normal() -> Self {
        Self {
            verbose: false,
            color: false,
        }
    }

    pub fn verbose() -> Self {
        Self {
            verbose: true,
            color: false,
        }
    }

    pub fn includes_healthy(self) -> bool {
        self.verbose
    }

    pub fn with_color(
        mut self,
        mode: ColorMode,
        stdout_is_terminal: bool,
        no_color: Option<&OsStr>,
    ) -> Self {
        self.color = mode.enabled(stdout_is_terminal, no_color);
        self
    }

    fn colorizer(self) -> Colorizer {
        Colorizer::new(self.color)
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
    let structured = diagnostics
        .iter()
        .all(|diagnostic| diagnostic.section().is_some());
    let color = options.colorizer();
    let (section_count, body) = if structured {
        let (count, lines) = sections::render(diagnostics, options, color);
        (Some(count), lines)
    } else {
        (None, render_flat(diagnostics, options, color))
    };
    let mut lines = vec![
        context.heading(section_count),
        color.paint(
            State::Healthy,
            format!("✓ {} healthy", state_count(diagnostics, State::Healthy)),
        ),
    ];
    let unsupported = state_count(diagnostics, State::Unsupported);
    if unsupported > 0 {
        lines.push(color.paint(
            State::Unsupported,
            format!("! {unsupported} unsupported (non-blocking)"),
        ));
    }
    lines.push(String::new());
    lines.extend(body);
    trim_trailing_empty(&mut lines);
    lines.join("\n")
}

fn state_count(diagnostics: &[Diagnostic], state: State) -> usize {
    diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.state == state)
        .count()
}

fn render_flat(diagnostics: &[Diagnostic], options: HumanOptions, color: Colorizer) -> Vec<String> {
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
            let state = format!("{:11}", diagnostic.state.to_string());
            lines.push(format!(
                "  {} {}",
                color.paint(diagnostic.state, state),
                human_field(human.summary())
            ));
        } else {
            group = None;
            lines.push(format!(
                "{} {}: {}",
                color.paint(diagnostic.state, diagnostic.state.to_string()),
                human_field(&diagnostic.resource),
                human_field(&diagnostic.message)
            ));
        }
    }
    lines
}

fn trim_trailing_empty(lines: &mut Vec<String>) {
    while lines.last().is_some_and(String::is_empty) {
        lines.pop();
    }
}
