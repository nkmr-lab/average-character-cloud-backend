#![cfg_attr(not(debug_assertions), deny(warnings))]

use actix_session::storage::RedisSessionStore;
use actix_session::{Session, SessionLength, SessionMiddleware};
use actix_web::cookie::Key;
use actix_web::{error, get, middleware, post, web, App, HttpRequest, HttpResponse, HttpServer};
use anyhow::Context;
use average_character_cloud_backend::faktory::FaktoryConnectionManager;
use average_character_cloud_backend::google_public_key_provider::{
    GooglePublicKeyProvider, GooglePublicKeyProviderCommand,
};
use chrono::Utc;
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::collections::HashMap;
use std::io;
use std::str::FromStr;
use time::Duration;
use tracing_actix_web::TracingLogger;

use actix_web_extras::middleware::Condition as OptionalCondition;
use average_character_cloud_backend::app_config::{AppConfig, AuthConfig, SessionConfig};
use average_character_cloud_backend::graphql::{create_schema, AppCtx, Loaders, Schema};
use average_character_cloud_backend::job::Job;
use average_character_cloud_backend::{entities, job, jobs};
use clap::{Parser, Subcommand};
use jsonwebtoken::jwk::{self, JwkSet};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
#[derive(Parser)]
#[clap(name = "average-character-cloud-backend")]
struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Migrate,
    MigrateStorage,
}

#[get("/graphiql")]
async fn graphiql(config: web::Data<AppConfig>) -> HttpResponse {
    let html = graphiql_source(
        &format!("/{}", {
            let mut path = config.mount_base.clone();
            path.push("graphql".to_string());
            path.join("/")
        }),
        None,
    );
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

#[post("/graphql")]
async fn graphql(
    st: web::Data<Arc<Schema>>,
    pool: web::Data<PgPool>,
    s3_client: web::Data<aws_sdk_s3::Client>,
    data: web::Json<GraphQLRequest>,
    session: Session,
    config: web::Data<AppConfig>,
) -> Result<HttpResponse, error::Error> {
    let ctx = AppCtx {
        pool: pool.get_ref().clone(),
        user_id: if let SessionConfig::Dummy { user_id } = &config.session {
            Some(entities::UserId::from(user_id.clone()))
        } else {
            session
                .get::<String>("user_id")
                .unwrap_or_else(|e| {
                    tracing::warn!("session decode error: : {}", e);
                    None
                })
                .map(entities::UserId::from)
        },
        now: Utc::now(),
        loaders: Loaders::new(pool.get_ref()),
        config: config.get_ref().clone(),
        s3_client: s3_client.get_ref().clone(),
    };
    let res = data.execute(&st, &ctx).await;
    let json = serde_json::to_string(&res)?;
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(json))
}

#[derive(Serialize, Deserialize)]
struct GoogleCallbackParams {
    g_csrf_token: String,
    credential: String,
}

#[get("/google_login")]
async fn google_login_front(config: web::Data<AppConfig>) -> HttpResponse {
    let AuthConfig::Google {
        client_id,
        enable_front,
        ..
    } = &config.auth
    else {
        return HttpResponse::NotFound().body("Not found");
    };

    if !enable_front {
        return HttpResponse::NotFound().body("Not found");
    }

    let content = format!(
        r#"
    <!DOCTYPE html>
    <html lang="en">

    <head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Google Login Front</title>
    </head>

    <body>
    <script src="https://accounts.google.com/gsi/client" async defer></script>
    <div id="g_id_onload" data-client_id="{}"
        data-login_uri="{}/{}" data-auto_prompt="false">
    </div>
    <div class="g_id_signin" data-type="standard" data-size="large" data-theme="outline" data-text="sign_in_with"
        data-shape="rectangular" data-logo_alignment="left">
    </div>
    </body>

    </html>
    "#,
        client_id,
        config.origin,
        {
            let mut path = config.mount_base.clone();
            path.push("google_login_callback".to_string());
            path.join("/")
        }
    );
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(content)
}

