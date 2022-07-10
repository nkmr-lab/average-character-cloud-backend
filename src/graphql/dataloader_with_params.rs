use async_trait::async_trait;
use dataloader::cached::Loader;
use dataloader::BatchFn;
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::io;
use std::marker::Send;
use std::sync::Arc;
use tokio::sync::Mutex;

#[async_trait]
pub trait BatchFnWithParams: Clone + Send {
    type K: Eq + Hash + Clone + fmt::Debug + Send + Sync;
    type V: Clone + Send;
    type P: Eq + Hash + Clone + Send + Sync;
    async fn load_with_params(
        &mut self,
        params: &Self::P,
        keys: &[Self::K],
    ) -> HashMap<Self::K, Self::V>;
}

#[derive(Debug, Clone)]
pub struct DataloaderWithParamsBatchFn<F: BatchFnWithParams> {
    params: F::P,
    f: F,
}

#[async_trait]
impl<F: BatchFnWithParams> BatchFn<F::K, F::V> for DataloaderWithParamsBatchFn<F> {
    async fn load(&mut self, keys: &[F::K]) -> HashMap<F::K, F::V> {
        self.f.load_with_params(&self.params, keys).await
    }
}

#[derive(Clone)]
pub struct DataloaderWithParams<F: BatchFnWithParams>(
    F,
    Arc<Mutex<HashMap<F::P, Loader<F::K, F::V, DataloaderWithParamsBatchFn<F>>>>>,
);

impl<F: BatchFnWithParams> DataloaderWithParams<F> {
    pub fn new(f: F) -> Self {
        Self(f, Arc::new(Mutex::new(HashMap::new())))
    }

    pub async fn load(&self, params: F::P, key: F::K) -> io::Result<F::V> {
        let mut map = self.1.lock().await;
        let loader = map
            .entry(params.clone())
            .or_insert_with(|| {
                Loader::new(DataloaderWithParamsBatchFn {
                    params,
                    f: self.0.clone(),
                })
                .with_yield_count(100)
            })
            .clone();
        drop(map);
        loader.try_load(key).await
    }
}
