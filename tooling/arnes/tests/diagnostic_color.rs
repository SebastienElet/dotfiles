use arnes::diagnostic::{
    ColorMode, Diagnostic, HumanContext, HumanOptions, HumanSection, Report, State,
};
use std::ffi::OsStr;

fn report() -> Report {
    Report::new(vec![
        Diagnostic::new("manifest", State::Healthy, "manifest is valid"),
        Diagnostic::new(
            "rules",
            State::Unsupported,
            "cursor does not expose native user rules",
        ),
        Diagnostic::new("skills", State::Drift, "destination is missing"),
        Diagnostic::new("config", State::Error, "settings.json could not be read"),
    ])
}

fn context() -> HumanContext {
    HumanContext::new("Diagnostics")
}

fn section(key: &str, label: &str) -> HumanSection {
    HumanSection::new(key, label)
}

fn colored(options: HumanOptions) -> HumanOptions {
    options.with_color(ColorMode::Always, false, Some(OsStr::new("1")))
}

fn strip_ansi(value: &str) -> String {
    ["\x1b[31m", "\x1b[32m", "\x1b[33m", "\x1b[36m", "\x1b[0m"]
        .into_iter()
        .fold(value.to_owned(), |plain, sequence| {
            plain.replace(sequence, "")
        })
}

#[test]
fn auto_colors_terminal_output() {
    let output = report().human(
        &context(),
        HumanOptions::normal().with_color(ColorMode::Auto, true, None),
    );

    assert!(output.contains('\x1b'));
}

#[test]
fn auto_keeps_redirected_output_plain() {
    let output = report().human(
        &context(),
        HumanOptions::normal().with_color(ColorMode::Auto, false, None),
    );

    assert!(!output.contains('\x1b'));
}

#[test]
fn auto_treats_empty_no_color_as_absent_and_non_empty_as_disabled() {
    for no_color in [None, Some(OsStr::new(""))] {
        let output = report().human(
            &context(),
            HumanOptions::normal().with_color(ColorMode::Auto, true, no_color),
        );
        assert!(output.contains('\x1b'), "{no_color:?}");
    }
    let output = report().human(
        &context(),
        HumanOptions::normal().with_color(ColorMode::Auto, true, Some(OsStr::new("1"))),
    );
    assert!(!output.contains('\x1b'));
}

#[cfg(unix)]
#[test]
fn auto_treats_non_utf8_no_color_as_non_empty() {
    use std::os::unix::ffi::OsStringExt;

    let no_color = std::ffi::OsString::from_vec(vec![0xff]);
    let output = report().human(
        &context(),
        HumanOptions::normal().with_color(ColorMode::Auto, true, Some(&no_color)),
    );

    assert!(!output.contains('\x1b'));
}

#[test]
fn always_overrides_terminal_and_no_color() {
    let output = report().human(&context(), colored(HumanOptions::normal()));

    assert!(output.contains('\x1b'));
}

#[test]
fn never_keeps_output_plain() {
    let output = report().human(
        &context(),
        HumanOptions::normal().with_color(ColorMode::Never, true, None),
    );

    assert!(!output.contains('\x1b'));
}

#[test]
fn colors_every_visible_state_and_summary() {
    let normal = report().human(&context(), colored(HumanOptions::normal()));
    let verbose = report().human(&context(), colored(HumanOptions::verbose()));

    assert!(normal.contains("\x1b[32m✓ 1 healthy\x1b[0m"));
    assert!(normal.contains("\x1b[36m! 1 unsupported (non-blocking)\x1b[0m"));
    assert!(normal.contains("\x1b[36munsupported\x1b[0m rules"));
    assert!(normal.contains("\x1b[33mdrift\x1b[0m skills"));
    assert!(normal.contains("\x1b[31merror\x1b[0m config"));
    assert!(verbose.contains("\x1b[32mhealthy\x1b[0m manifest"));
}

#[test]
fn stripping_color_preserves_normal_and_verbose_output_byte_for_byte() {
    for options in [HumanOptions::normal(), HumanOptions::verbose()] {
        let plain = report().human(&context(), options);
        let styled = report().human(&context(), colored(options));

        assert_eq!(strip_ansi(&styled), plain);
    }
}

#[test]
fn structured_output_colors_states_and_each_summary_segment() {
    let report = Report::new(vec![
        Diagnostic::new("skills", State::Healthy, "current")
            .with_human_section(section("cursor:user", "CURSOR")),
        Diagnostic::new("skills", State::Unsupported, "inventory unavailable")
            .with_human_section(section("cursor:user", "CURSOR")),
        Diagnostic::new("skills", State::Drift, "destination missing")
            .with_human_section(section("cursor:user", "CURSOR")),
        Diagnostic::new("skills", State::Error, "source unreadable")
            .with_human_section(section("cursor:user", "CURSOR")),
    ]);

    let output = report.human(&context(), colored(HumanOptions::verbose()));

    assert!(output.contains(
        "  \x1b[31m2 issues\x1b[0m · \x1b[36m1 unsupported\x1b[0m · \x1b[32m1 healthy\x1b[0m"
    ));
    assert!(output.contains("  \x1b[31mERROR\x1b[0m source unreadable"));
    assert!(output.contains("  \x1b[33mDRIFT\x1b[0m destination missing"));
    assert!(output.contains("  \x1b[36mUNSUPPORTED\x1b[0m inventory unavailable"));
    assert!(output.contains("  \x1b[32mHEALTHY\x1b[0m current"));
}

#[test]
fn drift_only_issue_summary_is_yellow() {
    let report = Report::new(vec![
        Diagnostic::new("skills", State::Drift, "destination missing")
            .with_human_section(section("cursor:user", "CURSOR")),
    ]);

    let output = report.human(&context(), colored(HumanOptions::normal()));

    assert!(output.contains("  \x1b[33m1 issue\x1b[0m · \x1b[32m0 healthy\x1b[0m"));
}

#[test]
fn json_never_contains_renderer_ansi() {
    let report = report();
    let _ = report.human(&context(), colored(HumanOptions::verbose()));

    assert!(!report.json().unwrap().contains('\x1b'));
}
