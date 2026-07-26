use std::collections::{BTreeMap, HashMap, HashSet};
use std::hash::Hash;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::time::{Duration, Instant};

use itertools::Itertools as _;
use tokio::sync::{Mutex, MutexGuard, Notify};

#[derive(Debug)]
pub struct Queue<K> {
    queue: Mutex<QueueInner<K>>,
    notify: Arc<Notify>,
}

#[derive(Debug)]
struct QueueInner<K> {
    scheduled: HashMap<K, Instant>,
    queue: BTreeMap<Instant, HashSet<K>>,
}

#[expect(
    clippy::single_char_lifetime_names,
    reason = "this is the guard's lifetime"
)]
struct QueueInnerGuard<'a, K> {
    guard: MutexGuard<'a, QueueInner<K>>,
    notify: Arc<Notify>,
    earliest: Option<Instant>,
}

impl<K> Drop for QueueInnerGuard<'_, K> {
    fn drop(&mut self) {
        let earliest = self.guard.queue.first_key_value().map(|(k, _)| *k);
        match (self.earliest, earliest) {
            (Some(a), Some(b)) if a > b => self.notify.notify_waiters(),
            (Some(_), None) | (None, Some(_)) => self.notify.notify_waiters(),
            (Some(_), Some(_)) | (None, None) => {}
        }
    }
}

impl<K> Deref for QueueInnerGuard<'_, K> {
    type Target = QueueInner<K>;

    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}

impl<K> DerefMut for QueueInnerGuard<'_, K> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard
    }
}

