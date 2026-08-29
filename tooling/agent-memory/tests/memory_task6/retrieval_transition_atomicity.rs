use super::support::*;
use agent_memory::{
    Clock, HumanConclusion, Status, Store, TransitionContext, UtcTimestamp, confirm,
    parse_utc_timestamp,
};
use std::sync::{Arc, Barrier};

struct SynchronizedClock {
    barrier: Arc<Barrier>,
    timestamp: UtcTimestamp,
}

impl SynchronizedClock {
    fn at(timestamp: &str, barrier: Arc<Barrier>) -> Self {
        Self {
            barrier,
            timestamp: parse_utc_timestamp(timestamp).unwrap(),
        }
    }
}

impl Clock for SynchronizedClock {
    fn now(&self) -> UtcTimestamp {
        self.barrier.wait();
        self.timestamp.clone()
    }
}

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
    let loaded_active = Arc::new(Barrier::new(3));
    let id = "mem_dddddddddddddddddddddddd";
    let attempts = [
        (Status::Achieved, true, "Goal achieved."),
        (Status::Abandoned, false, "Goal abandoned."),
    ]
    .map(|(status, achieved, reason)| {
        let worker_barrier = Arc::clone(&loaded_active);
        let worker_store = Store::open(memory_root(&root)).unwrap();
        std::thread::spawn(move || {
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
                    TransitionContext::new(
                        &worker_store,
                        &SynchronizedClock::at("2026-08-28T01:00:00Z", worker_barrier),
                    ),
                ),
            )
        })
    });
    loaded_active.wait();
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
