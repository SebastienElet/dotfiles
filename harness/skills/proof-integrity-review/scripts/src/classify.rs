use crate::{Result, model::Classification};
use regex::RegexBuilder;
use std::collections::{BTreeMap, BTreeSet};

pub fn classify(paths: &[String]) -> Result<Classification> {
    let rules: BTreeMap<String, Vec<String>> = serde_json::from_str(include_str!("rules.json"))?;
    let paths: BTreeSet<_> = paths.iter().cloned().collect();
    let mut categories = BTreeMap::new();
    let mut matched = BTreeSet::new();
    for (category, patterns) in rules {
        let regex = RegexBuilder::new(&patterns.join("|"))
            .case_insensitive(true)
            .build()?;
        let hits: Vec<_> = paths
            .iter()
            .filter(|path| regex.is_match(path))
            .cloned()
            .collect();
        if !hits.is_empty() {
            matched.extend(hits.iter().cloned());
            categories.insert(category, hits);
        }
    }
    Ok(Classification {
        schema_version: 1,
        triggered: !matched.is_empty(),
        categories,
        unmatched_paths: paths.difference(&matched).cloned().collect(),
        matched_paths: matched.into_iter().collect(),
    })
}