impl<K> Queue<K>
where
    K: Hash + Eq + Clone + Send + Sync + std::fmt::Display,
{
    #[must_use]
    async fn write(&self) -> QueueInnerGuard<'_, K> {
        let guard = self.queue.lock().await;
        let earliest = guard.queue.first_key_value().map(|(k, _)| *k);
        QueueInnerGuard {
            notify: Arc::clone(&self.notify),
            guard,
            earliest,
        }
    }

    pub fn new() -> Self {
        Self {
            queue: Mutex::new(QueueInner {
                scheduled: HashMap::new(),
                queue: BTreeMap::new(),
            }),
            notify: Arc::new(Notify::new()),
        }
    }

    pub async fn remove_key(&self, key: K) {
        let mut guard = self.write().await;
        let Some(when) = guard.scheduled.remove(&key) else {
            return;
        };

        guard.queue.entry(when).and_modify(|v| {
            v.remove(&key);
        });
    }

    pub async fn schedule_at(&self, key: K, when: Instant) {
        let mut guard = self.write().await;
        if let Some(scheduled) = guard.scheduled.get(&key).copied()
            && scheduled > when
            && let Some(entry) = guard.queue.get_mut(&scheduled)
        {
            entry.remove(&key);
        }

        guard.queue.entry(when).or_default().insert(key.clone());
        guard.scheduled.insert(key, when);
    }

    pub async fn schedule_at_bulk(&self, keys: HashSet<K>, when: Instant) {
        if keys.is_empty() {
            return;
        }

        let dkeys = keys.iter().map(|v| format!("{v}")).collect_vec();
        let dwhen = when.duration_since(Instant::now());
        tracing::trace!(keys = ?dkeys, when = ?dwhen, "scheduling keys in bulk");

        let mut guard = self.write().await;
        for key in &keys {
            if let Some(scheduled) = guard.scheduled.get(key).copied()
                && scheduled > when
                && let Some(entry) = guard.queue.get_mut(&scheduled)
            {
                entry.remove(key);
            }
        }

        guard.queue.entry(when).or_default().extend(keys.clone());
        guard.scheduled.extend(keys.into_iter().map(|k| (k, when)));
    }

    async fn earliest(&self) -> Option<Instant> {
        self.queue
            .lock()
            .await
            .queue
            .first_key_value()
            .map(|(k, _)| *k)
    }

    async fn remove_expired_at(
        &self,
        at: Instant,
    ) -> BTreeMap<Instant, HashSet<K>> {
        let mut guard = self.queue.lock().await;

        let mut exp = guard.queue.split_off(&(at + Duration::from_nanos(1)));
        std::mem::swap(&mut guard.queue, &mut exp);

        for key in exp.values().flatten() {
            guard.scheduled.remove(key);
        }

        exp
    }

    pub async fn drain_expired(&self) -> BTreeMap<Instant, HashSet<K>> {
        loop {
            let Some(earliest) = self.earliest().await else {
                tracing::trace!("waiting for new task to be scheduled");
                self.notify.notified().await;
                tracing::trace!("waked by new task scheduled");
                continue;
            };

            if earliest <= Instant::now() {
                let keys = self.remove_expired_at(Instant::now()).await;
                if keys.is_empty() {
                    tracing::trace!("no expired keys were drained");
                    continue;
                }

                return keys;
            }

            tracing::trace!(
                "waiting for next scheduled task at {:?}",
                earliest.duration_since(Instant::now())
            );
            let earliest = tokio::time::Instant::from_std(earliest);
            tokio::select! {
                () = self.notify.notified() => {
                    tracing::trace!("waked by new task scheduled");
                },
                () = tokio::time::sleep_until(earliest) => {
                    tracing::trace!("waked by task deadline");
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;
    use std::time::Duration;

    use super::*;

    #[tokio::test]
    async fn past_deadline_executed_immediately() {
        let q = Queue::new();
        let when = Instant::now().checked_sub(Duration::from_secs(1)).unwrap();
        q.schedule_at("task", when).await;

        let now = Instant::now();
        let keys = q
            .drain_expired()
            .await
            .into_values()
            .flatten()
            .collect_vec();

        assert!(now.elapsed() < Duration::from_millis(100),);
        assert_eq!(keys, vec!["task"]);
    }

    #[tokio::test]
    async fn single_task_expires_after_deadline() {
        let q = Queue::new();
        let when = Instant::now() + Duration::from_secs(1);
        q.schedule_at("task", when).await;

        let keys = q
            .drain_expired()
            .await
            .into_values()
            .flatten()
            .collect_vec();

        assert!(Instant::now() >= when);
        assert_eq!(keys, vec!["task"]);
    }

    #[tokio::test]
    async fn multiple_tasks_same_instant_returned_together() {
        let q = Queue::new();
        let when = Instant::now();
        for i in 0..3 {
            q.schedule_at(i, when).await;
        }

        let keys = q
            .drain_expired()
            .await
            .into_values()
            .flatten()
            .collect_vec();

        assert!(Instant::now() >= when);
        assert!(keys.contains(&0));
        assert!(keys.contains(&1));
        assert!(keys.contains(&2));
    }

    #[tokio::test]
    async fn multiple_tasks_different_instant_ordering() {
        let q = Queue::new();
        let when = Instant::now();
        for i in 0..3 {
            q.schedule_at(i * 2, when + Duration::from_millis(i * 10))
                .await;
            q.schedule_at(i * 2 + 1, when + Duration::from_millis(i * 10))
                .await;
        }

        tokio::time::sleep(Duration::from_millis(50)).await;

        let mut keys = q.drain_expired().await.into_values().flatten();

        assert!(Instant::now() >= when);
        assert_matches!(keys.next(), Some(0 | 1));
        assert_matches!(keys.next(), Some(0 | 1));
        assert_matches!(keys.next(), Some(2 | 3));
        assert_matches!(keys.next(), Some(2 | 3));
        assert_matches!(keys.next(), Some(4 | 5));
        assert_matches!(keys.next(), Some(4 | 5));
    }

    #[tokio::test]
    async fn tasks_ordered_by_deadline() {
        let q = Queue::new();
        let when = Instant::now();
        q.schedule_at(2, when + Duration::from_millis(200)).await;
        q.schedule_at(1, when + Duration::from_millis(100)).await;

        let first = q
            .drain_expired()
            .await
            .into_values()
            .flatten()
            .collect_vec();

        assert!(when.elapsed() >= Duration::from_millis(100));
        assert_eq!(first, vec![1]);

        let second = q
            .drain_expired()
            .await
            .into_values()
            .flatten()
            .collect_vec();

        assert!(when.elapsed() >= Duration::from_millis(200));
        assert_eq!(second, vec![2]);
    }
}
