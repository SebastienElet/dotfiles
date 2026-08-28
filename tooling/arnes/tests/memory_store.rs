#[allow(dead_code)]
#[path = "support/memory.rs"]
mod memory_support;

#[path = "memory_store/admission.rs"]
mod admission;
#[path = "memory_store/atomicity.rs"]
mod atomicity;
#[path = "memory_store/environment.rs"]
mod environment;
#[path = "memory_store/index_format.rs"]
mod index_format;
#[path = "memory_store/layout.rs"]
mod layout;
#[path = "memory_store/locking.rs"]
mod locking;
#[path = "memory_store/support.rs"]
mod support;
