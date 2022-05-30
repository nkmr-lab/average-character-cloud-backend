#![feature(let_else, try_blocks, let_chains, try_trait_v2)]

use actix_session::storage::RedisActorSessionStore;
use actix_session::{Session, SessionMiddleware};
use actix_web::cookie::Key;
use actix_web::{error, middleware, web, App, HttpRequest, HttpResponse, HttpServer};
use arc_swap::ArcSwap;
use chrono::{DateTime, FixedOffset, Utc};
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;
use reqwest::header::EXPIRES;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::collections::HashMap;
use std::io;

use average_charactor_cloud_backend::app_config::{AppConfig, AuthConfig};
use average_charactor_cloud_backend::graphql::{create_schema, AppCtx, Schema};
use jsonwebtoken::jwk::{self, JwkSet};
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
    session: Session,
) -> Result<HttpResponse, error::Error> {
    let ctx = AppCtx {
        pool: pool.get_ref().clone(),
        user_id: session.get::<String>("user_id").unwrap_or_else(|e| {
            log::warn!("session decode error: : {}", e);
            None
        }),
        now: Utc::now(),
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

#[derive(Debug, Clone)]
struct GooglePublicKey {
    jwks: JwkSet,
    expires: DateTime<FixedOffset>,
}

async fn fetch_google_public_key(
) -> Result<GooglePublicKey, Box<dyn std::error::Error + Send + Sync>> {
    let res = reqwest::get("https://www.googleapis.com/oauth2/v3/certs").await?;
    let expires = DateTime::parse_from_rfc2822(
        res.headers()
            .get(EXPIRES)
            .ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "no expires header")
            })?
            .to_str()?,
    )?;
    let jwks = serde_json::from_str::<JwkSet>(res.text().await?.as_str())?;

    Ok(GooglePublicKey {
        jwks: jwks.clone(),
        expires,
    })
}

async fn google_login_front(config: web::Data<AppConfig>) -> HttpResponse {
    let AuthConfig::Google { client_id, enable_front } = &config.auth else {
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
        data-login_uri="{}{}/google_login_callback" data-auto_prompt="false">
    </div>
    <div class="g_id_signin" data-type="standard" data-size="large" data-theme="outline" data-text="sign_in_with"
        data-shape="rectangular" data-logo_alignment="left">
    </div>
    </body>

    </html>
    "#,
        client_id, config.origin, config.mount_base
    );
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(content)
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
                        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "not found"))?,
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
                    .ok_or_else(|| "sub not found")?
                    .to_string())
            }
            _ => return Err("this should be a RSA".into()),
        }
    } else {
        return Err("No matching JWK found for the given kid".into());
    }
}

async fn google_callback(
    config: web::Data<AppConfig>,
    req: HttpRequest,
    params: web::Form<GoogleCallbackParams>,
    public_key_cache: web::Data<ArcSwap<Option<GooglePublicKey>>>,
    session: Session,
) -> Result<HttpResponse, error::Error> {
    let AuthConfig::Google { client_id,.. }= &config.auth else {
        return Err(error::ErrorBadRequest("Invalid auth kind"));
    };

    let jwks = if let Some(cache) = public_key_cache.load().as_ref() && cache.expires > Utc::now() {
        cache.jwks.clone()
    } else {
        log::info!("fetch google public key");
        let pubkey = fetch_google_public_key().await;
        match pubkey {
            Ok(pubkey) => {
                public_key_cache.store(Arc::new(Some(pubkey.clone())));
                pubkey.jwks
            }
            Err(e) => {
                log::error!("fetch google public key error: {}", e);
                return Ok(HttpResponse::InternalServerError().finish());
            }
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
        log::info!("verify google token error: {}", e);
        error::ErrorBadRequest(format!("Failed to verify token: {}", e))
    })?;
    session.insert("user_id", token)?;
    Ok(HttpResponse::Ok().finish())
}

#[derive(Debug, Deserialize)]
pub struct DummyAuthParams {
    user_id: String,
}

async fn dummy_auth(
    config: web::Data<AppConfig>,
    query: web::Query<DummyAuthParams>,
    session: Session,
) -> Result<HttpResponse, error::Error> {
    let AuthConfig::Dummy = &config.auth else {
        return Err(error::ErrorBadRequest("Invalid auth kind"));
    };

    session.insert("user_id", query.user_id.clone())?;
    Ok(HttpResponse::Ok().finish())
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    env_logger::init();
    let config = AppConfig::from_env().map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

    let host = config.host.clone();
    let port = config.port;
    let schema = Arc::new(create_schema());
    let pool = PgPoolOptions::new()
        .connect(&config.database_url)
        .await
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
    let google_public_key: web::Data<ArcSwap<Option<GooglePublicKey>>> =
        web::Data::new(ArcSwap::from(Arc::new(None)));
    let secret_key = Key::from(config.session.crypto_key.as_slice());
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(schema.clone()))
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(pool.clone()))
            .app_data(google_public_key.clone())
            .wrap(middleware::Logger::default())
            .wrap(SessionMiddleware::new(
                RedisActorSessionStore::new(config.session.redis_url.clone()),
                secret_key.clone(),
            ))
            .service(web::resource("/graphql").route(web::post().to(graphql)))
            .service(web::resource("/graphiql").route(web::get().to(graphiql)))
            .service(web::resource("/google_login_callback").route(web::post().to(google_callback)))
            .service(web::resource("/google_login").route(web::get().to(google_login_front)))
            .service(web::resource("/dummy_auth").route(web::get().to(dummy_auth)))
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
