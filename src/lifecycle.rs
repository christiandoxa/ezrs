//! Lifecycle hooks for Go-style startup, readiness, and shutdown orchestration.

use std::{any::type_name, future::Future, pin::Pin, sync::Arc, time::Duration};

use crate::{Context, Error, Result};

type HookFuture = Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>>;
type HookHandler = Arc<dyn Fn(Context) -> HookFuture + Send + Sync + 'static>;

/// Async lifecycle hook registered with a syntax-first handler.
#[derive(Clone)]
pub struct LifecycleHook {
    name: String,
    handler: HookHandler,
}

impl LifecycleHook {
    /// Creates a hook and derives its diagnostic name from the function item.
    pub fn new<F, Fut>(handler: F) -> Self
    where
        F: Fn(Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        Self::named(hook_name::<F>(), handler)
    }

    /// Creates a hook with an explicit diagnostic name.
    pub fn named<F, Fut>(name: impl Into<String>, handler: F) -> Self
    where
        F: Fn(Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        Self {
            name: name.into(),
            handler: Arc::new(move |ctx| Box::pin(handler(ctx))),
        }
    }

    /// Returns the hook diagnostic name.
    pub fn name(&self) -> &str {
        &self.name
    }

    async fn run(&self, ctx: Context) -> Result<()> {
        (self.handler)(ctx).await
    }
}

/// Ordered lifecycle supervisor for application startup and shutdown phases.
#[derive(Clone)]
pub struct Lifecycle {
    start: Vec<LifecycleHook>,
    ready: Vec<LifecycleHook>,
    shutdown: Vec<LifecycleHook>,
    shutdown_timeout: Duration,
}

impl Lifecycle {
    /// Creates an empty lifecycle with a 30 second shutdown timeout.
    pub fn new() -> Self {
        Self {
            start: Vec::new(),
            ready: Vec::new(),
            shutdown: Vec::new(),
            shutdown_timeout: Duration::from_secs(30),
        }
    }

    /// Registers a startup hook. Hooks run in registration order.
    pub fn on_start<F, Fut>(mut self, handler: F) -> Self
    where
        F: Fn(Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.start.push(LifecycleHook::new(handler));
        self
    }

    /// Registers a readiness hook. Hooks run in registration order.
    pub fn on_ready<F, Fut>(mut self, handler: F) -> Self
    where
        F: Fn(Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.ready.push(LifecycleHook::new(handler));
        self
    }

    /// Registers a shutdown hook. Hooks run in reverse registration order.
    pub fn on_shutdown<F, Fut>(mut self, handler: F) -> Self
    where
        F: Fn(Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.shutdown.push(LifecycleHook::new(handler));
        self
    }

    /// Sets the maximum duration for the whole shutdown phase.
    pub fn shutdown_timeout_secs(mut self, seconds: u64) -> Self {
        self.shutdown_timeout = Duration::from_secs(seconds);
        self
    }

    /// Returns the configured shutdown timeout.
    pub fn shutdown_timeout(&self) -> Duration {
        self.shutdown_timeout
    }

    /// Runs startup hooks in registration order.
    pub async fn run_start(&self, ctx: Context) -> Result<()> {
        run_forward(&self.start, ctx).await
    }

    /// Runs readiness hooks in registration order.
    pub async fn run_ready(&self, ctx: Context) -> Result<()> {
        run_forward(&self.ready, ctx).await
    }

    /// Runs shutdown hooks in reverse registration order within the timeout.
    pub async fn run_shutdown(&self, ctx: Context) -> Result<()> {
        let timeout = self.shutdown_timeout;
        tokio::time::timeout(timeout, async {
            for hook in self.shutdown.iter().rev() {
                hook.run(ctx.clone()).await?;
            }

            Ok(())
        })
        .await
        .map_err(|_| Error::timeout(format!("lifecycle shutdown exceeded {timeout:?}")))?
    }

    /// Returns startup hooks in registration order.
    pub fn start_hooks(&self) -> &[LifecycleHook] {
        &self.start
    }

    /// Returns readiness hooks in registration order.
    pub fn ready_hooks(&self) -> &[LifecycleHook] {
        &self.ready
    }

    /// Returns shutdown hooks in registration order.
    pub fn shutdown_hooks(&self) -> &[LifecycleHook] {
        &self.shutdown
    }
}

