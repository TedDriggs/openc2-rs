use futures::{
    StreamExt, future,
    stream::{self, BoxStream},
};

/// Returns a [`BoxStream`] containing a single ready value.
pub fn stream_just<T>(item: T) -> BoxStream<'static, T>
where
    T: Send + 'static,
{
    stream::once(future::ready(item)).boxed()
}
