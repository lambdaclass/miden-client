use clap::error::Result;
use miden_lib::transaction::TransactionKernel;
use miden_objects::{
    accounts::{Account, AccountCode, AccountId, AccountStorage, AccountStub},
    assembly::{AstSerdeOptions, ModuleAst},
    assets::{Asset, AssetVault},
    Digest, Felt, Word,
};
use miden_tx::utils::{Deserializable, Serializable};
use rusqlite::{params, Transaction};

use super::SqliteStore;
use crate::{errors::StoreError, store::AuthInfo};

// TYPES
// ================================================================================================
type SerializedAccountData = (i64, String, String, String, i64, bool);
type SerializedAccountsParts = (i64, i64, String, String, String, Option<Vec<u8>>);

type SerializedAccountAuthData = (i64, Vec<u8>);
type SerializedAccountAuthParts = (i64, Vec<u8>);

type SerializedAccountVaultData = (String, String);
type SerializedAccountVaultParts = (String, String);

type SerializedAccountCodeData = (String, String, Vec<u8>);
type SerializedAccountCodeParts = (String, String, Vec<u8>);

type SerializedAccountStorageData = (String, Vec<u8>);
type SerializedAccountStorageParts = (String, Vec<u8>);

impl SqliteStore {
    // ACCOUNTS
    // --------------------------------------------------------------------------------------------

    pub(super) fn get_account_ids(&self) -> Result<Vec<AccountId>, StoreError> {
        const QUERY: &str = "SELECT DISTINCT id FROM accounts";

        self.db
            .prepare(QUERY)?
            .query_map([], |row| row.get(0))
            .expect("no binding parameters used in query")
            .map(|result| {
                Ok(result
                    .map(|id: i64| AccountId::try_from(id as u64).expect("account id is valid"))?)
            })
            .collect::<Result<Vec<AccountId>, StoreError>>()
    }

    pub(super) fn get_account_stubs(&self) -> Result<Vec<(AccountStub, Option<Word>)>, StoreError> {
        const QUERY: &str =
            "SELECT a.id, a.nonce, a.vault_root, a.storage_root, a.code_root, a.account_seed \
            FROM accounts a \
            WHERE a.nonce = (SELECT MAX(b.nonce) FROM accounts b WHERE b.id = a.id)";

        self.db
            .prepare(QUERY)?
            .query_map([], parse_accounts_columns)
            .expect("no binding parameters used in query")
            .map(|result| Ok(result?).and_then(parse_accounts))
            .collect()
    }

    pub(crate) fn get_account_stub(
        &self,
        account_id: AccountId,
    ) -> Result<(AccountStub, Option<Word>), StoreError> {
        let account_id_int: u64 = account_id.into();
        const QUERY: &str = "SELECT id, nonce, vault_root, storage_root, code_root, account_seed \
            FROM accounts WHERE id = ? \
            ORDER BY nonce DESC \
            LIMIT 1";
        self.db
            .prepare(QUERY)?
            .query_map(params![account_id_int as i64], parse_accounts_columns)?
            .map(|result| Ok(result?).and_then(parse_accounts))
            .next()
            .ok_or(StoreError::AccountDataNotFound(account_id))?
    }

    // TODO: Get all parts from a single query
    pub(crate) fn get_account(
        &self,
        account_id: AccountId,
    ) -> Result<(Account, Option<Word>), StoreError> {
        let (account_stub, seed) = self.get_account_stub(account_id)?;
        let (_procedures, module_ast) = self.get_account_code(account_stub.code_root())?;

        let account_code = AccountCode::new(module_ast, &TransactionKernel::assembler()).unwrap();

        let account_storage = self.get_account_storage(account_stub.storage_root())?;

        let account_vault = self.get_vault_assets(account_stub.vault_root())?;
        let account_vault = AssetVault::new(&account_vault)?;

        let account = Account::new(
            account_stub.id(),
            account_vault,
            account_storage,
            account_code,
            account_stub.nonce(),
        );

        Ok((account, seed))
    }

