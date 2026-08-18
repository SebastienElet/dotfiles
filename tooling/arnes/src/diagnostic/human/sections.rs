use super::color::Colorizer;
use super::{HumanOptions, human_field};
use crate::diagnostic::{Diagnostic, State};

struct Section<'a> {
    key: &'a str,
    label: &'a str,
    position: usize,
    diagnostics: Vec<(usize, &'a Diagnostic)>,
}

pub(super) fn render(
    diagnostics: &[Diagnostic],
    options: HumanOptions,
    color: Colorizer,
) -> (usize, Vec<String>) {
    let mut sections = collect(diagnostics);
    let count = sections.len();
    sections.sort_by(|left, right| {
        section_rank(right)
            .cmp(&section_rank(left))
            .then(left.position.cmp(&right.position))
    });
    let mut lines = Vec::new();
    for section in sections {
        render_section(&mut lines, section, options, color);
    }
    (count, lines)
}

fn collect(diagnostics: &[Diagnostic]) -> Vec<Section<'_>> {
    let mut sections: Vec<Section<'_>> = Vec::new();
    for (index, diagnostic) in diagnostics.iter().enumerate() {
        let metadata = diagnostic.section().expect("structured diagnostic section");
        match sections
            .iter_mut()
            .find(|section| section.key == metadata.key())
        {
            Some(section) => section.diagnostics.push((index, diagnostic)),
            None => sections.push(Section {
                key: metadata.key(),
                label: metadata.label(),
                position: index,
                diagnostics: vec![(index, diagnostic)],
            }),
        }
    }
    sections
}

fn render_section(
    lines: &mut Vec<String>,
    mut section: Section<'_>,
    options: HumanOptions,
    color: Colorizer,
) {
    if !options.includes_healthy()
        && section
            .diagnostics
            .iter()
            .all(|(_, diagnostic)| diagnostic.state == State::Healthy)
    {
        return;
    }
    if !lines.is_empty() {
        lines.push(String::new());
    }
    lines.push(human_field(section.label));
    lines.push(format!("  {}", counts(&section.diagnostics, color)));
    lines.push(String::new());
    section.diagnostics.sort_by(|left, right| {
        state_rank(right.1.state)
            .cmp(&state_rank(left.1.state))
            .then(left.0.cmp(&right.0))
    });
    render_diagnostics(lines, &section.diagnostics, options, color);
}

fn render_diagnostics(
    lines: &mut Vec<String>,
    diagnostics: &[(usize, &Diagnostic)],
    options: HumanOptions,
    color: Colorizer,
) {
    let mut group = None;
    for (_, diagnostic) in diagnostics
        .iter()
        .filter(|(_, diagnostic)| options.includes_healthy() || diagnostic.state != State::Healthy)
    {
        let human = diagnostic.human();
        let next_group = human
            .map(|metadata| metadata.group())
            .filter(|group| !group.is_empty());
        if next_group != group {
            if let Some(label) = next_group {
                lines.push(format!("  {}", human_field(label)));
            }
            group = next_group;
        }
        let indent = if group.is_some() { "    " } else { "  " };
        let summary = human.map_or(diagnostic.message.as_str(), |metadata| metadata.summary());
        lines.push(format!(
            "{indent}{} {}",
            color.paint(
                diagnostic.state,
                diagnostic.state.to_string().to_uppercase()
            ),
            human_field(summary)
        ));
        if let Some(metadata) = human {
            for detail in metadata.details() {
                lines.push(format!(
                    "{indent}  {:9} {}",
                    human_field(detail.label()),
                    human_field(detail.value())
                ));
            }
        }
    }
}

fn counts(diagnostics: &[(usize, &Diagnostic)], color: Colorizer) -> String {
    let issues = count(diagnostics, |state| {
        matches!(state, State::Error | State::Drift)
    });
    let unsupported = count(diagnostics, |state| state == State::Unsupported);
    let healthy = count(diagnostics, |state| state == State::Healthy);
    let mut parts = Vec::new();
    if issues > 0 {
        let state = if count(diagnostics, |state| state == State::Error) > 0 {
            State::Error
        } else {
            State::Drift
        };
        parts.push(color.paint(
            state,
            format!("{issues} {}", if issues == 1 { "issue" } else { "issues" }),
        ));
    }
    if unsupported > 0 {
        parts.push(color.paint(State::Unsupported, format!("{unsupported} unsupported")));
    }
    parts.push(color.paint(State::Healthy, format!("{healthy} healthy")));
    parts.join(" · ")
}

fn count(diagnostics: &[(usize, &Diagnostic)], predicate: impl Fn(State) -> bool) -> usize {
    diagnostics
        .iter()
        .filter(|(_, diagnostic)| predicate(diagnostic.state))
        .count()
}

fn section_rank(section: &Section<'_>) -> u8 {
    section
        .diagnostics
        .iter()
        .map(|(_, diagnostic)| state_rank(diagnostic.state))
        .max()
        .unwrap_or(0)
}

fn state_rank(state: State) -> u8 {
    match state {
        State::Error => 3,
        State::Drift => 2,
        State::Unsupported => 1,
        State::Healthy => 0,
    }
}
