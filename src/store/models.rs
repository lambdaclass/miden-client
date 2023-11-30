use diesel::prelude::*;

use super::schema;

#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::account_code)]
pub struct AccountCode {
    pub root: Vec<u8>,
    pub procedures: Vec<u8>,
    pub module: Vec<u8>,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::account_storage)]
pub struct AccountStorage {
    pub root: Vec<u8>,
    pub slots: Vec<u8>,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::account_vaults)]
pub struct AccountVaults {
    pub root: Vec<u8>,
    pub assets: Vec<u8>,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::account_keys)]
pub struct AccountKeys {
    pub account_id: u64,
    pub key_pair: Vec<u8>,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::accounts)]
pub struct Accounts {
    pub id: i64,
    pub code_root: Vec<u8>,
    pub storage_root: Vec<u8>,
    pub vault_root: Vec<u8>,
    pub nonce: i64,
    pub committed: bool,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::input_notes)]
pub struct InputNotes {
    pub hash: Vec<u8>,
    pub nullifier: Vec<u8>,
    pub script: Vec<u8>,
    pub vault: Vec<u8>,
    pub inputs: Vec<u8>,
    pub serial_num: Vec<u8>,
    pub sender_id: u64,
    pub tag: u64,
    pub num_assets: u64,
    pub inclusion_proof: Vec<u8>,
    pub recipients: Vec<u8>,
    pub status: String,
    pub commit_height: u64,
}