    /// Retrieve account keys data by Account Id
    pub(crate) fn get_account_auth(&self, account_id: AccountId) -> Result<AuthInfo, StoreError> {
        let account_id_int: u64 = account_id.into();
        const QUERY: &str = "SELECT account_id, auth_info FROM account_auth WHERE account_id = ?";
        self.db
            .prepare(QUERY)?
            .query_map(params![account_id_int as i64], parse_account_auth_columns)?
            .map(|result| Ok(result?).and_then(parse_account_auth))
            .next()
            .ok_or(StoreError::AccountDataNotFound(account_id))?
    }

    /// Retrieve account code-related data by code root
    pub(super) fn get_account_code(
        &self,
        root: Digest,
    ) -> Result<(Vec<Digest>, ModuleAst), StoreError> {
        let root_serialized = root.to_string();
        const QUERY: &str = "SELECT root, procedures, module FROM account_code WHERE root = ?";

        self.db
            .prepare(QUERY)?
            .query_map(params![root_serialized], parse_account_code_columns)?
            .map(|result| Ok(result?).and_then(parse_account_code))
            .next()
            .ok_or(StoreError::AccountCodeDataNotFound(root))?
    }

    /// Retrieve account storage data by vault root
    pub(super) fn get_account_storage(&self, root: Digest) -> Result<AccountStorage, StoreError> {
        let root_serialized = &root.to_string();

        const QUERY: &str = "SELECT root, slots FROM account_storage WHERE root = ?";
        self.db
            .prepare(QUERY)?
            .query_map(params![root_serialized], parse_account_storage_columns)?
            .map(|result| Ok(result?).and_then(parse_account_storage))
            .next()
            .ok_or(StoreError::AccountStorageNotFound(root))?
    }

    /// Retrieve assets by vault root
    pub(super) fn get_vault_assets(&self, root: Digest) -> Result<Vec<Asset>, StoreError> {
        let vault_root =
            serde_json::to_string(&root).map_err(StoreError::InputSerializationError)?;

        const QUERY: &str = "SELECT root, assets FROM account_vaults WHERE root = ?";
        self.db
            .prepare(QUERY)?
            .query_map(params![vault_root], parse_account_asset_vault_columns)?
            .map(|result| Ok(result?).and_then(parse_account_asset_vault))
            .next()
            .ok_or(StoreError::VaultDataNotFound(root))?
    }

    pub(crate) fn insert_account(
        &mut self,
        account: &Account,
        account_seed: Option<Word>,
        auth_info: &AuthInfo,
    ) -> Result<(), StoreError> {
        let tx = self.db.transaction()?;

        insert_account_code(&tx, account.code())?;
        insert_account_storage(&tx, account.storage())?;
        insert_account_asset_vault(&tx, account.vault())?;
        insert_account_record(&tx, account, account_seed)?;
        insert_account_auth(&tx, account.id(), auth_info)?;

        Ok(tx.commit()?)
    }
}

// HELPERS
// ================================================================================================

/// Update previously-existing account after a transaction execution
///
/// Because the Client retrieves the account by account ID before applying the delta, we don't
/// need to check that it exists here. This inserts a new row into the accounts table.
/// We can later identify the proper account state by looking at the nonce.
pub(crate) fn update_account(
    tx: &Transaction<'_>,
    new_account_state: &Account,
) -> Result<(), StoreError> {
    insert_account_storage(tx, new_account_state.storage())?;
    insert_account_asset_vault(tx, new_account_state.vault())?;
    insert_account_record(tx, new_account_state, None)
}

pub(super) fn insert_account_record(
    tx: &Transaction<'_>,
    account: &Account,
    account_seed: Option<Word>,
) -> Result<(), StoreError> {
    let (id, code_root, storage_root, vault_root, nonce, committed) = serialize_account(account)?;

    let account_seed = account_seed.map(|seed| seed.to_bytes());

    const QUERY: &str =  "INSERT INTO accounts (id, code_root, storage_root, vault_root, nonce, committed, account_seed) VALUES (?, ?, ?, ?, ?, ?, ?)";
    tx.execute(
        QUERY,
        params![id, code_root, storage_root, vault_root, nonce, committed, account_seed],
    )?;
    Ok(())
}

