#![cfg(test)]

#[path = "memory_index/diagnostics.rs"]
mod diagnostics;
#[path = "memory_index/index_bounds.rs"]
mod index_bounds;
#[path = "memory_index/races.rs"]
mod races;
#[path = "memory_index/rebuild.rs"]
mod rebuild;
#[path = "memory_index/security.rs"]
mod security;
use crate::index_support as support;
