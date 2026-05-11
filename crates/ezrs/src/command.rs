use std::{future::Future, pin::Pin, sync::Arc};

use crate::{Context, Result};

pub(crate) type CommandFuture = Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>>;
pub(crate) type CommandHandler = Arc<dyn Fn(Context) -> CommandFuture + Send + Sync + 'static>;

#[derive(Clone)]
pub(crate) struct Command {
    pub(crate) name: String,
    pub(crate) handler: CommandHandler,
}

impl Command {
    pub(crate) fn new<F, Fut>(name: impl Into<String>, handler: F) -> Self
    where
        F: Fn(Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        Self {
            name: name.into(),
            handler: Arc::new(move |ctx| Box::pin(handler(ctx))),
        }
    }
}