/// Inserts an [AccountCode]
fn insert_account_code(tx: &Transaction<'_>, account_code: &AccountCode) -> Result<(), StoreError> {
    let (code_root, code, module) = serialize_account_code(account_code)?;
    const QUERY: &str =
        "INSERT OR IGNORE INTO account_code (root, procedures, module) VALUES (?, ?, ?)";
    tx.execute(QUERY, params![code_root, code, module,])?;
    Ok(())
}

/// Inserts an [AccountStorage]
pub(super) fn insert_account_storage(
    tx: &Transaction<'_>,
    account_storage: &AccountStorage,
) -> Result<(), StoreError> {
    let (storage_root, storage_slots) = serialize_account_storage(account_storage)?;
    const QUERY: &str = "INSERT OR IGNORE INTO account_storage (root, slots) VALUES (?, ?)";
    tx.execute(QUERY, params![storage_root, storage_slots])?;
    Ok(())
}

/// Inserts an [AssetVault]
pub(super) fn insert_account_asset_vault(
    tx: &Transaction<'_>,
    asset_vault: &AssetVault,
) -> Result<(), StoreError> {
    let (vault_root, assets) = serialize_account_asset_vault(asset_vault)?;
    const QUERY: &str = "INSERT OR IGNORE INTO account_vaults (root, assets) VALUES (?, ?)";
    tx.execute(QUERY, params![vault_root, assets])?;
    Ok(())
}

/// Inserts an [AuthInfo] for the account with id `account_id`
pub(super) fn insert_account_auth(
    tx: &Transaction<'_>,
    account_id: AccountId,
    auth_info: &AuthInfo,
) -> Result<(), StoreError> {
    let (account_id, auth_info) = serialize_account_auth(account_id, auth_info)?;
    const QUERY: &str = "INSERT INTO account_auth (account_id, auth_info) VALUES (?, ?)";
    tx.execute(QUERY, params![account_id, auth_info])?;
    Ok(())
}

/// Parse accounts colums from the provided row into native types
pub(super) fn parse_accounts_columns(
    row: &rusqlite::Row<'_>,
) -> Result<SerializedAccountsParts, rusqlite::Error> {
    let id: i64 = row.get(0)?;
    let nonce: i64 = row.get(1)?;
    let vault_root: String = row.get(2)?;
    let storage_root: String = row.get(3)?;
    let code_root: String = row.get(4)?;
    let account_seed: Option<Vec<u8>> = row.get(5)?;
    Ok((id, nonce, vault_root, storage_root, code_root, account_seed))
}

/// Parse an account from the provided parts.
pub(super) fn parse_accounts(
    serialized_account_parts: SerializedAccountsParts,
) -> Result<(AccountStub, Option<Word>), StoreError> {
    let (id, nonce, vault_root, storage_root, code_root, account_seed) = serialized_account_parts;
    let account_seed = account_seed.map(|seed| Word::read_from_bytes(&seed)).transpose()?;

    Ok((
        AccountStub::new(
            (id as u64)
                .try_into()
                .expect("Conversion from stored AccountID should not panic"),
            Felt::new(nonce as u64),
            serde_json::from_str(&vault_root).map_err(StoreError::JsonDataDeserializationError)?,
            Digest::try_from(&storage_root)?,
            Digest::try_from(&code_root)?,
        ),
        account_seed,
    ))
}

/// Serialized the provided account into database compatible types.
fn serialize_account(account: &Account) -> Result<SerializedAccountData, StoreError> {
    let id: u64 = account.id().into();
    let code_root = account.code().root().to_string();
    let storage_root = account.storage().root().to_string();
    let vault_root = serde_json::to_string(&account.vault().commitment())
        .map_err(StoreError::InputSerializationError)?;
    let committed = account.is_on_chain();
    let nonce = account.nonce().as_int() as i64;

    Ok((id as i64, code_root, storage_root, vault_root, nonce, committed))
}