#[get("/run_task")]
async fn run_task_front(
    faktory_pool: web::Data<r2d2::Pool<FaktoryConnectionManager>>,
) -> Result<HttpResponse, error::Error> {
    async {
        (jobs::UpdateSeeds {}).enqueue(&faktory_pool).await?;
        Ok(HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body("ok"))
    }
    .await
    .map_err(|e: anyhow::Error| {
        tracing::error!("run_task_front error: {}", e);
        error::ErrorInternalServerError(e)
    })
}

fn verify_google_token(
    jwks: JwkSet,
    token: &str,
    client_id: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let header = jsonwebtoken::decode_header(token)?;

    let kid = match header.kid {
        Some(k) => k,
        None => return Err("Token doesn't have a `kid` header field".into()),
    };
    if let Some(j) = jwks.find(&kid) {
        match j.algorithm {
            jwk::AlgorithmParameters::RSA(ref rsa) => {
                let decoding_key = jsonwebtoken::DecodingKey::from_rsa_components(&rsa.n, &rsa.e)?;
                let mut validation = jsonwebtoken::Validation::new(
                    j.common
                        .algorithm
                        .ok_or_else(|| io::Error::other("not found"))?,
                );
                validation.set_audience(&[client_id]);
                validation.set_issuer(&["accounts.google.com", "https://accounts.google.com"]);
                let decoded_token = jsonwebtoken::decode::<HashMap<String, serde_json::Value>>(
                    token,
                    &decoding_key,
                    &validation,
                )?;
                Ok(decoded_token.claims["sub"]
                    .as_str()
                    .ok_or("sub not found")?
                    .to_string())
            }
            _ => Err("this should be a RSA".into()),
        }
    } else {
        Err("No matching JWK found for the given kid".into())
    }
}

#[post("/logout")]
async fn logout(
    config: web::Data<AppConfig>,
    session: Session,
) -> Result<HttpResponse, error::Error> {
    session.clear();
    Ok(HttpResponse::SeeOther()
        .append_header((
            actix_web::http::header::LOCATION,
            config.logout_redirect_url.to_string(),
        ))
        .finish())
}

