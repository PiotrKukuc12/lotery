use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use lotery::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, ResultAdmin, ResultGameInfo};
use lotery::state::{Admin, GameState};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(Admin), &out_dir);
    export_schema(&schema_for!(GameState), &out_dir);
    export_schema(&schema_for!(ResultAdmin), &out_dir);
    export_schema(&schema_for!(ResultGameInfo), &out_dir);
}
