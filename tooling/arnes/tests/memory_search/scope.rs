use super::support::{admit_project, admit_user, memory_root, project_scope};
use arnes::memory::{Index, SearchRequest, SearchSelection, Store, search};

#[test]
fn isolates_the_requested_project_and_optionally_includes_user_scope() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("agent-memory");
    let store = Store::open(memory_root(&root)).unwrap();
    let project_a = project_scope(fixture.path(), "project-a.git");
    let project_b = project_scope(fixture.path(), "project-b.git");
    let selected_project = admit_project(
        &store,
        fixture.path(),
        &project_a,
        "Agent project A.",
        &["agent"],
        "Established.",
    );
    admit_project(
        &store,
        fixture.path(),
        &project_b,
        "Agent project B.",
        &["agent"],
        "Established.",
    );
    let selected_user = admit_user(
        &store,
        fixture.path(),
        "Agent user.",
        &["agent"],
        "Established.",
    );
    let index = Index::load_or_rebuild(&store).unwrap().index;
    let with_user = search(
        &index,
        SearchRequest {
            query: "agent",
            project_key: project_a.key(),
            include_user: true,
            limit: 5,
        },
    );
    let project_only = search(
        &index,
        SearchRequest {
            query: "agent",
            project_key: project_a.key(),
            include_user: false,
            limit: 5,
        },
    );

    let mut expected = vec![selected_project.clone(), selected_user];
    expected.sort();
    assert_eq!(ids(&with_user), expected);
    assert_eq!(ids(&project_only), vec![selected_project]);
}

#[test]
fn limits_results_and_reports_only_matches_omitted_by_the_limit() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("agent-memory");
    let store = Store::open(memory_root(&root)).unwrap();
    let project = project_scope(fixture.path(), "project-a.git");
    for number in 0..6 {
        admit_project(
            &store,
            fixture.path(),
            &project,
            &format!("Agent statement {number}."),
            &["agent"],
            "Established.",
        );
    }
    let index = Index::load_or_rebuild(&store).unwrap().index;
    let selection = search(
        &index,
        SearchRequest {
            query: "agent",
            project_key: project.key(),
            include_user: false,
            limit: 5,
        },
    );

    assert_eq!(selection.selected.len(), 5);
    assert_eq!(selection.omitted_by_limit, 1);
    assert!(selection.diagnostics.is_empty());
}

fn ids(selection: &SearchSelection) -> Vec<String> {
    selection
        .selected
        .iter()
        .map(|entry| entry.entry_id.clone())
        .collect()
}
