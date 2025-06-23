use crate::entities;
use chrono::{DateTime, Utc};

pub trait GenerateTemplatesRepository {
    type Error;

    async fn create(
        &mut self,
        generate_template: entities::GenerateTemplate,
    ) -> Result<entities::GenerateTemplate, Self::Error>;

    async fn get_by_ids(
        &mut self,
        user_id: entities::UserId,
        ids: &[entities::GenerateTemplateId],
    ) -> Result<Vec<entities::GenerateTemplate>, Self::Error>;

    async fn update(
        &mut self,
        now: DateTime<Utc>,
        generate_template: entities::GenerateTemplate,
    ) -> Result<entities::GenerateTemplate, Self::Error>;

    async fn query(
        &mut self,
        user_id: entities::UserId,
        after_id: Option<entities::GenerateTemplateId>,
        before_id: Option<entities::GenerateTemplateId>,
        limit: entities::Limit,
    ) -> Result<Vec<entities::GenerateTemplate>, Self::Error>;
}
