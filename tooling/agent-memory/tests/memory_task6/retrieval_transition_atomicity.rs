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
    fn at(timestamp: &str, barrier: Arc<Barrier>) -> Result<Self, agent_memory::MemoryError> {
        Ok(Self {
            barrier,
            timestamp: parse_utc_timestamp(timestamp)?,
        })
    }
}

impl Clock for SynchronizedClock {
    fn now(&self) -> UtcTimestamp {
        self.barrier.wait();
        self.timestamp.clone()
    }
}

#[test]
fn concurrent_human_conclusions_publish_exactly_one_terminal_transition()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let (root, store) = open_store(fixture.path())?;
    let yaml = entry_yaml(
        'd',
        "goal",
        &[SourceFixture {
            kind: "user-decision",
            locator: "decision:concurrent-transition",
            fingerprint: 'd',
        }],
    )?;
    write_user_entry(&root, 'd', &yaml)?;
    let loaded_active = Arc::new(Barrier::new(3));
    let id = user_entry_id('d', "goal");
    let attempts = [
        (Status::Achieved, true, "Goal achieved."),
        (Status::Abandoned, false, "Goal abandoned."),
    ]
    .into_iter()
    .map(
        |(status, achieved, reason)| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
            let worker_barrier = Arc::clone(&loaded_active);
            let worker_store = Store::open(&memory_root(&root)?)?;
            let id = id.clone();
            Ok(std::thread::spawn(
                move || -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                    let conclusion = if achieved {
                        HumanConclusion::goal_achieved(reason)
                    } else {
                        HumanConclusion::goal_abandoned(reason)
                    }?;
                    Ok((
                        status,
                        confirm(
                            &id,
                            conclusion,
                            TransitionContext::new(
                                &worker_store,
                                &SynchronizedClock::at("2026-08-28T01:00:00Z", worker_barrier)?,
                            ),
                        ),
                    ))
                },
            ))
        },
    )
    .collect::<Result<Vec<_>, _>>()?;
    loaded_active.wait();
    let results = attempts
        .into_iter()
        .map(|attempt| attempt.join().map_err(|_| "worker thread panicked")?)
        .collect::<Result<Vec<_>, _>>()?;
    let success = results
        .iter()
        .find_map(|(status, result)| result.as_ref().ok().map(|_| *status))
        .ok_or("missing fixture value")?;

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
    let stored = store.load(&id)?.ok_or("missing fixture value")?;
    assert_eq!(stored.status(), success);
    assert_eq!(
        stored.transition().ok_or("missing fixture value")?.to(),
        success
    );
    Ok(())
}
