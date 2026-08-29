use super::support::{admit_project, admit_user, memory_root, project_scope};
use agent_memory::{Index, SearchRequest, SearchSelection, Store, search};
use std::fs;

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

#[test]
fn clamps_requested_limits_above_five_and_reports_the_remaining_matches() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("agent-memory");
    let store = Store::open(memory_root(&root)).unwrap();
    let project = project_scope(fixture.path(), "project-a.git");
    for number in 0..6 {
        admit_project(
            &store,
            fixture.path(),
            &project,
            &format!("Bounded agent statement {number}."),
            &["bounded agent"],
            "Established.",
        );
    }
    let index = Index::load_or_rebuild(&store).unwrap().index;

    let selection = search(
        &index,
        SearchRequest {
            query: "bounded agent",
            project_key: project.key(),
            include_user: false,
            limit: 6,
        },
    );

    assert_eq!(selection.selected.len(), 5);
    assert_eq!(selection.omitted_by_limit, 1);
}

#[test]
fn returns_only_diagnostics_visible_to_the_requested_scopes() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("agent-memory");
    let store = Store::open(memory_root(&root)).unwrap();
    let project_a = project_scope(fixture.path(), "project-a.git");
    let project_b = project_scope(fixture.path(), "project-b.git");
    let project_a_id = admit_project(
        &store,
        fixture.path(),
        &project_a,
        "Private diagnostic A.",
        &["private diagnostic"],
        "Established.",
    );
    let project_b_id = admit_project(
        &store,
        fixture.path(),
        &project_b,
        "Private diagnostic B.",
        &["private diagnostic"],
        "Established.",
    );
    let user_id = admit_user(
        &store,
        fixture.path(),
        "Private diagnostic user.",
        &["private diagnostic"],
        "Established.",
    );
    for id in [&project_a_id, &project_b_id, &user_id] {
        fs::write(find_yaml(&root, id), b"not: [valid").unwrap();
    }
    let index = Index::load_or_rebuild(&store).unwrap().index;

    let project_only = search(
        &index,
        SearchRequest {
            query: "unmatched",
            project_key: project_a.key(),
            include_user: false,
            limit: 5,
        },
    );
    let with_user = search(
        &index,
        SearchRequest {
            query: "unmatched",
            project_key: project_a.key(),
            include_user: true,
            limit: 5,
        },
    );

    assert_eq!(diagnostic_ids(&project_only), vec![project_a_id.clone()]);
    let mut expected = vec![project_a_id, user_id];
    expected.sort();
    assert_eq!(diagnostic_ids(&with_user), expected);
    assert!(!diagnostic_ids(&with_user).contains(&project_b_id));
}

fn ids(selection: &SearchSelection) -> Vec<String> {
    selection
        .selected
        .iter()
        .map(|entry| entry.entry_id.clone())
        .collect()
}

fn diagnostic_ids(selection: &SearchSelection) -> Vec<String> {
    let mut ids = selection
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.entry_id.clone())
        .collect::<Vec<_>>();
    ids.sort();
    ids
}

fn find_yaml(root: &std::path::Path, id: &str) -> std::path::PathBuf {
    let user = root.join(format!("entries/user/{id}.yaml"));
    if user.is_file() {
        return user;
    }
    fs::read_dir(root.join("entries/project"))
        .unwrap()
        .map(|entry| entry.unwrap().path().join(format!("{id}.yaml")))
        .find(|path| path.is_file())
        .unwrap()
}
