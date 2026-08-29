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
fn terminal_transition_published_during_oracle_resolution_is_omitted() {
    let fixture = tempfile::tempdir().unwrap();
    let (root, store) = open_store(fixture.path());
    let yaml = entry_yaml(
        'a',
        "goal",
        &[SourceFixture {
            kind: "local-file",
            locator: "/tmp/concurrent-proof",
            fingerprint: 'a',
        }],
    );
    write_user_entry(&root, 'a', &yaml);
    let key = project_key(fixture.path());
    let selection = select(&store, &key, 5);
    let resolution_started = Arc::new(Barrier::new(2));
    let resume_resolution = Arc::new(Barrier::new(2));
    let resolver = SynchronizedResolver {
        resolution_started: Arc::clone(&resolution_started),
        resume_resolution: Arc::clone(&resume_resolution),
    };
    let retrieval_store = Store::open(memory_root(&root)).unwrap();
    let retrieval = std::thread::spawn(move || {
        retrieve(
            RetrievalRequest::new(&selection, &key, true),
            RetrievalContext::new(
                &retrieval_store,
                &FixedClock::at("2026-08-28T01:00:00Z"),
                &resolver,
                environment(),
            ),
        )
    });
    resolution_started.wait();
    let transition = confirm(
        "mem_aaaaaaaaaaaaaaaaaaaaaaaa",
        HumanConclusion::goal_achieved("Goal completed concurrently.").unwrap(),
        TransitionContext::new(&store, &FixedClock::at("2026-08-28T01:00:00Z")),
    )
    .unwrap();
    assert_eq!(transition.status(), Status::Achieved);
    resume_resolution.wait();
    let report = retrieval.join().unwrap();

    assert!(report.injected.is_empty());
    assert_eq!(report.omitted[0].code, "selection_stale");
    assert_eq!(report.omitted[0].effect, OmissionEffect::NotApplied);
}
