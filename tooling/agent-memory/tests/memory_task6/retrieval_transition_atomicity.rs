use super::support::*;
use agent_memory::{HumanConclusion, Status, Store, TransitionContext, confirm};
use std::sync::{Arc, Barrier};

#[test]
fn concurrent_human_conclusions_publish_exactly_one_terminal_transition() {
    let fixture = tempfile::tempdir().unwrap();
    let (root, store) = open_store(fixture.path());
    let yaml = entry_yaml(
        'd',
        "goal",
        &[SourceFixture {
            kind: "user-decision",
            locator: "decision:concurrent-transition",
            fingerprint: 'd',
        }],
    );
    write_user_entry(&root, 'd', &yaml);
    let barrier = Arc::new(Barrier::new(3));
    let id = "mem_dddddddddddddddddddddddd";
    let attempts = [
        (Status::Achieved, true, "Goal achieved."),
        (Status::Abandoned, false, "Goal abandoned."),
    ]
    .map(|(status, achieved, reason)| {
        let worker_barrier = Arc::clone(&barrier);
        let worker_store = Store::open(memory_root(&root)).unwrap();
        std::thread::spawn(move || {
            worker_barrier.wait();
            let conclusion = if achieved {
                HumanConclusion::goal_achieved(reason)
            } else {
                HumanConclusion::goal_abandoned(reason)
            }
            .unwrap();
            (
                status,
                confirm(
                    id,
                    conclusion,
                    TransitionContext::new(&worker_store, &FixedClock::at("2026-08-28T01:00:00Z")),
                ),
            )
        })
    });
    barrier.wait();
    let results = attempts.map(|attempt| attempt.join().unwrap());
    let success = results
        .iter()
        .find_map(|(status, result)| result.as_ref().ok().map(|_| *status))
        .unwrap();

    assert_eq!(
        results.iter().filter(|(_, result)| result.is_ok()).count(),
        1
    );
    assert_eq!(
        results
            .iter()
            .filter_map(|(_, result)| result.as_ref().err())
            .map(agent_memory::MemoryError::code)
            .collect::<Vec<_>>(),
        ["entry_not_active"]
    );
    let stored = store.load(id).unwrap().unwrap();
    assert_eq!(stored.status(), success);
    assert_eq!(stored.transition().unwrap().to(), success);
}
