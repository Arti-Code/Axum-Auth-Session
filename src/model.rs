use async_trait::async_trait;
use axum_session_auth::Authentication;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, SqlitePool};
use tracing::info;




#[derive(Deserialize)]
pub struct UserRequest {
  pub username: String,
  pub password: String
}

#[derive(Clone, Serialize)]
pub struct User {
  pub id: i64,
  pub anonymous: bool,
  pub username: String
}

#[async_trait]
impl Authentication<User, i64, SqlitePool> for  User {
    async fn load_user(userid:i64,pool:Option< &SqlitePool>) -> Result<User, anyhow::Error> {
      let user: UserSql = sqlx::query_as("SELECT * FROM user WHERE id = ?1").bind(&userid).fetch_one(pool.unwrap()).await.unwrap();
      info!("{} is loaded!", user.username);
      Ok(User { id: user.id as i64, anonymous: false, username: user.username })
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


#[derive(FromRow, Debug)]
pub struct UserSql {
  pub id: i32,
  pub username: String,
  pub password: String
}

/* impl Debug for UserSql {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "UserSql {{ id: {}, username: {}, password: {} }}", self.id, self.username, self.password)
  }
} */