impl Default for Lifecycle {
    fn default() -> Self {
        Self::new()
    }
}

async fn run_forward(hooks: &[LifecycleHook], ctx: Context) -> Result<()> {
    for hook in hooks {
        hook.run(ctx.clone()).await?;
    }

    Ok(())
}

fn hook_name<T>() -> String {
    let raw = type_name::<T>();
    let Some(last) = raw.rsplit("::").next() else {
        return String::from("hook");
    };

    if last.contains("{{closure}}") {
        return String::from("hook");
    }

    raw.split("::")
        .filter_map(clean_type_path_part)
        .last()
        .unwrap_or_else(|| String::from("hook"))
}

fn clean_type_path_part(part: &str) -> Option<String> {
    if part.is_empty() || part.contains("{{closure}}") {
        return None;
    }

    let without_generics = part.split('<').next().unwrap_or(part);
    let cleaned = without_generics
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '_' || *ch == '-')
        .collect::<String>();

    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::{Args, state::TypeStore};

    fn context() -> Context {
        Context::runtime(Args::default(), TypeStore::default(), TypeStore::default())
    }

    fn recorder() -> Arc<Mutex<Vec<&'static str>>> {
        Arc::new(Mutex::new(Vec::new()))
    }

    async fn named_start(_ctx: Context) -> Result<()> {
        Ok(())
    }

    #[tokio::test]
    async fn runs_start_and_ready_in_registration_order() {
        let events = recorder();

        let first = events.clone();
        let second = events.clone();
        let ready = events.clone();
        let lifecycle = Lifecycle::new()
            .on_start(move |_ctx| {
                let first = first.clone();
                async move {
                    first.lock().expect("events poisoned").push("start-a");
                    Ok(())
                }
            })
            .on_start(move |_ctx| {
                let second = second.clone();
                async move {
                    second.lock().expect("events poisoned").push("start-b");
                    Ok(())
                }
            })
            .on_ready(move |_ctx| {
                let ready = ready.clone();
                async move {
                    ready.lock().expect("events poisoned").push("ready");
                    Ok(())
                }
            });

        lifecycle.run_start(context()).await.expect("start hooks");
        lifecycle.run_ready(context()).await.expect("ready hooks");

        assert_eq!(
            *events.lock().expect("events poisoned"),
            ["start-a", "start-b", "ready"]
        );
    }

    #[tokio::test]
    async fn runs_shutdown_in_reverse_registration_order() {
        let events = recorder();

        let first = events.clone();
        let second = events.clone();
        let lifecycle = Lifecycle::new()
            .on_shutdown(move |_ctx| {
                let first = first.clone();
                async move {
                    first.lock().expect("events poisoned").push("shutdown-a");
                    Ok(())
                }
            })
            .on_shutdown(move |_ctx| {
                let second = second.clone();
                async move {
                    second.lock().expect("events poisoned").push("shutdown-b");
                    Ok(())
                }
            });

        lifecycle
            .run_shutdown(context())
            .await
            .expect("shutdown hooks");

        assert_eq!(
            *events.lock().expect("events poisoned"),
            ["shutdown-b", "shutdown-a"]
        );
    }

    #[tokio::test]
    async fn returns_first_hook_error() {
        let lifecycle = Lifecycle::new()
            .on_start(|_ctx| async { Err(Error::msg("bad start")) })
            .on_start(|_ctx| async { panic!("must not run after error") });

        let error = lifecycle.run_start(context()).await.expect_err("error");
        assert_eq!(error.to_string(), "bad start");
    }

    #[tokio::test]
    async fn times_out_shutdown_phase() {
        let lifecycle = Lifecycle::new()
            .shutdown_timeout_secs(0)
            .on_shutdown(|_ctx| async {
                tokio::time::sleep(Duration::from_millis(10)).await;
                Ok(())
            });

        let error = lifecycle
            .run_shutdown(context())
            .await
            .expect_err("timeout");
        assert!(error.to_string().contains("lifecycle shutdown exceeded"));
    }

    #[test]
    fn derives_hook_name_from_function_item() {
        let hook = LifecycleHook::new(named_start);
        assert_eq!(hook.name(), "named_start");
    }
}
