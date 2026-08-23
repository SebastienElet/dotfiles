use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

pub const MAX_DETAIL_WORKERS: usize = 8;

pub fn bounded_map<T: Sync, R: Send>(items: &[T], f: impl Fn(&T) -> R + Sync) -> Vec<R> {
    if items.is_empty() {
        return Vec::new();
    }

    let next_index = AtomicUsize::new(0);
    let results = Mutex::new((0..items.len()).map(|_| None).collect::<Vec<Option<R>>>());

    thread::scope(|scope| {
        for _ in 0..items.len().min(MAX_DETAIL_WORKERS) {
            scope.spawn(|| {
                loop {
                    let index = next_index.fetch_add(1, Ordering::Relaxed);
                    let Some(item) = items.get(index) else {
                        break;
                    };
                    let result = f(item);
                    results
                        .lock()
                        .expect("bounded_map results mutex poisoned by a worker panic")[index] =
                        Some(result);
                }
            });
        }
    });

    results
        .into_inner()
        .expect("bounded_map results mutex poisoned after workers joined")
        .into_iter()
        .map(|result| result.expect("bounded_map worker exited without storing its claimed result"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::bounded_map;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn maps_an_empty_slice_to_an_empty_vector() {
        let results: Vec<usize> = bounded_map(&[], |item: &usize| *item);

        assert!(results.is_empty());
    }

    #[test]
    fn preserves_order_and_every_result() {
        let items: Vec<_> = (0..20).collect();

        let results = bounded_map(&items, |item| item * 2);

        assert_eq!(results, (0..20).map(|item| item * 2).collect::<Vec<_>>());
        assert_eq!(results.len(), items.len());
    }

    #[test]
    fn never_runs_more_than_eight_items_simultaneously() {
        let items: Vec<_> = (0..32).collect();
        let active = AtomicUsize::new(0);
        let maximum = AtomicUsize::new(0);

        let results = bounded_map(&items, |item| {
            let current = active.fetch_add(1, Ordering::SeqCst) + 1;
            maximum.fetch_max(current, Ordering::SeqCst);
            thread::sleep(Duration::from_millis(5));
            active.fetch_sub(1, Ordering::SeqCst);
            item * 2
        });

        assert!(maximum.load(Ordering::SeqCst) <= 8);
        assert_eq!(results.len(), items.len());
        assert!(results.iter().all(|result| result % 2 == 0));
    }
}
