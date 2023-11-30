use std::collections::BTreeMap;

use crypto::Word;
use diesel::prelude::*;
use objects::{
    accounts::{
        Account, AccountCode as AccountCodeObject, AccountStorage as AccountStorageObject,
        AccountStub, AccountVault as AccountVaultObject,
    },
    assembly::AstSerdeOptions,
    assets::Asset,
};

use super::schema;

// ACCOUNT CODE TABLE
// --------------------------------------------------------------------------------------------

#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::account_code)]
pub struct AccountCode {
    pub root: Vec<u8>,
    pub procedures: Vec<u8>,
    pub module: Vec<u8>,
}

#[derive(Insertable)]
#[diesel(table_name = schema::account_code)]
pub struct NewAccountCode {
    pub root: Vec<u8>,
    pub procedures: Vec<u8>,
    pub module: Vec<u8>,
}

impl NewAccountCode {
    pub fn from_account_code(account_code: &AccountCodeObject) -> Result<NewAccountCode, ()> {
        Ok(NewAccountCode {
            root: serde_json::to_string(&account_code.root())
                .unwrap() // TODO: remove unwraps
                .into_bytes(),
            procedures: serde_json::to_string(&account_code.procedures())
                .unwrap() // TODO: remove unwraps
                .into_bytes(),
            module: account_code.module().to_bytes(AstSerdeOptions {
                serialize_imports: true,
            }),
        })
    }
}

// ACCOUNT STORAGE TABLE
// --------------------------------------------------------------------------------------------

#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::account_storage)]
pub struct AccountStorage {
    pub root: Vec<u8>,
    pub slots: Vec<u8>,
}

#[derive(Insertable)]
#[diesel(table_name = schema::account_storage)]
pub struct NewAccountStorage {
    pub root: Vec<u8>,
    pub slots: Vec<u8>,
}

impl NewAccountStorage {
    pub fn from_account_storage(
        account_storage: &AccountStorageObject,
    ) -> Result<NewAccountStorage, ()> {
        let storage_slots: BTreeMap<u64, &Word> = account_storage.slots().leaves().collect();
        let slots = serde_json::to_string(&storage_slots).unwrap().into_bytes(); // TODO: remove unwraps

        Ok(NewAccountStorage {
            root: serde_json::to_string(&account_storage.root())
                .unwrap() // TODO: remove unwraps
                .into_bytes(),
            slots,
        })
    }
}

// ACCOUNT VAULTS TABLE
// --------------------------------------------------------------------------------------------

#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::account_vaults)]
pub struct AccountVaults {
    pub root: Vec<u8>,
    pub assets: Vec<u8>,
}

#[derive(Insertable)]
#[diesel(table_name = schema::account_vaults)]
pub struct NewAccountVault {
    pub root: Vec<u8>,
    pub assets: Vec<u8>,
}

impl NewAccountVault {
    pub fn from_account_vault(account_vault: &AccountVaultObject) -> Result<NewAccountVault, ()> {
        let assets: Vec<Asset> = account_vault.assets().collect();
        let assets = serde_json::to_string(&assets).unwrap().into_bytes();
        Ok(NewAccountVault {
            root: serde_json::to_string(&account_vault.commitment())
                .unwrap()
                .into_bytes(),
            assets,
        })
    }
}

// ACCOUNT KEYS TABLE
// --------------------------------------------------------------------------------------------

#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::account_keys)]
pub struct AccountKeys {
    pub account_id: u64,
    pub key_pair: Vec<u8>,
}

// ACCOUNTS TABLE
// --------------------------------------------------------------------------------------------

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

impl Accounts {
    pub fn to_account_stub(&self) -> Result<AccountStub, ()> {
        Ok(AccountStub::new(
            (self.id as u64)
                .try_into()
                .expect("Conversion from stored AccountID should not panic"),
            (self.nonce as u64).into(),
            serde_json::from_str(&String::from_utf8(self.vault_root.clone()).unwrap()).unwrap(), // TODO: remove unwraps
            serde_json::from_str(&String::from_utf8(self.storage_root.clone()).unwrap()).unwrap(), // TODO: remove unwraps
            serde_json::from_str(&String::from_utf8(self.code_root.clone()).unwrap()).unwrap(), // TODO: remove unwraps
        ))
    }
}

#[derive(Insertable)]
#[diesel(table_name = schema::accounts)]
pub struct NewAccount {
    pub id: i64,
    pub code_root: Vec<u8>,
    pub storage_root: Vec<u8>,
    pub vault_root: Vec<u8>,
    pub nonce: i64,
    pub committed: bool,
}

impl NewAccount {
    pub fn from_account(account: &Account) -> Result<NewAccount, ()> {
        let id: u64 = account.id().into();
        Ok(NewAccount {
            id: id as i64,
            code_root: serde_json::to_string(&account.code().root())
                .unwrap() // TODO: remove unwraps
                .into_bytes(),
            storage_root: serde_json::to_string(&account.storage().root())
                .unwrap() // TODO: remove unwraps
                .into_bytes(),
            vault_root: serde_json::to_string(&account.vault().commitment())
                .unwrap() // TODO: remove unwraps
                .into_bytes(),
            nonce: account.nonce().inner() as i64,
            committed: account.is_on_chain(),
        })
    }
}

// INPUT NOTES TABLE
// --------------------------------------------------------------------------------------------

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
