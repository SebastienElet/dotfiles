use super::support::{admit_project, memory_root, project_scope};
use arnes::memory::{Index, SearchRequest, SearchSelection, Store, search};

#[test]
fn normalizes_nfkd_diacritics_case_and_separators_for_phrase_and_statement_matches() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("agent-memory");
    let store = Store::open(memory_root(&root)).unwrap();
    let project = project_scope(fixture.path(), "project-a.git");
    let alias = admit_project(
        &store,
        fixture.path(),
        &project,
        "A release rule.",
        &["Déploiement---mémoire"],
        "Established.",
    );
    let statement = admit_project(
        &store,
        fixture.path(),
        &project,
        "La MEMOIRE accompagne le deploiement.",
        &["release process"],
        "Established.",
    );
    let index = Index::load_or_rebuild(&store).unwrap().index;
    let selection = search(
        &index,
        SearchRequest {
            query: "deploiement mémoire",
            project_key: project.key(),
            include_user: true,
            limit: 5,
        },
    );

    assert_eq!(ids(&selection), vec![alias, statement]);
}

#[test]
fn ranks_phrase_count_then_term_tokens_then_statement_tokens_and_ties_by_id() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("agent-memory");
    let store = Store::open(memory_root(&root)).unwrap();
    let project = project_scope(fixture.path(), "project-a.git");
    let two_phrases = admit_project(
        &store,
        fixture.path(),
        &project,
        "Unrelated statement.",
        &["alpha beta", "beta gamma"],
        "Established.",
    );
    let one_phrase_more_tokens = admit_project(
        &store,
        fixture.path(),
        &project,
        "Alpha beta gamma.",
        &["alpha beta", "gamma delta"],
        "Established.",
    );
    let statement_only = admit_project(
        &store,
        fixture.path(),
        &project,
        "Alpha beta gamma statement match.",
        &["unrelated term"],
        "Established.",
    );
    let tie_a = admit_project(
        &store,
        fixture.path(),
        &project,
        "Alpha beta first.",
        &["unrelated first"],
        "Established.",
    );
    let tie_b = admit_project(
        &store,
        fixture.path(),
        &project,
        "Alpha beta second.",
        &["unrelated second"],
        "Established.",
    );
    let index = Index::load_or_rebuild(&store).unwrap().index;
    let selection = search(
        &index,
        SearchRequest {
            query: "alpha beta gamma",
            project_key: project.key(),
            include_user: false,
            limit: 10,
        },
    );

    let mut ties = [tie_a, tie_b];
    ties.sort();
    assert_eq!(
        ids(&selection),
        vec![
            two_phrases,
            one_phrase_more_tokens,
            statement_only,
            ties[0].clone(),
            ties[1].clone(),
        ]
    );
}

#[test]
fn requires_one_term_phrase_or_two_distinct_tokens() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("agent-memory");
    let store = Store::open(memory_root(&root)).unwrap();
    let project = project_scope(fixture.path(), "project-a.git");
    let phrase = admit_project(
        &store,
        fixture.path(),
        &project,
        "Unrelated.",
        &["agent"],
        "Established.",
    );
    admit_project(
        &store,
        fixture.path(),
        &project,
        "Only agent appears.",
        &["unrelated term"],
        "Established.",
    );
    let index = Index::load_or_rebuild(&store).unwrap().index;
    let agent = search(
        &index,
        SearchRequest {
            query: "agent",
            project_key: project.key(),
            include_user: false,
            limit: 5,
        },
    );
    let unrelated = search(
        &index,
        SearchRequest {
            query: "sans rapport",
            project_key: project.key(),
            include_user: false,
            limit: 5,
        },
    );

    assert_eq!(ids(&agent), vec![phrase]);
    assert!(unrelated.selected.is_empty());
}

fn ids(selection: &SearchSelection) -> Vec<String> {
    selection
        .selected
        .iter()
        .map(|entry| entry.entry_id.clone())
        .collect()
}
