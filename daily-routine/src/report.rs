use crate::config::Config;
use crate::model::{Category, Report, ReportItem};
use crate::util::truncate_chars;

const CATEGORIES: [Category; 4] = [
    Category::Review,
    Category::Retour,
    Category::Linear,
    Category::Suivant,
];
const THINGS_TITLE_MAX_CHARS: usize = 120;

pub fn things_title(item: &ReportItem) -> String {
    truncate_chars(
        &format!(
            "[{}] {} {}",
            item.category.label(),
            item.reference,
            item.title
        ),
        THINGS_TITLE_MAX_CHARS,
    )
}

pub fn render(config: &Config, report: &Report) -> String {
    let mut output = String::new();

    for (category_index, category) in CATEGORIES.into_iter().enumerate() {
        if category_index > 0 {
            output.push('\n');
        }
        output.push_str(category.label());
        output.push('\n');

        if report
            .warnings
            .iter()
            .any(|warning| warning.categories.contains(&category))
        {
            output.push_str("(partial; see stderr)\n");
        }

        let mut category_items = report
            .items
            .iter()
            .filter(|item| item.category == category)
            .peekable();
        if category_items.peek().is_none() {
            if !report
                .warnings
                .iter()
                .any(|warning| warning.categories.contains(&category))
            {
                output.push_str("(none)\n");
            }
            continue;
        }

        for item in category_items {
            render_item(config, item, &mut output);
        }
    }

    output
}

fn render_item(config: &Config, item: &ReportItem, output: &mut String) {
    let track = config
        .tracks
        .get(item.track_index)
        .map_or("unknown", |track| track.name.as_str());
    let reasons = if item.reasons.is_empty() {
        "none".to_owned()
    } else {
        item.reasons
            .iter()
            .map(|reason| reason.label())
            .collect::<Vec<_>>()
            .join(", ")
    };

    output.push_str(&format!("- {} {}\n", item.reference, item.title));
    output.push_str(&format!("  Track: {track}\n"));
    output.push_str(&format!("  Reasons: {reasons}\n"));
    output.push_str(&format!("  URL: {}\n", item.url));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::model::{Category, LinearReason, Report, ReportItem, Warning};

    fn config() -> Config {
        Config::parse(
            r#"
                stale_days = 7
                next_count = 3

                [[tracks]]
                name = "Application"
                teams = ["APP"]

                [[tracks.repos]]
                provider = "github"
                path = "ExampleOrg/application"
            "#,
        )
        .unwrap()
    }

    fn item(category: Category, reference: &str, title: &str) -> ReportItem {
        ReportItem {
            category,
            track_index: 0,
            reference: reference.to_owned(),
            title: title.to_owned(),
            url: format!("https://example.test/{reference}"),
            event_at: "2026-08-11T08:00:00Z".to_owned(),
            reasons: Vec::new(),
            priority: None,
        }
    }

    #[test]
    fn things_titles_are_stable_and_truncate_the_complete_unicode_title() {
        let retour = item(Category::Retour, "#101", "Improve validation flow");
        assert_eq!(
            things_title(&retour),
            "[RETOUR] #101 Improve validation flow"
        );

        let accented = item(Category::Linear, "APP-123", &"é".repeat(120));
        let title = things_title(&accented);
        assert_eq!(title.chars().count(), 120);
        assert!(title.ends_with('é'));
        assert!(title.starts_with("[LINEAR] APP-123 "));
    }

    #[test]
    fn render_keeps_category_order_and_shows_item_context() {
        let report = Report {
            items: vec![ReportItem {
                reasons: vec![LinearReason::MissingPriority],
                ..item(Category::Linear, "APP-123", "Improve validation")
            }],
            warnings: Vec::new(),
        };

        let output = render(&config(), &report);

        assert!(output.find("REVIEW").unwrap() < output.find("RETOUR").unwrap());
        assert!(output.find("RETOUR").unwrap() < output.find("LINEAR").unwrap());
        assert!(output.find("LINEAR").unwrap() < output.find("SUIVANT").unwrap());
        assert!(output.contains("Track: Application"));
        assert!(output.contains("Reasons: missing priority"));
        assert!(output.contains("URL: https://example.test/APP-123"));
    }

    #[test]
    fn render_distinguishes_successful_empty_and_partial_categories() {
        let report = Report {
            items: Vec::new(),
            warnings: vec![Warning {
                categories: vec![Category::Review, Category::Retour],
                message: "provider unavailable".to_owned(),
            }],
        };

        let output = render(&config(), &report);

        assert_eq!(output.matches("(partial; see stderr)").count(), 2);
        assert_eq!(output.matches("(none)").count(), 2);
    }

    #[test]
    fn render_always_shows_reasons_even_when_there_are_none() {
        let report = Report {
            items: vec![item(Category::Review, "#5", "Review this")],
            warnings: Vec::new(),
        };

        assert!(render(&config(), &report).contains("Reasons: none"));
    }
}
