use axum::{extract::State, http::StatusCode, response::IntoResponse, *};
use axum_session_auth::AuthSession;
use axum_session_sqlx::SessionSqlitePool;
use sqlx::{Pool, Sqlite, SqlitePool};
use tracing::info;
use crate::model::*;



pub async fn register(State(pool): State<Pool<Sqlite>>, Json(user): Json<UserRequest>) -> impl IntoResponse {
  info!("register: {}", user.username);
  let rows: Vec<UserSql> = sqlx::query_as("SELECT * FROM user WHERE username = ?1").bind(&user.username).fetch_all(&pool).await.unwrap();
  if rows.len() != 0 {
    let msg = format!("username: {} is already taken!", user.username);
    (StatusCode::BAD_REQUEST, msg).into_response()
  } else {
      let hash_password = bcrypt::hash(user.password, 10).unwrap();
      sqlx::query("INSERT INTO user (username, password) VALUES (?1, ?2)").bind(&user.username).bind(&hash_password).execute(&pool).await.unwrap();
      info!("user: {} is registered!", user.username);
      (StatusCode::OK, "Register successful!").into_response()
  }
}

pub async fn login(auth: AuthSession<User, i64, SessionSqlitePool, SqlitePool>, State(pool): State<Pool<Sqlite>>, Json(user): Json<UserRequest>) -> impl IntoResponse {
  let rows: Vec<UserSql> = sqlx::query_as("SELECT * FROM user WHERE username = ?1").bind(&user.username).fetch_all(&pool).await.unwrap();
  if rows.len() == 0 {
    let msg = format!("username: {} is not registered", user.username);
    (StatusCode::BAD_REQUEST, msg).into_response()
  } else {
      let is_valid = bcrypt::verify(user.password, &rows[0].password).unwrap();
      if is_valid {
        auth.login_user(rows[0].id as i64);
        info!("{} logged in!", user.username);
        (StatusCode::OK, "Login successful").into_response()
      } else {
          (StatusCode::UNAUTHORIZED, "Password is incorrect!").into_response()
      }
  }
}

pub async fn logout(auth: AuthSession<User, i64, SessionSqlitePool, SqlitePool>) -> impl IntoResponse {
  auth.logout_user();
  info!("user logged out!");
  (StatusCode::OK, "Log out successful!").into_response()
}

pub async fn profile(Extension(user): Extension<User>) -> impl IntoResponse {
  //let msg = format!("Hello , {} , your id is {}", user.username, user.id);
  info!("profile: {} ({})", user.username, user.id);
  //Json(user)
  (StatusCode::OK, Json(user)).into_response()
}

pub async fn remove(auth: AuthSession<User, i64, SessionSqlitePool, SqlitePool>, Extension(user): Extension<User>, State(pool): State<Pool<Sqlite>>,) -> impl IntoResponse {
    match sqlx::query("DELETE FROM user WHERE username = ?1").bind(&user.username).execute(&pool).await {
        Ok(_) => {
            auth.logout_user();
            info!("{} removed", user.username);
            return (StatusCode::OK, "User removed").into_response()
        },
        Err(e) => {
            info!("Error: {}", e);
            return (StatusCode::BAD_REQUEST, "Error").into_response()
        }
    }
}