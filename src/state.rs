use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Started {},
    Finished {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]

pub struct Admin {
    pub owner: Addr,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]

pub struct Player {
    pub address: Addr,
    pub choosed_number: u32,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GameState {
    pub participants: Vec<Player>,
    pub random_number: Option<u32>,
    pub winner: Option<Addr>,
    pub num_diff: u32,
    pub status: Option<Status>,
}

pub const GAMESTATE: Item<GameState> = Item::new("game_state");
pub const ADMIN: Item<Admin> = Item::new("admin");
