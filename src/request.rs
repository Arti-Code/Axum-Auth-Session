use axum::{extract::State, http::StatusCode, response::IntoResponse, *};
use axum_session_auth::AuthSession;
use axum_session_sqlx::SessionSqlitePool;
use sqlx::{Pool, Sqlite, SqlitePool};
use tracing::info;
use crate::model::{character::CharacterRequest, user::*};



pub async fn user_register(State(pool): State<Pool<Sqlite>>, Json(user): Json<UserRequest>) -> impl IntoResponse {
  println!("register: {}", user.username);
  let rows: Vec<UserSql> = sqlx::query_as("SELECT * FROM users WHERE username = ?1")
    .bind(&user.username).fetch_all(&pool).await.unwrap();
  if rows.len() != 0 {
    let msg = format!("username: {} is already taken!", user.username);
    (StatusCode::BAD_REQUEST, msg).into_response()
  } else {
      let hash_password = bcrypt::hash(user.password, 10).unwrap();
      sqlx::query("INSERT INTO users (username, password, admin) VALUES (?1, ?2, ?3)")
        .bind(&user.username).bind(&hash_password).bind(false)
        .execute(&pool).await.unwrap();
      println!("user: {} is registered!", user.username);
      (StatusCode::OK, "Register successful!").into_response()
  }
}

pub async fn login(auth: AuthSession<User, i64, SessionSqlitePool, SqlitePool>, State(pool): State<Pool<Sqlite>>, Json(user): Json<UserRequest>) -> impl IntoResponse {
  let rows: Vec<UserSql> = sqlx::query_as("SELECT * FROM users WHERE username = ?1")
    .bind(&user.username).fetch_all(&pool).await.unwrap();
  if rows.len() == 0 {
    let msg = format!("username: {} is not registered", user.username);
    (StatusCode::BAD_REQUEST, msg).into_response()
  } else {      
      let is_valid = bcrypt::verify(user.password, &rows[0].password).unwrap();
      if is_valid {
        auth.login_user(rows[0].id as i64);
        println!("{} logged in!", user.username);
        let id = auth.session.get_session_id();
        
        println!("[SESSION DATA]: uuid:{}", id.uuid());
        println!("[SESSION DATA]: inner:{}", id.inner());
        
        match auth.session.get::<String>(&auth.session.get_session_id().to_string()) {
          Some(s) => {
            println!("[SESSION DATA]: id:{} | data:{}", id, s);
          },
          None => {
            //println!("can't found session data");
          }
        }
        let response_user = UserSession {
          session: id.to_string(),
          username: rows[0].username.clone(),
          admin: rows[0].admin,
        };
        
        (StatusCode::OK, Json(response_user)).into_response()
      } else {
          (StatusCode::UNAUTHORIZED, "Password is incorrect!").into_response()
      }
  }
}

pub async fn logout(auth: AuthSession<User, i64, SessionSqlitePool, SqlitePool>) -> impl IntoResponse {
  auth.logout_user();
  println!("user logged out!");
  (StatusCode::OK, "Log out successful!").into_response()
}

pub async fn user_profile(Extension(user): Extension<User>) -> impl IntoResponse {
  //let msg = format!("Hello , {} , your id is {}", user.username, user.id);
  //println!("profile: {} ({})", user.username, user.id);
  //Json(user)
  (StatusCode::OK, Json(user)).into_response()
}

pub async fn user_remove(auth: AuthSession<User, i64, SessionSqlitePool, SqlitePool>, Extension(user): Extension<User>, State(pool): State<Pool<Sqlite>>,) -> impl IntoResponse {
    match &auth.current_user {
      Some(current_user) => {
        if current_user.admin || current_user.username == user.username {
          match sqlx::query("DELETE FROM users WHERE username = ?1")
            .bind(&user.username).execute(&pool).await {
            Ok(_) => {
              if  current_user.username == user.username {
                auth.logout_user();
              }
              println!("{} removed", user.username);
              return (StatusCode::OK, "User removed").into_response()
            },
            Err(e) => {
              println!("Error: {}", e);
              return (StatusCode::BAD_REQUEST, e.to_string()).into_response()
            }
          }
        } else {
          return (StatusCode::BAD_REQUEST, "Error").into_response()
        }
      },
      None => {
        return (StatusCode::NETWORK_AUTHENTICATION_REQUIRED, "Authentication needed").into_response()
      }
    }
}

pub async fn user_remove2(auth: AuthSession<User, i64, SessionSqlitePool, SqlitePool>, Extension(user): Extension<String>, State(pool): State<Pool<Sqlite>>,) -> impl IntoResponse {
    match &auth.current_user {
      Some(current_user) => {
        if current_user.admin || current_user.username == user {
          println!("removing user: {}", user);
          match sqlx::query("DELETE FROM users WHERE username = ?1")
            .bind(&current_user.username).execute(&pool).await {
            Ok(_) => {
              println!("user {} removed", user);
              if current_user.username == user {
                auth.logout_user();
              }
              //println!("{} removed", user.username);
              return (StatusCode::OK, "User removed").into_response()
            },
            Err(e) => {
              println!("Error: {}", e);
              return (StatusCode::BAD_REQUEST, e.to_string()).into_response()
            }
          }
        } else {
          println!("Error: user {} is not admin", current_user.username);
          return (StatusCode::BAD_REQUEST, "Error").into_response()
        }
      },
      None => {
        println!("Authentication needed");
        return (StatusCode::NETWORK_AUTHENTICATION_REQUIRED, "Authentication needed").into_response()
      }
    }
}


// CHARACTER

pub async fn character_create(State(pool): State<Pool<Sqlite>>, Json(user): Json<CharacterRequest>) -> impl IntoResponse {
  println!("register: {}", user.name);
  let rows: Vec<UserSql> = sqlx::query_as("SELECT * FROM characters WHERE name = ?1")
    .bind(&user.name).fetch_all(&pool).await.unwrap();
  if rows.len() != 0 {
    let msg = format!("character: {} is exists!", user.name);
    (StatusCode::BAD_REQUEST, msg).into_response()
  } else {
      sqlx::query("INSERT INTO characters (name, location_id, hp) VALUES (?1, ?2 ?3)")
        .bind(&user.name).bind(1).bind(100).execute(&pool).await.unwrap();
      println!("Character {} created!", user.name);
      (StatusCode::OK, "Character created!").into_response()
  }
}

pub async fn admin(Extension(user): Extension<User>, State(pool): State<Pool<Sqlite>>) -> impl IntoResponse {
    if !user.is_admin() {
        (StatusCode::NETWORK_AUTHENTICATION_REQUIRED, "administrator access only").into_response()
    } else {
        let users: Vec<UserSql> = sqlx::query_as("SELECT * from users").fetch_all(&pool).await.unwrap();
        println!("{} users founded", users.len());
        let users: Vec<User> = users.iter().map(|u| {
          User {
            username: u.username.clone(),
            id: u.id as i64,
            anonymous: false,
            admin: u.admin,
          }
        }).collect();
        (StatusCode::OK, Json(users)).into_response()
    }
}