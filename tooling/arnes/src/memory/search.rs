use super::ProjectKey;
use super::index::{Index, IndexDiagnostic, IndexRow};
use super::store::document::StoredScope;
use std::cmp::Reverse;
use std::collections::BTreeSet;
use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

#[derive(Clone, Copy, Debug)]
pub struct SearchRequest<'a> {
    pub query: &'a str,
    pub project_key: &'a ProjectKey,
    pub include_user: bool,
    pub limit: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedMemory {
    pub entry_id: String,
    pub kind: String,
    pub path: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchSelection {
    pub selected: Vec<SelectedMemory>,
    pub omitted_by_limit: usize,
    pub diagnostics: Vec<IndexDiagnostic>,
}

pub fn search(index: &Index, request: SearchRequest<'_>) -> SearchSelection {
    let query_tokens = normalized_tokens(request.query);
    let query_set = query_tokens
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let mut matches = index
        .entries()
        .iter()
        .filter(|entry| in_scope(&entry.scope, &request))
        .filter_map(|entry| score(entry, &query_tokens, &query_set).map(|score| (score, entry)))
        .collect::<Vec<_>>();
    matches.sort_by_key(|(score, entry)| (Reverse(*score), entry.id.as_str()));
    let omitted_by_limit = matches.len().saturating_sub(request.limit);
    let selected = matches
        .into_iter()
        .take(request.limit)
        .map(|(_, entry)| SelectedMemory {
            entry_id: entry.id.clone(),
            kind: entry.kind.clone(),
            path: entry.path.clone(),
        })
        .collect();
    SearchSelection {
        selected,
        omitted_by_limit,
        diagnostics: index.diagnostics().to_vec(),
    }
}

fn score(
    entry: &IndexRow,
    query_tokens: &[String],
    query_set: &BTreeSet<&str>,
) -> Option<(usize, usize, usize)> {
    let term_phrases = entry
        .retrieval_terms
        .iter()
        .map(|term| normalized_tokens(term))
        .collect::<Vec<_>>();
    let phrase_count = term_phrases
        .iter()
        .filter(|phrase| contains_phrase(query_tokens, phrase))
        .count();
    let term_tokens = term_phrases
        .iter()
        .flatten()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let statement_tokens = entry
        .statement_tokens
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let matching_terms = term_tokens.intersection(query_set).count();
    let matching_statement = statement_tokens.intersection(query_set).count();
    let distinct_matches = term_tokens
        .union(&statement_tokens)
        .filter(|token| query_set.contains(**token))
        .count();
    (phrase_count > 0 || distinct_matches >= 2).then_some((
        phrase_count,
        matching_terms,
        matching_statement,
    ))
}

pub(super) fn normalized_tokens(value: &str) -> Vec<String> {
    let normalized = value
        .nfkd()
        .filter(|character| !is_combining_mark(*character))
        .flat_map(char::to_lowercase)
        .map(|character| {
            if character.is_alphanumeric() {
                character
            } else {
                ' '
            }
        })
        .collect::<String>();
    normalized.split_whitespace().map(str::to_owned).collect()
}

fn contains_phrase(query: &[String], phrase: &[String]) -> bool {
    !phrase.is_empty()
        && phrase.len() <= query.len()
        && query.windows(phrase.len()).any(|window| window == phrase)
}

fn in_scope(scope: &StoredScope, request: &SearchRequest<'_>) -> bool {
    match scope {
        StoredScope::Project { key } => key == request.project_key.as_str(),
        StoredScope::User => request.include_user,
    }
}
