use sqlx::{Executor, Pool, Sqlite};


use crate::model::user::UserSql;



pub async fn get_db() -> Pool<Sqlite> {
  let pool = sqlx::sqlite::SqlitePool::connect("sqlite://db.sqlite").await.unwrap();
  db_setup(&pool).await;
  list_users(&pool).await;
  pool
}


async fn db_setup(pool: &Pool<Sqlite>) {
    add_users_table(pool).await;
    add_characters_table(pool).await;
    add_robots_table(pool).await;
    init_users(pool).await;
/* let rows: Vec<UserSql> = sqlx::query_as("SELECT * FROM users WHERE id = ?1")
  .bind(&1).fetch_all(pool).await.unwrap();
  if rows.len() == 0 {
    sqlx::query("INSERT INTO users (username, password, admin) VALUES (?1, ?2, ?3)")
      .bind(&"guest").bind(&"guest").bind(false).execute(pool).await.unwrap();
    let hash_password = bcrypt::hash("arrakis".to_string(), 10).unwrap();
    sqlx::query("INSERT INTO users (username, password, admin) VALUES (?1, ?2, ?3)")
      .bind(&"admin").bind(&hash_password).bind(true).execute(pool).await.unwrap();
  }; */
}

async fn list_users(pool: &Pool<Sqlite>) {
  println!("[USERS LIST]");
  let rows: Vec<UserSql> = sqlx::query_as("SELECT * FROM users").fetch_all(pool).await.unwrap();
  for row in rows {
    println!("id: {} | user: {} {}", row.id, row.username, if row.admin { "[admin]" } else { "[user]" });
  }
    println!("--------------------");
}

async fn add_users_table(pool: &Pool<Sqlite>) {
  pool.execute("
    CREATE TABLE IF NOT EXISTS users (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      username TEXT NOT NULL,
      password TEXT NOT NULL,
      admin BOOL DEFAULT FALSE
    )
  ").await.unwrap();
}

async fn add_characters_table(pool: &Pool<Sqlite>) {
  pool.execute("
    CREATE TABLE IF NOT EXISTS characters (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      name TEXT NOT NULL,
      location_id INTEGER,
      hp INTEGER DEFAULT 100
    )
  ").await.unwrap();
}

async fn add_robots_table(pool: &Pool<Sqlite>) {
  pool.execute("
    CREATE TABLE IF NOT EXISTS robots (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      name TEXT NOT NULL,
      owner_id INTEGER NOT NULL,
      FOREIGN KEY (owner_id) REFERENCES users(id)
    )
  ").await.unwrap();
}

async fn init_users(pool: &Pool<Sqlite>) {
    let rows: Vec<UserSql> = sqlx::query_as("SELECT * FROM users WHERE id = ?1")
        .bind(&1).fetch_all(pool).await.unwrap();
    if rows.len() == 0 {
        sqlx::query("INSERT INTO users (username, password, admin) VALUES (?1, ?2, ?3)")
        .bind(&"guest").bind(&"guest").bind(false).execute(pool).await.unwrap();
        let hash_password = bcrypt::hash("arrakis".to_string(), 10).unwrap();
        sqlx::query("INSERT INTO users (username, password, admin) VALUES (?1, ?2, ?3)")
        .bind(&"admin").bind(&hash_password).bind(true).execute(pool).await.unwrap();
    }
}