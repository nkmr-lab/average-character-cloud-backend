use actix_web::{error, middleware, web, App, HttpResponse, HttpServer};
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;
use std::io;

mod app_config;
mod graphql;

use crate::graphql::{create_schema, Schema};
use app_config::AppConfig;
use std::sync::Arc;

fn graphiql(config: web::Data<AppConfig>) -> HttpResponse {
    let html = graphiql_source(&format!("{}graphql", config.mount_base), None);
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

async fn graphql(
    st: web::Data<Arc<Schema>>,
    data: web::Json<GraphQLRequest>,
) -> Result<HttpResponse, error::Error> {
    let res = data.execute(&st, &()).await;
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

    HttpServer::new(move || {
        App::new()
            .data(schema.clone())
            .data(config.clone())
            .wrap(middleware::Logger::default())
            .service(web::resource("/graphql").route(web::post().to(graphql)))
            .service(web::resource("/graphiql").route(web::get().to(graphiql)))
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
