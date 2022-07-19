use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, export_schema_with_title, remove_schemas, schema_for};

use neptune::{
    base_config::{BaseConfig, BaseSetConfigMsg},
    execute_base::BaseExecuteMsg,
    investment,
};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(BaseExecuteMsg), &out_dir);
    export_schema(&schema_for!(BaseConfig), &out_dir);
    export_schema(&schema_for!(BaseSetConfigMsg), &out_dir);
    export_schema_with_title(&schema_for!(investment::QueryMsg), &out_dir, "investment_query_msg");
    export_schema_with_title(&schema_for!(investment::ExecuteMsg), &out_dir, "investment_execute_msg");


}