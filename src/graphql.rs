use std::str::FromStr;

use chrono::{DateTime, Utc};
use juniper::FieldError;
use juniper::{
    graphql_interface, EmptySubscription, GraphQLInputObject, GraphQLObject, IntoFieldError,
    RootNode, ScalarValue, ID,
};
use sqlx::PgPool;
use ulid::Ulid;

use crate::entities;

#[derive(Debug)]
pub struct AppCtx {
    pub pool: PgPool,
    pub user_id: Option<String>,
    pub now: DateTime<Utc>,
}

#[derive(Debug)]
pub enum AppError {
    Other(String),
    Internal(Box<dyn std::error::Error + Send + Sync>),
}

impl<S: ScalarValue> IntoFieldError<S> for AppError {
    fn into_field_error(self) -> FieldError<S> {
        match self {
            AppError::Other(msg) => msg.into(),
            AppError::Internal(err) => {
                log::error!("{}", err);
                "Internal error".into()
            }
        }
    }
}

type AppResult<T> = Result<T, AppError>;

enum Limit {
    First(i32),
    Last(i32),
}

impl Limit {
    fn encode(first: Option<i32>, last: Option<i32>) -> AppResult<Self> {
        let max = 100;

        match (first, last) {
            (Some(first), None) => {
                if first < 0 {
                    Err(AppError::Other(
                        "first must be greater than or equal to 0".to_string(),
                    ))
                } else if first > max {
                    Err(AppError::Other(format!(
                        "first must be less than or equal to {}",
                        max
                    )))
                } else {
                    Ok(Limit::First(first))
                }
            }
            (None, Some(last)) => {
                if last < 0 {
                    Err(AppError::Other(
                        "last must be greater than or equal to 0".to_string(),
                    ))
                } else if last > max {
                    Err(AppError::Other(format!(
                        "last must be less than or equal to {}",
                        max
                    )))
                } else {
                    Ok(Limit::Last(last))
                }
            }
            _ => Err(AppError::Other(
                "Must provide either first or last, not both".to_string(),
            )),
        }
    }
}

impl juniper::Context for AppCtx {}

enum NodeID {
    Record(Ulid),
}

impl NodeID {
    fn to_id(&self) -> ID {
        match self {
            NodeID::Record(id) => ID::new(base64::encode(format!("record:{}", id))),
        }
    }

    fn from_id(id: &ID) -> Option<NodeID> {
        let id = id.to_string();
        let buf = base64::decode(id).ok()?;
        let s = String::from_utf8(buf).ok()?;

        let Some(record_id) = s.strip_prefix("record:") else {
            return None;
        };

        let ulid = Ulid::from_str(record_id).ok()?;
        Some(NodeID::Record(ulid))
    }
}

/**
 * replayの仕様に従うこと
 *   https://relay.dev/docs/guides/graphql-server-specification/
 *   https://relay.dev/graphql/connections.htm
*/

#[graphql_interface(for = [Record])]
trait Node {
    fn id(&self) -> &ID;
}

#[derive(GraphQLObject, Clone, Debug)]
struct PageInfo {
    has_next_page: bool,
    has_previous_page: bool,
    start_cursor: Option<String>,
    end_cursor: Option<String>,
}

#[derive(GraphQLObject, Clone, Debug)]
#[graphql(impl = NodeValue)]
struct Record {
    id: ID,
    record_id: String,
    character: String,
    figure: String,
    created_at: String,
}

impl Record {
    fn from_entity(record: &entities::record::Record) -> Record {
        Record {
            id: NodeID::Record(record.id).to_id(),
            record_id: record.id.to_string(),
            character: record.character.to_string(),
            figure: record.figure.to_json(),
            created_at: record.created_at.to_rfc3339(),
        }
    }
}

#[graphql_interface]
impl Node for Record {
    fn id(&self) -> &ID {
        &self.id
    }
}

#[derive(GraphQLObject, Clone, Debug)]
struct RecordEdge {
    cursor: String,
    node: Record,
}

#[derive(GraphQLObject, Clone, Debug)]
struct RecordConnection {
    page_info: PageInfo,
    edges: Vec<RecordEdge>,
}

#[derive(GraphQLInputObject, Clone, Debug)]
struct NewRecord {
    character: String,
    figure: String,
}

#[derive(Clone, Debug)]
pub struct QueryRoot;

#[juniper::graphql_object(Context = AppCtx)]
impl QueryRoot {
    async fn node(id: ID) -> AppResult<Option<NodeValue>> {
        Ok(None)
    }

    async fn records(
        ctx: &AppCtx,
        character: Option<String>,
        first: Option<i32>,
        after: Option<String>,
        last: Option<i32>,
        before: Option<String>,
    ) -> AppResult<RecordConnection> {
        Ok(RecordConnection {
            page_info: PageInfo {
                has_next_page: false,
                has_previous_page: false,
                start_cursor: None,
                end_cursor: None,
            },
            edges: vec![],
        })
    }
}
#[derive(Clone, Debug)]
pub struct MutationRoot;

#[juniper::graphql_object(Context = AppCtx)]
impl MutationRoot {
    async fn create_record(ctx: &AppCtx, new_record: NewRecord) -> AppResult<Record> {
        let Some(user_id) = ctx.user_id.clone() else {
            return Err(AppError::Other("Authentication required".to_string()));
        } ;

        let &[character] = new_record.character.chars().collect::<Vec<_>>().as_slice() else {
            return Err(AppError::Other(
                "character must be one character".to_string(),
            ));
        };

        let Some(figure) = entities::figure::Figure::from_json(&new_record.figure) else {
            return Err(AppError::Other("figure must be valid json".to_string()));
        };

        let record = entities::record::Record {
            id: Ulid::from_datetime(ctx.now.clone()),
            user_id,
            character,
            figure,
            created_at: ctx.now.clone(),
        };

        sqlx::query!(
            "\
            INSERT INTO records (id, user_id, character, figure, created_at) \
            VALUES ($1, $2, $3, $4, $5)\
            ",
            record.id.to_string(),
            record.user_id,
            record.character.to_string(),
            record.figure.to_json_ast(),
            record.created_at,
        )
        .execute(&ctx.pool)
        .await
        .map_err(|err| AppError::Internal(Box::new(err)))?;

        Ok(Record::from_entity(&record))
    }
}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot, EmptySubscription<AppCtx>>;

pub fn create_schema() -> Schema {
    Schema::new(
        QueryRoot {},
        MutationRoot {},
        EmptySubscription::<AppCtx>::new(),
    )
}