/// Parse account_auth columns from the provided row into native types
fn parse_account_auth_columns(
    row: &rusqlite::Row<'_>,
) -> Result<SerializedAccountAuthParts, rusqlite::Error> {
    let account_id: i64 = row.get(0)?;
    let auth_info_bytes: Vec<u8> = row.get(1)?;
    Ok((account_id, auth_info_bytes))
}

/// Parse an `AuthInfo` from the provided parts.
fn parse_account_auth(
    serialized_account_auth_parts: SerializedAccountAuthParts,
) -> Result<AuthInfo, StoreError> {
    let (_, auth_info_bytes) = serialized_account_auth_parts;
    let auth_info = AuthInfo::read_from_bytes(&auth_info_bytes)?;
    Ok(auth_info)
}

/// Serialized the provided account_auth into database compatible types.
fn serialize_account_auth(
    account_id: AccountId,
    auth_info: &AuthInfo,
) -> Result<SerializedAccountAuthData, StoreError> {
    let account_id: u64 = account_id.into();
    let auth_info = auth_info.to_bytes();
    Ok((account_id as i64, auth_info))
}

/// Parse account_code columns from the provided row into native types.
fn parse_account_code_columns(
    row: &rusqlite::Row<'_>,
) -> Result<SerializedAccountCodeParts, rusqlite::Error> {
    let root: String = row.get(0)?;
    let procedures: String = row.get(1)?;
    let module: Vec<u8> = row.get(2)?;
    Ok((root, procedures, module))
}

/// Parse an account_code from the provided parts.
fn parse_account_code(
    serialized_account_code_parts: SerializedAccountCodeParts,
) -> Result<(Vec<Digest>, ModuleAst), StoreError> {
    let (_, procedures, module) = serialized_account_code_parts;

    let procedures =
        serde_json::from_str(&procedures).map_err(StoreError::JsonDataDeserializationError)?;
    let module = ModuleAst::from_bytes(&module)?;
    Ok((procedures, module))
}

/// Serialize the provided account_code into database compatible types.
fn serialize_account_code(
    account_code: &AccountCode,
) -> Result<SerializedAccountCodeData, StoreError> {
    let root = account_code.root().to_string();
    let procedures = serde_json::to_string(account_code.procedures())
        .map_err(StoreError::InputSerializationError)?;
    let module = account_code.module().to_bytes(AstSerdeOptions { serialize_imports: true });

    Ok((root, procedures, module))
}

/// Parse account_storage columns from the provided row into native types.
fn parse_account_storage_columns(
    row: &rusqlite::Row<'_>,
) -> Result<SerializedAccountStorageParts, rusqlite::Error> {
    let root: String = row.get(0)?;
    let storage: Vec<u8> = row.get(1)?;
    Ok((root, storage))
}

/// Parse an account_storage from the provided parts.
fn parse_account_storage(
    serialized_account_storage_parts: SerializedAccountStorageParts,
) -> Result<AccountStorage, StoreError> {
    let (_, storage) = serialized_account_storage_parts;

    let storage = AccountStorage::read_from_bytes(&storage)?;
    Ok(storage)
}

/// Serialize the provided account_storage into database compatible types.
fn serialize_account_storage(
    account_storage: &AccountStorage,
) -> Result<SerializedAccountStorageData, StoreError> {
    let root = account_storage.root().to_string();
    let storage = account_storage.to_bytes();

    Ok((root, storage))
}

/// Parse account_vault columns from the provided row into native types.
fn parse_account_asset_vault_columns(
    row: &rusqlite::Row<'_>,
) -> Result<SerializedAccountVaultParts, rusqlite::Error> {
    let root: String = row.get(0)?;
    let assets: String = row.get(1)?;
    Ok((root, assets))
}

