use anyhow::Ok;
use axum::{extract::{Request, State}, http::StatusCode, middleware::{from_fn, Next}, response::IntoResponse, routing::{get, post}, Extension, Json, Router};
use axum_session::{Key, SessionConfig, SessionLayer, SessionStore};
use axum_session_auth::{AuthConfig, AuthSession, AuthSessionLayer, Authentication};
use axum_session_sqlx::SessionSqlitePool;
use serde::Deserialize;
use sqlx::{prelude::FromRow, Executor, Pool, Sqlite, SqlitePool};
use  async_trait::async_trait;


#[tokio::main]
async fn main() {
  let pool = db().await;
  let session_store = session(pool.clone()).await;
  let app = app(pool, session_store);

  let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

  axum::serve(listener, app).await.unwrap()
}

async fn db() -> Pool<Sqlite> {
  let pool = sqlx::sqlite::SqlitePool::connect("sqlite://db.sqlite").await.unwrap();

  pool.execute("
    CREATE TABLE IF NOT EXISTS user (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      username TEXT,
      password TEXT
    )
  ").await.unwrap();

  let rows: Vec<UserSql> = sqlx::query_as("SELECT * FROM user WHERE id = ?1").bind(&1).fetch_all(&pool).await.unwrap();

  if rows.len() == 0 {
    sqlx::query("INSERT INTO user (username, password) VALUES (?1, ?2)").bind(&"guest").bind(&"guest").execute(&pool).await.unwrap();
  };

  pool
}

async fn session(pool: Pool<Sqlite>) -> SessionStore<SessionSqlitePool> {
  let config = SessionConfig::default().with_table_name("session_table").with_key(Key::generate());

  let session_store = SessionStore::<SessionSqlitePool>::new(Some(pool.clone().into()), config).await.unwrap();

  session_store
}

fn app(pool: Pool<Sqlite>, session_store : SessionStore<SessionSqlitePool>) -> Router {
  let config = AuthConfig::<i64>::default().with_anonymous_user_id(Some(1));
  Router::new()
    .route("/", get(|| async {"Hello world!"}))
    .route("/register", post(register))
    .route("/login", post(login))
    .route("/logout", get(log_out))
    .route("/protected", get(protected).route_layer(from_fn(auth)))
    .layer(AuthSessionLayer::<User, i64, SessionSqlitePool, SqlitePool>::new(Some(pool.clone())).with_config(config))
    .layer(SessionLayer::new(session_store))
    .with_state(pool)
}

async fn register(State(pool): State<Pool<Sqlite>>, Json(user): Json<UserRequest>) -> impl IntoResponse {
  let rows : Vec<UserSql> = sqlx::query_as("SELECT * FROM user WHERE username = ?1").bind(&user.username).fetch_all(&pool).await.unwrap();

  if rows.len() != 0 {
    let msg = format!("Username : {} is already taken!", user.username);
    (StatusCode::BAD_REQUEST, msg).into_response()
  } else {
      let hash_password = bcrypt::hash(user.password, 10).unwrap();

      sqlx::query("INSERT INTO user (username, password) VALUES (?1, ?2)").bind(&user.username).bind(&hash_password).execute(&pool).await.unwrap();
      (StatusCode::OK, "Register successful!").into_response()
  }
}

async fn login(auth: AuthSession<User, i64, SessionSqlitePool, SqlitePool>, State(pool): State<Pool<Sqlite>>, Json(user): Json<UserRequest>) -> impl IntoResponse {
  let rows: Vec<UserSql> = sqlx::query_as("SELECT * FROM user WHERE username = ?1").bind(&user.username).fetch_all(&pool).await.unwrap();
  if rows.len() == 0 {
    let msg = format!("Username : {} is not registered", user.username);
    (StatusCode::BAD_REQUEST, msg).into_response()
  } else {
      let is_valid = bcrypt::verify(user.password, &rows[0].password).unwrap();
      if is_valid {
        auth.login_user(rows[0].id as i64);
        (StatusCode::OK, "Login successful").into_response()
      } else {
          (StatusCode::UNAUTHORIZED, "Password is incorrect!").into_response()
      }
  }
}

async fn log_out(auth: AuthSession<User, i64, SessionSqlitePool, SqlitePool>) -> impl IntoResponse {
  auth.logout_user();
  (StatusCode::OK, "Log out successful!").into_response()
}

async fn protected(Extension(user): Extension<User>) -> impl IntoResponse {
  let msg = format!("Hello , {} , your id is {}", user.username, user.id);
  (StatusCode::OK, msg).into_response()
}

async fn auth(auth: AuthSession<User, i64, SessionSqlitePool, SqlitePool>, mut req: Request, next: Next) -> impl IntoResponse {
  if auth.is_authenticated() {
    let user = auth.current_user.unwrap().clone();
    req.extensions_mut().insert(user);
    next.run(req).await
  } else {
      (StatusCode::UNAUTHORIZED, "Guest, you are unauthorized!").into_response()
  }
}

#[derive(Deserialize)]
struct UserRequest {
  username: String,
  password: String
}

#[derive(Clone)]
pub struct User {
  pub id : i64,
  pub anonymous: bool,
  pub username: String
}

#[async_trait]
impl Authentication<User, i64, SqlitePool> for  User {
    async fn load_user(userid:i64,pool:Option< &SqlitePool>) -> Result<User, anyhow::Error> {
        if userid == 1 {
          Ok(User { id: userid, anonymous: true, username: "guest".to_string() })
        } else {
            let user: UserSql = sqlx::query_as("SELECT * FROM user WHERE id = ?1").bind(&userid).fetch_one(pool.unwrap()).await.unwrap();
            Ok(User { id: user.id as i64, anonymous: false, username: user.username })
        }
    }

    fn is_active(&self) -> bool {
        !self.anonymous
    }

    fn is_anonymous(&self) -> bool {
        self.anonymous
    }
    fn is_authenticated(&self) -> bool {
        !self.anonymous
    }
}


#[derive(FromRow)]
struct UserSql {
  id: i32,
  username: String,
  password: String
}
