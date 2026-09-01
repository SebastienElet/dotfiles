#[allow(dead_code)]
#[path = "support/memory.rs"]
mod memory_support;

#[path = "memory_index/support.rs"]
#[allow(dead_code)]
mod support;

#[path = "memory_search/freshness.rs"]
mod freshness;
#[path = "memory_search/ranking.rs"]
mod ranking;
#[path = "memory_search/scope.rs"]
mod scope;
