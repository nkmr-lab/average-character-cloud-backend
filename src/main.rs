use actix_web::{error, middleware, web, App, HttpResponse, HttpServer};
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::io;

use average_charactor_cloud_backend::app_config::AppConfig;
use average_charactor_cloud_backend::graphql::{create_schema, AppCtx, Schema};
use std::sync::Arc;

async fn graphiql(config: web::Data<AppConfig>) -> HttpResponse {
    let html = graphiql_source(&format!("{}graphql", config.mount_base), None);
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

async fn graphql(
    st: web::Data<Arc<Schema>>,
    pool: web::Data<PgPool>,
    data: web::Json<GraphQLRequest>,
) -> Result<HttpResponse, error::Error> {
    let ctx = AppCtx {
        pool: pool.get_ref().clone(),
    };
    let res = data.execute(&st, &ctx).await;
    let json = serde_json::to_string(&res)?;
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(json))
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    env_logger::init();
    let config = AppConfig::from_env().map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

    let host = config.host.clone();
    let port = config.port;
    let schema = Arc::new(create_schema());
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(schema.clone()))
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(pool.clone()))
            .wrap(middleware::Logger::default())
            .service(web::resource("/graphql").route(web::post().to(graphql)))
            .service(web::resource("/graphiql").route(web::get().to(graphiql)))
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
