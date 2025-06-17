//use std::env::*;
use std::net::SocketAddr;
use axum::{middleware::from_fn, routing::{get, post}, Router};
use axum_session::{SessionLayer, SessionStore};
use axum_session_auth::{AuthConfig, AuthSessionLayer};
use axum_session_sqlx::SessionSqlitePool;
use sqlx::{Pool, Sqlite, SqlitePool};
use tower_http::cors::CorsLayer;
use tracing_subscriber;
mod request;
mod model;
mod db;
mod session;
use db::*;
use request::*;
use session::*;
use model::user::*;



#[tokio::main]
async fn main() {
  tracing_subscriber::fmt().init();
  let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
  let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
  let name = env!("CARGO_PKG_NAME");
  let version = env!("CARGO_PKG_VERSION");
  let author = env!("CARGO_PKG_AUTHORS");
  println!("");
  println!("{}", name.to_uppercase());
  println!("ver. {}", version);
  println!("by {}", author);
  println!("");
  println!("running on {}", addr);
  println!("");
  let pool = get_db().await;
  let session_store = session(pool.clone()).await;
  let app = app(pool, session_store);
  axum::serve(listener, app).await.unwrap()
}

fn app(pool: Pool<Sqlite>, session_store: SessionStore<SessionSqlitePool>) -> Router {
  let config = AuthConfig::<i64>::default().with_anonymous_user_id(Some(1));
  let cors2 = CorsLayer::permissive();
  Router::new()
    .route("/", get(|| async {"Hello world!"}))
    .route("/register", post(user_register))
    .route("/login", get(login))
    .route("/logout", get(logout)).route_layer(from_fn(auth))
    .route("/delete", post(user_remove)).route_layer(from_fn(auth))
    .route("/delete2", post(user_remove2)).route_layer(from_fn(auth))
    .route("/profile", get(user_profile).route_layer(from_fn(auth)))
    .route("/character_create", post(character_create).route_layer(from_fn(auth)))
    .route("/admin", get(admin).route_layer(from_fn(auth)))
    .layer(cors2)
    .layer(AuthSessionLayer::<User, i64, SessionSqlitePool, SqlitePool>::new(Some(pool.clone())).with_config(config))
    .layer(SessionLayer::new(session_store))
    .with_state(pool)
}