/// Parse a vector of assets from the provided parts.
fn parse_account_asset_vault(
    serialized_account_asset_vault_parts: SerializedAccountVaultParts,
) -> Result<Vec<Asset>, StoreError> {
    let (_, assets) = serialized_account_asset_vault_parts;

    let assets = serde_json::from_str(&assets).map_err(StoreError::JsonDataDeserializationError)?;
    Ok(assets)
}

/// Serialize the provided asset_vault into database compatible types.
fn serialize_account_asset_vault(
    asset_vault: &AssetVault,
) -> Result<SerializedAccountVaultData, StoreError> {
    let root = serde_json::to_string(&asset_vault.commitment())
        .map_err(StoreError::InputSerializationError)?;
    let assets: Vec<Asset> = asset_vault.assets().collect();
    let assets = serde_json::to_string(&assets).map_err(StoreError::InputSerializationError)?;
    Ok((root, assets))
}

#[cfg(test)]
mod tests {
    use miden_objects::{
        accounts::{AccountCode, AccountId},
        assembly::ModuleAst,
        crypto::dsa::rpo_falcon512::SecretKey,
    };
    use miden_tx::utils::{Deserializable, Serializable};

    use super::{insert_account_auth, AuthInfo};
    use crate::{
        mock::DEFAULT_ACCOUNT_CODE,
        store::sqlite_store::{accounts::insert_account_code, tests::create_test_store},
    };

    #[test]
    fn test_account_code_insertion_no_duplicates() {
        let mut store = create_test_store();
        let assembler = miden_lib::transaction::TransactionKernel::assembler();
        let module_ast = ModuleAst::parse(DEFAULT_ACCOUNT_CODE).unwrap();
        let account_code = AccountCode::new(module_ast, &assembler).unwrap();
        let tx = store.db.transaction().unwrap();

        // Table is empty at the beginning
        let mut actual: usize =
            tx.query_row("SELECT Count(*) FROM account_code", [], |row| row.get(0)).unwrap();
        assert_eq!(actual, 0);

        // First insertion generates a new row
        insert_account_code(&tx, &account_code).unwrap();
        actual = tx.query_row("SELECT Count(*) FROM account_code", [], |row| row.get(0)).unwrap();
        assert_eq!(actual, 1);

        // Second insertion passes but does not generate a new row
        assert!(insert_account_code(&tx, &account_code).is_ok());
        actual = tx.query_row("SELECT Count(*) FROM account_code", [], |row| row.get(0)).unwrap();
        assert_eq!(actual, 1);
    }

    #[test]
    fn test_auth_info_serialization() {
        let exp_key_pair = SecretKey::new();
        let auth_info = AuthInfo::RpoFalcon512(exp_key_pair.clone());
        let bytes = auth_info.to_bytes();
        let actual = AuthInfo::read_from_bytes(&bytes).unwrap();
        match actual {
            AuthInfo::RpoFalcon512(act_key_pair) => {
                assert_eq!(exp_key_pair.to_bytes(), act_key_pair.to_bytes());
                assert_eq!(exp_key_pair.public_key(), act_key_pair.public_key());
            },
        }
    }

    #[test]
    fn test_auth_info_store() {
        let exp_key_pair = SecretKey::new();

        let mut store = create_test_store();

        let account_id = AccountId::try_from(3238098370154045919u64).unwrap();
        {
            let tx = store.db.transaction().unwrap();
            insert_account_auth(&tx, account_id, &AuthInfo::RpoFalcon512(exp_key_pair.clone()))
                .unwrap();
            tx.commit().unwrap();
        }

        let account_auth = store.get_account_auth(account_id).unwrap();
        match account_auth {
            AuthInfo::RpoFalcon512(act_key_pair) => {
                assert_eq!(exp_key_pair.to_bytes(), act_key_pair.to_bytes());
                assert_eq!(exp_key_pair.public_key(), act_key_pair.public_key());
            },
        }
    }
}
