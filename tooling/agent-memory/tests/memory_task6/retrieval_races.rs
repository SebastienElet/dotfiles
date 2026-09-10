use super::support::*;
use agent_memory::{
    HumanConclusion, OmissionEffect, RetrievalContext, RetrievalRequest, SourceResolution,
    SourceResolver, Status, Store, TransitionContext, confirm, retrieve,
};
use std::sync::{Arc, Barrier};

struct SynchronizedResolver {
    resolution_started: Arc<Barrier>,
    resume_resolution: Arc<Barrier>,
}

impl SourceResolver for SynchronizedResolver {
    fn resolve(&self, _source: &agent_memory::EntrySource) -> SourceResolution {
        self.resolution_started.wait();
        self.resume_resolution.wait();
        valid('a')
    }
}

#[test]
fn terminal_transition_published_during_oracle_resolution_is_omitted()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let (root, store) = open_store(fixture.path())?;
    let yaml = entry_yaml(
        'a',
        "goal",
        &[SourceFixture {
            kind: "local-file",
            locator: "/tmp/concurrent-proof",
            fingerprint: 'a',
        }],
    )?;
    write_user_entry(&root, 'a', &yaml)?;
    let key = project_key(fixture.path())?;
    let selection = select(&store, &key, 5)?;
    let resolution_started = Arc::new(Barrier::new(2));
    let resume_resolution = Arc::new(Barrier::new(2));
    let resolver = SynchronizedResolver {
        resolution_started: Arc::clone(&resolution_started),
        resume_resolution: Arc::clone(&resume_resolution),
    };
    let retrieval_store = Store::open(&memory_root(&root)?)?;
    let clock = FixedClock::at("2026-08-28T01:00:00Z")?;
    let retrieval = std::thread::spawn(move || {
        retrieve(
            RetrievalRequest::new(&selection, &key, true),
            &RetrievalContext::new(&retrieval_store, &clock, &resolver, environment()),
        )
    });
    resolution_started.wait();
    let id = user_entry_id('a', "goal");
    let transition = confirm(
        &id,
        HumanConclusion::goal_achieved("Goal completed concurrently.")?,
        TransitionContext::new(&store, &FixedClock::at("2026-08-28T01:00:00Z")?),
    )?;
    assert_eq!(transition.status(), Status::Achieved);
    resume_resolution.wait();
    let report = retrieval.join().map_err(|_| "worker thread panicked")?;

    assert!(report.injected.is_empty());
    assert_eq!(
        report
            .omitted
            .first()
            .ok_or("missing fixture element")?
            .code,
        "selection_stale"
    );
    assert_eq!(
        report
            .omitted
            .first()
            .ok_or("missing fixture element")?
            .effect,
        OmissionEffect::NotApplied
    );
    Ok(())
}
