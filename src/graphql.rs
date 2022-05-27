use juniper::FieldResult;
use juniper::{
    graphql_interface, EmptySubscription, GraphQLInputObject, GraphQLObject, RootNode, ID,
};
use sqlx::PgPool;

pub struct AppCtx {
    pub pool: PgPool,
    pub user_id: Option<String>,
}

impl juniper::Context for AppCtx {}

enum NodeID {
    Record(String),
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

        if let Some(record_id) = s.strip_prefix("record:") {
            Some(NodeID::Record(record_id.to_string()))
        } else {
            None
        }
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
    async fn node(id: ID) -> FieldResult<Option<NodeValue>> {
        Ok(None)
    }

    async fn records(
        character: Option<String>,
        first: Option<i32>,
        after: Option<String>,
        last: Option<i32>,
        before: Option<String>,
    ) -> FieldResult<RecordConnection> {
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
    fn create_record(new_record: NewRecord) -> FieldResult<Record> {
        Ok(Record {
            id: ID::new(""),
            record_id: String::new(),
            character: String::new(),
            figure: String::new(),
            created_at: String::new(),
        })
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
