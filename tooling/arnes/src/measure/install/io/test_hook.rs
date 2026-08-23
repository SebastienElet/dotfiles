use std::cell::RefCell;
use std::thread_local;

thread_local! {
    static AFTER_PUBLISH_HOOK: RefCell<Option<Box<dyn FnOnce()>>> = RefCell::new(None);
}

pub fn set_after_publish_hook(hook: impl FnOnce() + 'static) {
    AFTER_PUBLISH_HOOK.with(|slot| slot.replace(Some(Box::new(hook))));
}

pub fn run_after_publish_hook() {
    AFTER_PUBLISH_HOOK.with(|slot| {
        if let Some(hook) = slot.take() {
            hook();
        }
    });
}
