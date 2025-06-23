use anyhow::Context;

use crate::entities;
use crate::ports;
use crate::BatchFnWithParams;
use crate::ShareableError;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct GenerateTemplateByIdLoader<A> {
    pub generate_templates_repository: A,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GenerateTemplateByIdLoaderParams {
    pub user_id: entities::UserId,
}

impl<A> BatchFnWithParams for GenerateTemplateByIdLoader<A>
where
    A: ports::GenerateTemplatesRepository<Error = anyhow::Error> + Send + Clone,
{
    type K = entities::GenerateTemplateId;
    type V = Result<Option<entities::GenerateTemplate>, ShareableError>;
    type P = GenerateTemplateByIdLoaderParams;

    async fn load_with_params(
        &mut self,
        params: &Self::P,
        keys: &[Self::K],
    ) -> HashMap<Self::K, Self::V> {
        let generate_template_map = self
            .generate_templates_repository
            .get_by_ids(params.user_id.clone(), keys)
            .await
            .map(|generate_templates| {
                generate_templates
                    .into_iter()
                    .map(|generate_template| (generate_template.id, generate_template))
                    .collect::<HashMap<_, _>>()
            })
            .map_err(ShareableError::from);

        keys.iter()
            .map(|key| {
                (
                    *key,
                    generate_template_map
                        .as_ref()
                        .map(|generate_template_map| generate_template_map.get(key).cloned())
                        .map_err(|e| e.clone()),
                )
            })
            .collect()
    }
}

#[derive(Clone, Debug)]
pub struct GenerateTemplatesLoader<A> {
    pub generate_templates_repository: A,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GenerateTemplatesLoaderParams {
    pub user_id: entities::UserId,
    pub after_id: Option<entities::GenerateTemplateId>,
    pub before_id: Option<entities::GenerateTemplateId>,
    pub limit: entities::Limit,
}

impl<A> BatchFnWithParams for GenerateTemplatesLoader<A>
where
    A: ports::GenerateTemplatesRepository<Error = anyhow::Error> + Send + Clone,
{
    type K = ();
    type V = Result<ports::PaginationResult<entities::GenerateTemplate>, ShareableError>;
    type P = GenerateTemplatesLoaderParams;

    async fn load_with_params(
        &mut self,
        params: &Self::P,
        _: &[Self::K],
    ) -> HashMap<Self::K, Self::V> {
        let result = self
            .generate_templates_repository
            .query(
                params.user_id.clone(),
                params.after_id.clone(),
                params.before_id.clone(),
                params.limit.increment_unchecked(),
            )
            .await
            .and_then(|mut generate_templates| {
                let has_next = generate_templates.len()
                    > usize::try_from(params.limit.value()).context("into usize")?;
                generate_templates
                    .truncate(usize::try_from(params.limit.value()).context("into usize")?);
                if params.limit.kind() == entities::LimitKind::Last {
                    generate_templates.reverse();
                }
                Ok(ports::PaginationResult {
                    values: generate_templates,
                    has_next,
                })
            })
            .map_err(ShareableError::from);
        vec![((), result)].into_iter().collect()
    }
}
