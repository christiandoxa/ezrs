//! First-class worker pool for bounded fan-out/fan-in jobs.

use std::{future::Future, marker::PhantomData, sync::Arc};

use tokio::sync::{Mutex, mpsc};

use crate::{Error, Result, TaskGroup};

/// Builds and runs a worker pool around a Rust worker function or closure.
///
/// Prefer `WorkerPool::new(process_job)` when a function item can carry the
/// worker identity in code. Use closures when a worker needs cloned state.
#[derive(Debug)]
pub struct WorkerPool<T, W> {
    worker: W,
    workers: usize,
    buffer: usize,
    _job: PhantomData<fn(T)>,
}

impl<T, W> WorkerPool<T, W> {
    /// Creates a worker pool using a worker function or closure.
    pub fn new(worker: W) -> Self {
        Self {
            worker,
            workers: 1,
            buffer: 64,
            _job: PhantomData,
        }
    }

    /// Sets the number of worker tasks.
    pub fn workers(mut self, workers: usize) -> Self {
        self.workers = workers.max(1);
        self
    }

    /// Sets the bounded jobs channel size.
    pub fn buffer(mut self, buffer: usize) -> Self {
        self.buffer = buffer.max(1);
        self
    }
}

impl<T, W, Fut> WorkerPool<T, W>
where
    T: Send + 'static,
    W: Fn(T) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<()>> + Send + 'static,
{
    /// Runs all jobs and waits for workers to finish.
    pub async fn run<I>(self, jobs: I) -> Result<()>
    where
        I: IntoIterator<Item = T>,
    {
        let (sender, receiver) = mpsc::channel::<T>(self.buffer);
        let receiver = Arc::new(Mutex::new(receiver));
        let group = TaskGroup::new().cancel_on_error(true);

        for _ in 0..self.workers {
            let receiver = Arc::clone(&receiver);
            let worker = self.worker.clone();
            group.spawn(async move {
                loop {
                    let job = receiver.lock().await.recv().await;
                    let Some(job) = job else { break };
                    worker(job).await?;
                }
                Ok(())
            });
        }

        for job in jobs {
            if group.cancellation().is_cancelled() {
                break;
            }
            sender
                .send(job)
                .await
                .map_err(|_| Error::msg("worker pool stopped before accepting all jobs"))?;
        }
        drop(sender);

        group.join().await
    }
}

/// Creates a worker pool using a worker function or closure.
pub fn worker_pool<T, W>(worker: W) -> WorkerPool<T, W> {
    WorkerPool::new(worker)
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use super::*;

    #[tokio::test]
    async fn worker_pool_runs_all_jobs() {
        let count = Arc::new(AtomicUsize::new(0));
        let worker_count = Arc::clone(&count);

        WorkerPool::new(move |_job: u8| {
            let worker_count = Arc::clone(&worker_count);
            async move {
                worker_count.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        })
        .workers(2)
        .buffer(2)
        .run(0..5)
        .await
        .expect("run");

        assert_eq!(count.load(Ordering::SeqCst), 5);
    }

    #[tokio::test]
    async fn worker_pool_returns_worker_error() {
        let error = WorkerPool::new(|job: u8| async move {
            if job == 2 {
                Err(Error::msg("bad job"))
            } else {
                Ok(())
            }
        })
        .workers(2)
        .run(0..4)
        .await
        .expect_err("error");

        assert_eq!(error.to_string(), "bad job");
    }
}