#[post("/google_login_callback")]
async fn google_callback(
    config: web::Data<AppConfig>,
    req: HttpRequest,
    params: web::Form<GoogleCallbackParams>,
    google_public_key_provider: web::Data<mpsc::Sender<GooglePublicKeyProviderCommand>>,
    session: Session,
) -> Result<HttpResponse, error::Error> {
    let AuthConfig::Google {
        client_id,
        redirect_url,
        ..
    } = &config.auth
    else {
        return Err(error::ErrorBadRequest("Invalid auth kind"));
    };

    let (jwks_tx, jwks_rx) = oneshot::channel();
    google_public_key_provider
        .send(GooglePublicKeyProviderCommand::Get { resp: jwks_tx })
        .await
        .unwrap();
    let jwks = match jwks_rx.await.unwrap() {
        Ok(jwks) => jwks,
        Err(e) => {
            tracing::error!("get google public key error: {}", e);
            return Ok(HttpResponse::InternalServerError().finish());
        }
    };

    let csrf_token_cookie = req
        .cookie("g_csrf_token")
        .map(|c| c.value().to_string())
        .ok_or_else(|| error::ErrorBadRequest("No CSRF token in Cookie.".to_string()))?;
    let csrf_token_body = params.g_csrf_token.as_str();
    if csrf_token_cookie != csrf_token_body {
        return Err(error::ErrorBadRequest(
            "Failed to verify double submit cookie.".to_string(),
        ));
    }

    let credential = params.credential.as_str();
    let token = verify_google_token(jwks, credential, client_id).map_err(|e| {
        tracing::info!("verify google token error: {}", e);
        error::ErrorBadRequest(format!("Failed to verify token: {}", e))
    })?;
    session.insert("user_id", token)?;
    Ok(HttpResponse::SeeOther()
        .append_header((actix_web::http::header::LOCATION, redirect_url.to_string()))
        .finish())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    let config = AppConfig::from_env().map_err(io::Error::other)?;

    let aws_config = aws_config::from_env().load().await;
    let mut s3_config_builder = aws_sdk_s3::config::Builder::from(&aws_config);
    if config.storage.path_style {
        s3_config_builder = s3_config_builder.force_path_style(true);
    }
    let s3_client = aws_sdk_s3::Client::from_conf(s3_config_builder.build());

    let pool = PgPoolOptions::new()
        .connect(&config.database_url)
        .await
        .context("Failed to connect to database")?;
    let faktory_pool = r2d2::Pool::builder()
        .build(FaktoryConnectionManager::new(config.faktory_url.clone()))
        .context("Failed to connect to faktory")?;

    match cli.command {
        Some(Commands::Migrate) => sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .context("migrate"),
        Some(Commands::MigrateStorage) => {
            let bucket_exists = s3_client
                .head_bucket()
                .bucket(config.storage.bucket.clone())
                .send()
                .await
                .is_ok();
            if bucket_exists {
                tracing::info!("S3 bucket already exists: {}", config.storage.bucket);
            } else {
                s3_client
                    .create_bucket()
                    .bucket(config.storage.bucket.clone())
                    .send()
                    .await
                    .context("Failed to create S3 bucket")?;
            }
            Ok(())
        }
        None => {
            let host = config.host.clone();
            let port = config.port;
            let workers = config.workers;
            let schema = Arc::new(create_schema());

            let redis_session_config = if let SessionConfig::Redis {
                redis_url,
                crypto_key,
            } = &config.session
            {
                let secret_key = Key::from(crypto_key.as_slice());
                let store = RedisSessionStore::new(redis_url.clone()).await?;

                Some((secret_key, store))
            } else {
                None
            };

            let (google_public_key_provider_tx, google_public_key_provider_rx) = mpsc::channel(100);
            tokio::spawn(async move {
                GooglePublicKeyProvider::run(google_public_key_provider_rx).await;
            });

            job::run_worker(&config.faktory_url, job::Ctx { pool: pool.clone() })?;

            if config.enqueue_cron_task {
                let faktory_pool = faktory_pool.clone();

                tokio::spawn(async move {
                    for datetime in cron::Schedule::from_str("0 0 1/6 * * * *")
                        .unwrap()
                        .upcoming(Utc)
                    {
                        let now = Utc::now();
                        let delay = datetime - now;
                        if let Ok(delay) = delay.to_std() {
                            tokio::time::sleep(delay).await;
                        } else {
                            tracing::warn!("delay is negative");
                            continue;
                        }

                        if let Err(e) = (jobs::UpdateSeeds {}).enqueue(&faktory_pool).await {
                            tracing::error!("enqueue update_seeds error: {}", e);
                        }
                    }
                });
            }

            HttpServer::new(move || {
                let mut app = App::new()
                    .wrap(TracingLogger::default())
                    .app_data(web::Data::new(schema.clone()))
                    .app_data(web::Data::new(config.clone()))
                    .app_data(web::Data::new(pool.clone()))
                    .app_data(web::Data::new(s3_client.clone()))
                    .app_data(web::Data::new(faktory_pool.clone()))
                    .service(graphql)
                    .service(graphiql)
                    .service(logout);
                if config.enable_task_front {
                    app = app.service(run_task_front);
                }

                if let AuthConfig::Google { enable_front, .. } = &config.auth {
                    app = app
                        .app_data(web::Data::new(google_public_key_provider_tx.clone()))
                        .service(google_callback);

                    if *enable_front {
                        app = app.service(google_login_front);
                    }
                }

                app.wrap(middleware::Logger::default())
                    .wrap(OptionalCondition::from_option(
                        redis_session_config.clone().map(|(secret_key, store)| {
                            SessionMiddleware::builder(store, secret_key)
                                .cookie_path(format!("/{}", config.mount_base.join("/")))
                                .session_length(SessionLength::Predetermined {
                                    max_session_length: Some(Duration::days(1)),
                                })
                                .cookie_name("average-character-cloud-session".to_string())
                                .build()
                        }),
                    ))
            })
            .workers(workers)
            .bind((host.as_str(), port))?
            .run()
            .await
            .context("Failed to start server")
        }
    }
}
