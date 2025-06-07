use std::net::SocketAddr;
use axum::{extract::Request, http::StatusCode, middleware::{from_fn, Next}, response::IntoResponse, routing::{get, post}, Router};
use axum_session::{Key, SessionConfig, SessionLayer, SessionStore};
use axum_session_auth::{AuthConfig, AuthSession, AuthSessionLayer};
use axum_session_sqlx::SessionSqlitePool;
use sqlx::{Executor, Pool, Sqlite, SqlitePool};
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing_subscriber;
mod request;
mod model;
use request::*;
use model::user::*;



#[tokio::main]
async fn main() {
  let pool = db().await;
  let session_store = session(pool.clone()).await;
  let app = app(pool, session_store);
  tracing_subscriber::fmt().init();
  let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
  let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
  info!("Starting server on {}", addr);
  axum::serve(listener, app).await.unwrap()
}

async fn db() -> Pool<Sqlite> {
  let pool = sqlx::sqlite::SqlitePool::connect("sqlite://db.sqlite").await.unwrap();
  db_setup(&pool).await;
  list_users(&pool).await;
  pool
}

async fn session(pool: Pool<Sqlite>) -> SessionStore<SessionSqlitePool> {
  let config = SessionConfig::default().with_table_name("session_table").with_key(Key::generate());
  let session_store = SessionStore::<SessionSqlitePool>::new(Some(pool.clone().into()), config).await.unwrap();
  session_store
}

async fn db_setup(pool: &Pool<Sqlite>) {
  pool.execute("
    CREATE TABLE IF NOT EXISTS users (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      username TEXT,
      password TEXT,
      admin BOOL
    )
  ").await.unwrap();
  pool.execute("
    CREATE TABLE IF NOT EXISTS characters (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      name TEXT,
      location_id INTEGER,
      hp INTEGER
    )
  ").await.unwrap();
let rows: Vec<UserSql> = sqlx::query_as("SELECT * FROM users WHERE id = ?1")
  .bind(&1).fetch_all(pool).await.unwrap();
  if rows.len() == 0 {
    sqlx::query("INSERT INTO users (username, password, admin) VALUES (?1, ?2, ?3)")
      .bind(&"guest").bind(&"guest").bind(false).execute(pool).await.unwrap();
    let hash_password = bcrypt::hash("arrakis".to_string(), 10).unwrap();
    sqlx::query("INSERT INTO users (username, password, admin) VALUES (?1, ?2, ?3)")
      .bind(&"admin").bind(&hash_password).bind(true).execute(pool).await.unwrap();
  };
}

async fn list_users(pool: &Pool<Sqlite>) {
  println!("----==== USERS LIST ====----");
  let rows: Vec<UserSql> = sqlx::query_as("SELECT * FROM users").fetch_all(pool).await.unwrap();
  //println!("user count: {}", rows.len());
  for row in rows {
    println!("id: {} | user: {} {}", row.id, row.username, if row.admin { "[admin]" } else { "[user]" });
  }
}

fn app(pool: Pool<Sqlite>, session_store: SessionStore<SessionSqlitePool>) -> Router {
  let config = AuthConfig::<i64>::default().with_anonymous_user_id(Some(1));
  let cors2 = CorsLayer::permissive();
  Router::new()
    .route("/", get(|| async {"Hello world!"}))
    .route("/register", post(user_register))
    .route("/login", get(login))
    .route("/logout", get(logout)).route_layer(from_fn(auth))
    .route("/delete", get(user_remove)).route_layer(from_fn(auth))
    .route("/delete2", post(user_remove2)).route_layer(from_fn(auth))
    .route("/profile", get(user_profile).route_layer(from_fn(auth)))
    .route("/character_create", post(character_create).route_layer(from_fn(auth)))
    .route("/admin", get(admin).route_layer(from_fn(auth)))
    .layer(cors2)
    .layer(AuthSessionLayer::<User, i64, SessionSqlitePool, SqlitePool>::new(Some(pool.clone())).with_config(config))
    .layer(SessionLayer::new(session_store))
    .with_state(pool)
}

async fn auth(auth: AuthSession<User, i64, SessionSqlitePool, SqlitePool>, mut req: Request, next: Next) -> impl IntoResponse {
  if auth.is_authenticated() {
    let user = auth.current_user.unwrap().clone();
    let store = auth.session.get_store();
    let id = auth.session.get_session_id().inner();
    match auth.session.get::<String>(&id) {
      Some(data) => {
        //store.client.unwrap();
        info!("id: {}", &id);
        info!("data: {}", data.as_str());
        //info!("ACCESS GRANTED: {}", user.username.clone());
      },
      None => {
        info!("NONE");
      }
    }
    
    req.extensions_mut().insert(user);
    next.run(req).await
  } else {
      let user = auth.current_user.unwrap().clone();
      info!("ACCESS DENIED: {}", user.username.clone());
      (StatusCode::UNAUTHORIZED, "ACCESS DENIED").into_response()
  }
}

