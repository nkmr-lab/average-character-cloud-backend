use async_trait::async_trait;
use dataloader::cached::Loader;
use dataloader::BatchFn;
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::marker::Send;

#[async_trait]
pub trait BatchFnWithParams<K, V, P> {
    async fn load_with_params(&mut self, params: &P, keys: &[K]) -> HashMap<K, V>
    where
        K: 'async_trait,
        V: 'async_trait,
        P: 'async_trait;
}

#[derive(Debug, Clone)]
pub struct DataloaderWithParamsBatchFn<P, F> {
    params: P,
    f: F,
}

#[async_trait]
impl<K: Send + Sync, V: Send, P: Send + Sync, F: BatchFnWithParams<K, V, P> + Send> BatchFn<K, V>
    for DataloaderWithParamsBatchFn<P, F>
{
    async fn load(&mut self, keys: &[K]) -> HashMap<K, V>
    where
        K: 'async_trait,
        V: 'async_trait,
        P: 'async_trait,
    {
        self.f.load_with_params(&self.params, keys).await
    }
}

#[derive(Debug, Clone)]
pub struct DataloaderWithParams<F, A, B>(F, HashMap<A, B>);

impl<
        K: Eq + Hash + Clone + fmt::Debug + Send + Sync,
        V: Clone + Send,
        P: Eq + Hash + Clone + Send + Sync,
        F: BatchFnWithParams<K, V, P> + Clone + Send,
    > DataloaderWithParams<F, P, Loader<K, V, DataloaderWithParamsBatchFn<P, F>>>
{
    pub async fn load(&mut self, params: P, key: K) -> V {
        self.1
            .entry(params.clone())
            .or_insert_with(|| {
                Loader::new(DataloaderWithParamsBatchFn {
                    params,
                    f: self.0.clone(),
                })
            })
            .load(key)
            .await
    }
}
