#![cfg(test)]

use crate::index_support as support;

#[path = "memory_search/freshness.rs"]
mod freshness;
#[path = "memory_search/ranking.rs"]
mod ranking;
#[path = "memory_search/scope.rs"]
mod scope;
