#![cfg(test)]

#[path = "memory_index/support.rs"]
mod index_support;
#[path = "support/memory.rs"]
mod memory_support;
#[path = "memory_task6/support.rs"]
mod retrieval_support;

mod crate_boundary;
mod memory_admission;
mod memory_cache;
mod memory_cli;
mod memory_concurrency;
mod memory_error_classes;
mod memory_eval_trace;
mod memory_git_sources;
mod memory_hooks;
mod memory_identity;
mod memory_index;
mod memory_model;
mod memory_oracle;
mod memory_retrieval;
mod memory_search;
mod memory_sources;
mod memory_store;
mod memory_validation;
