use axum::{extract::Request, http::StatusCode, middleware::Next, response::IntoResponse};
use axum_extra::extract::CookieJar;
use axum_session::{Key, SessionConfig, SessionStore};
use axum_session_auth::AuthSession;
use axum_session_sqlx::SessionSqlitePool;
use sqlx::{Pool, Sqlite, SqlitePool};
use tracing::info;

use crate::model::user::User;




pub async fn session(pool: Pool<Sqlite>) -> SessionStore<SessionSqlitePool> {
  let config = SessionConfig::default().with_table_name("session_table").with_key(Key::generate());
  let session_store = SessionStore::<SessionSqlitePool>::new(Some(pool.clone().into()), config).await.unwrap();
  session_store
}


pub async fn auth(auth: AuthSession<User, i64, SessionSqlitePool, SqlitePool>, mut req: Request, next: Next) -> impl IntoResponse {
  let jar = CookieJar::from_headers(&req.headers());
  jar.get("session").map(|cookie| {
    info!("Session cookie found: {}", cookie.value());
  }).unwrap_or_else(|| {
    info!("No session cookie found");
  });
  if auth.is_authenticated() {
    let user = auth.current_user.unwrap().clone();
    //let store = auth.session.get_store();
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

