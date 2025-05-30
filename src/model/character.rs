use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;


#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Character {
  pub location_id: u64,
  pub name: String,
  pub hp: i32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CharacterRequest {
    pub name: String
}


#[derive(FromRow, Debug)]
pub struct CharacterSql {
  pub id: u64,
  pub name: String,
  pub location_id: String,
  pub hp: i32,
}


impl Character {
    pub fn new(name: String) -> Self {
        Self {
            location_id: 1,
            name,
            hp: 100,
        }
    }
}