//! Small channel helpers for Go-style application code.
//!
//! These helpers intentionally stay close to `tokio::sync::mpsc`: the channel is
//! bounded, send is async, receive returns `Option<T>`, and advanced callers can
//! unwrap the Tokio handles when they need the full API.

use tokio::sync::mpsc;

use crate::{Cancellation, Error, Result};

/// Creates a bounded async channel.
pub fn channel<T>(buffer: usize) -> (Sender<T>, Receiver<T>) {
    let (sender, receiver) = mpsc::channel(buffer);
    (Sender { inner: sender }, Receiver { inner: receiver })
}

/// Sending side of an ezrs channel.
#[derive(Debug)]
pub struct Sender<T> {
    inner: mpsc::Sender<T>,
}

impl<T> Sender<T> {
    /// Sends one value, returning an ezrs error if the receiver is closed.
    pub async fn send(&self, value: T) -> Result<()> {
        self.inner
            .send(value)
            .await
            .map_err(|_| Error::msg("channel receiver closed"))
    }

    /// Returns true when the receiving side has been dropped.
    pub fn is_closed(&self) -> bool {
        self.inner.is_closed()
    }

    /// Borrows the underlying Tokio sender.
    pub fn as_tokio(&self) -> &mpsc::Sender<T> {
        &self.inner
    }

    /// Converts into the underlying Tokio sender.
    pub fn into_tokio(self) -> mpsc::Sender<T> {
        self.inner
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

/// Receiving side of an ezrs channel.
#[derive(Debug)]
pub struct Receiver<T> {
    inner: mpsc::Receiver<T>,
}

impl<T> Receiver<T> {
    /// Receives the next value, or `None` when all senders are closed.
    pub async fn recv(&mut self) -> Option<T> {
        self.inner.recv().await
    }

    /// Closes the receiver without dropping it.
    pub fn close(&mut self) {
        self.inner.close();
    }

    /// Borrows the underlying Tokio receiver.
    pub fn as_tokio(&mut self) -> &mut mpsc::Receiver<T> {
        &mut self.inner
    }

    /// Converts into the underlying Tokio receiver.
    pub fn into_tokio(self) -> mpsc::Receiver<T> {
        self.inner
    }
}

/// Result of selecting between two receivers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Select2<A, B> {
    /// The first receiver produced a value.
    Left(A),
    /// The second receiver produced a value.
    Right(B),
    /// Both receivers are closed.
    Closed,
}

/// Receives from whichever of two channels is ready first.
pub async fn select_recv2<A, B>(left: &mut Receiver<A>, right: &mut Receiver<B>) -> Select2<A, B> {
    tokio::select! {
        value = left.recv() => match value {
            Some(value) => Select2::Left(value),
            None => match right.recv().await {
                Some(value) => Select2::Right(value),
                None => Select2::Closed,
            },
        },
        value = right.recv() => match value {
            Some(value) => Select2::Right(value),
            None => match left.recv().await {
                Some(value) => Select2::Left(value),
                None => Select2::Closed,
            },
        },
    }
}

/// Receives one value or reports cancellation.
pub async fn recv_or_cancel<T>(
    receiver: &mut Receiver<T>,
    cancellation: &Cancellation,
) -> Result<Option<T>> {
    tokio::select! {
        value = receiver.recv() => Ok(value),
        _ = cancellation.cancelled() => Err(Error::cancelled("receive cancelled")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn channel_sends_and_receives() {
        let (sender, mut receiver) = channel(2);

        sender.send("hello").await.expect("send");
        drop(sender);

        assert_eq!(receiver.recv().await, Some("hello"));
        assert_eq!(receiver.recv().await, None);
    }

    #[tokio::test]
    async fn select_receives_ready_side() {
        let (_left_tx, mut left_rx) = channel::<u8>(1);
        let (right_tx, mut right_rx) = channel(1);

        right_tx.send(9).await.expect("send");

        assert_eq!(
            select_recv2(&mut left_rx, &mut right_rx).await,
            Select2::Right(9)
        );
    }

    #[tokio::test]
    async fn recv_or_cancel_reports_cancellation() {
        let (_sender, mut receiver) = channel::<u8>(1);
        let cancellation = Cancellation::new();

        cancellation.cancel();

        let error = recv_or_cancel(&mut receiver, &cancellation)
            .await
            .expect_err("cancelled");
        assert_eq!(error.to_string(), "cancelled: receive cancelled");
    }
}
