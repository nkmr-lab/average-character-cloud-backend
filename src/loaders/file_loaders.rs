use std::collections::HashMap;

use crate::entities;
use crate::ports;

use crate::BatchFnWithParams;
use crate::ShareableError;
#[derive(Clone, Debug)]
pub struct FileByIdLoader<A> {
    pub files_repository: A,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FileByIdLoaderParams {
    pub user_id: entities::UserId,
    pub verified_only: bool,
}

impl<A> BatchFnWithParams for FileByIdLoader<A>
where
    A: ports::FilesRepository<Error = anyhow::Error> + Send + Clone,
{
    type K = entities::FileId;
    type V = Result<Option<entities::File>, ShareableError>;
    type P = FileByIdLoaderParams;

    async fn load_with_params(
        &mut self,
        params: &Self::P,
        keys: &[Self::K],
    ) -> HashMap<Self::K, Self::V> {
        let file_map = self
            .files_repository
            .get_by_ids(params.user_id.clone(), keys, params.verified_only)
            .await
            .map(|files| {
                files
                    .into_iter()
                    .map(|file| (file.id, file))
                    .collect::<HashMap<_, _>>()
            })
            .map_err(ShareableError::from);

        keys.iter()
            .map(|key| {
                (
                    *key,
                    file_map
                        .as_ref()
                        .map(|file_map| file_map.get(key).cloned())
                        .map_err(|e| e.clone()),
                )
            })
            .collect()
    }
}
