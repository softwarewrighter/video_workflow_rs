//! GPU-bound task queue with semaphore-based concurrency control.
//!
//! Ensures serialized access to GPU resources that cannot handle parallel requests.

use std::future::Future;
use std::sync::Arc;
use tokio::sync::Semaphore;

/// Queue for GPU-bound tasks with configurable concurrency limits.
#[derive(Clone)]
pub struct GpuQueue {
    tts: Arc<Semaphore>,
    lipsync: Arc<Semaphore>,
}

impl GpuQueue {
    /// Create a new queue with specified capacities.
    pub fn new(tts_capacity: usize, lipsync_capacity: usize) -> Self {
        Self {
            tts: Arc::new(Semaphore::new(tts_capacity)),
            lipsync: Arc::new(Semaphore::new(lipsync_capacity)),
        }
    }

    /// Create queue with default capacities (TTS=1, Lipsync=2).
    pub fn default_capacities() -> Self {
        Self::new(1, 2)
    }

    /// Run a TTS task, waiting for queue slot.
    pub async fn run_tts<F, T>(&self, task: F) -> T
    where
        F: Future<Output = T>,
    {
        let _permit = self.tts.acquire().await.expect("semaphore closed");
        task.await
    }

    /// Run a lipsync task, waiting for queue slot.
    pub async fn run_lipsync<F, T>(&self, task: F) -> T
    where
        F: Future<Output = T>,
    {
        let _permit = self.lipsync.acquire().await.expect("semaphore closed");
        task.await
    }
}

impl Default for GpuQueue {
    fn default() -> Self {
        Self::default_capacities()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    #[tokio::test]
    async fn tts_serializes_requests() {
        let queue = GpuQueue::new(1, 2);
        let counter = Arc::new(AtomicUsize::new(0));
        let max_concurrent = Arc::new(AtomicUsize::new(0));

        let mut handles = vec![];
        for _ in 0..5 {
            let q = queue.clone();
            let c = counter.clone();
            let m = max_concurrent.clone();
            handles.push(tokio::spawn(async move {
                q.run_tts(async {
                    let current = c.fetch_add(1, Ordering::SeqCst) + 1;
                    m.fetch_max(current, Ordering::SeqCst);
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    c.fetch_sub(1, Ordering::SeqCst);
                })
                .await;
            }));
        }
        for h in handles {
            h.await.unwrap();
        }
        assert_eq!(max_concurrent.load(Ordering::SeqCst), 1);
    }
}
