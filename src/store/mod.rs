use super::{errors::StoreError, AccountStub, ClientConfig};
use crypto::{utils::collections::BTreeMap, Word};
use objects::{
    accounts::{Account, AccountCode, AccountId, AccountStorage, AccountVault},
    assembly::AstSerdeOptions,
    assets::Asset,
    notes::{Note, NoteMetadata, RecordedNote},
    Digest, Felt,
};
use rusqlite::{params, Connection};

// mod migrations;
use entity::{account_code, account_keys, account_storage, account_vaults, accounts};
use migration::{Migrator, MigratorTrait};
use sea_orm::{
    ActiveModelTrait, Database, DatabaseConnection, DatabaseTransaction, EntityTrait, Set,
    TransactionTrait,
};

// TYPES
// ================================================================================================

type SerializedInputNoteData = (
    String,
    String,
    String,
    String,
    String,
    String,
    i64,
    i64,
    i64,
    String,
    String,
    String,
    i64,
);

type SerializedInputNoteParts = (String, String, String, String, u64, u64, u64, String);

// CLIENT STORE
// ================================================================================================

pub struct Store {
    db: DatabaseConnection,
}

impl Store {
    // CONSTRUCTORS
    // --------------------------------------------------------------------------------------------

    /// Returns a new instance of [Store] instantiated with the specified configuration options.
    pub async fn new(config: StoreConfig) -> Result<Self, StoreError> {
        let url = format!("sqlite://{}?mode=rwc", config.path);
        let db = Database::connect(&url)
            .await
            .expect("Failed to setup the database");
        Migrator::up(&db, None)
            .await
            .expect("Failed to run migrations for tests");
        Ok(Self { db })
    }

    // ACCOUNTS
    // --------------------------------------------------------------------------------------------

    pub async fn get_accounts(&self) -> Result<Vec<AccountStub>, StoreError> {
        Ok(accounts::Entity::find()
            .all(&self.db)
            .await
            .unwrap()
            .iter()
            .map(|account| {
                AccountStub::new(
                    (account.id as u64).try_into().unwrap(),
                    (account.nonce as u64).into(),
                    serde_json::from_str(&String::from_utf8(account.vault_root.clone()).unwrap())
                        .unwrap(),
                    serde_json::from_str(&String::from_utf8(account.storage_root.clone()).unwrap())
                        .unwrap(),
                    serde_json::from_str(&String::from_utf8(account.code_root.clone()).unwrap())
                        .unwrap(),
                )
            })
            .collect())
    }

    pub async fn insert_account_with_metadata(
        &mut self,
        account: &Account,
    ) -> Result<(), StoreError> {
        let tx: DatabaseTransaction = self.db.begin().await.unwrap();

        Self::insert_account_code(&tx, account.code()).await?;
        Self::insert_account_storage(&tx, account.storage()).await?;
        Self::insert_account_vault(&tx, account.vault()).await?;
        Self::insert_account(&tx, account).await?;

        tx.commit().await.map_err(StoreError::QueryError)?;
        Ok(())
    }

    pub async fn insert_account_code(
        tx: &DatabaseTransaction,
        account_code: &AccountCode,
    ) -> Result<(), StoreError> {
        let account_code = account_code::ActiveModel {
            root: Set(serde_json::to_string(&account_code.root())
                .unwrap()
                .into_bytes()),
            procedures: Set(serde_json::to_string(account_code.procedures())
                .unwrap()
                .into_bytes()),
            module: Set(account_code.module().to_bytes(AstSerdeOptions {
                serialize_imports: true,
            })),
        };
        let _account_code = account_code
            .insert(tx)
            .await
            .map_err(StoreError::QueryError)?;

        Ok(())
    }

    pub async fn insert_account_storage(
        tx: &DatabaseTransaction,
        account_storage: &AccountStorage,
    ) -> Result<(), StoreError> {
        let storage_slots: BTreeMap<u64, &Word> = account_storage.slots().leaves().collect();
        let storage_slots = serde_json::to_string(&storage_slots)
            .map_err(StoreError::InputSerializationError)
            .unwrap()
            .into_bytes();

        let account_storage = account_storage::ActiveModel {
            root: Set(serde_json::to_string(&account_storage.root())
                .unwrap()
                .into_bytes()),
            slots: Set(storage_slots),
        };

        let _account_storage = account_storage
            .insert(tx)
            .await
            .map_err(StoreError::QueryError)?;

        Ok(())
    }

    pub async fn insert_account_vault(
        tx: &DatabaseTransaction,
        account_vault: &AccountVault,
    ) -> Result<(), StoreError> {
        let assets: Vec<Asset> = account_vault.assets().collect();
        let assets = serde_json::to_string(&assets).map_err(StoreError::InputSerializationError)?;

        let account_vault = account_vaults::ActiveModel {
            root: Set(serde_json::to_string(&account_vault.commitment())
                .unwrap()
                .into_bytes()),
            assets: Set(serde_json::to_string(&assets).unwrap().into_bytes()),
        };

        let _account_vault = account_vault
            .insert(tx)
            .await
            .map_err(StoreError::QueryError)?;

        Ok(())
    }

    pub async fn insert_account(
        tx: &DatabaseTransaction,
        account: &Account,
    ) -> Result<(), StoreError> {
        let id: u64 = account.id().into();
        let account = accounts::ActiveModel {
            id: Set(id as i64),
            code_root: Set(serde_json::to_string(&account.code().root())
                .unwrap()
                .into_bytes()),
            storage_root: Set(serde_json::to_string(&account.storage().root())
                .unwrap()
                .into_bytes()),
            vault_root: Set(serde_json::to_string(&account.vault().commitment())
                .unwrap()
                .into_bytes()),
            nonce: Set(account.nonce().inner() as i64),
            committed: Set(account.is_on_chain()),
        };

        let _account = account.insert(tx).await.map_err(StoreError::QueryError)?;

        Ok(())
    }

    // NOTES
    // --------------------------------------------------------------------------------------------

    /// Retrieves the input notes from the database
    pub fn get_input_notes(&self) -> Result<Vec<RecordedNote>, StoreError> {
        todo!()
        // const QUERY: &str = "SELECT script, inputs, vault, serial_num, sender_id, tag, num_assets, inclusion_proof FROM input_notes";

        // self.db
        //     .prepare(QUERY)
        //     .map_err(StoreError::QueryError)?
        //     .query_map([], parse_input_note_columns)
        //     .expect("no binding parameters used in query")
        //     .map(|result| {
        //         result
        //             .map_err(StoreError::ColumnParsingError)
        //             .and_then(parse_input_note)
        //     })
        //     .collect::<Result<Vec<RecordedNote>, _>>()
    }

    /// Retrieves the input note with the specified hash from the database
    pub fn get_input_note_by_hash(&self, hash: Digest) -> Result<RecordedNote, StoreError> {
        todo!()
        // let query_hash =
        //     serde_json::to_string(&hash).map_err(StoreError::InputSerializationError)?;
        // const QUERY: &str = "SELECT script, inputs, vault, serial_num, sender_id, tag, num_assets, inclusion_proof FROM input_notes WHERE hash = ?";

        // self.db
        //     .prepare(QUERY)
        //     .map_err(StoreError::QueryError)?
        //     .query_map(params![query_hash.to_string()], parse_input_note_columns)
        //     .map_err(StoreError::QueryError)?
        //     .map(|result| {
        //         result
        //             .map_err(StoreError::ColumnParsingError)
        //             .and_then(parse_input_note)
        //     })
        //     .next()
        //     .ok_or(StoreError::InputNoteNotFound(hash))?
    }

    /// Inserts the provided input note into the database
    pub fn insert_input_note(&self, recorded_note: &RecordedNote) -> Result<(), StoreError> {
        todo!()
        // let (
        //     hash,
        //     nullifier,
        //     script,
        //     vault,
        //     inputs,
        //     serial_num,
        //     sender_id,
        //     tag,
        //     num_assets,
        //     inclusion_proof,
        //     recipients,
        //     status,
        //     commit_height,
        // ) = serialize_input_note(recorded_note)?;

        // const QUERY: &str = "\
        // INSERT INTO input_notes
        //     (hash, nullifier, script, vault, inputs, serial_num, sender_id, tag, num_assets, inclusion_proof, recipients, status, commit_height)
        //  VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";

        // self.db
        //     .execute(
        //         QUERY,
        //         params![
        //             hash,
        //             nullifier,
        //             script,
        //             vault,
        //             inputs,
        //             serial_num,
        //             sender_id,
        //             tag,
        //             num_assets,
        //             inclusion_proof,
        //             recipients,
        //             status,
        //             commit_height
        //         ],
        //     )
        //     .map_err(StoreError::QueryError)
        //     .map(|_| ())
    }
}

// STORE CONFIG
// ================================================================================================

pub struct StoreConfig {
    path: String,
}

impl From<&ClientConfig> for StoreConfig {
    fn from(config: &ClientConfig) -> Self {
        Self {
            path: config.store_path.clone(),
        }
    }
}

// HELPERS
// ================================================================================================
/// Parse input note columns from the provided row into native types.
fn parse_input_note_columns(
    row: &rusqlite::Row<'_>,
) -> Result<SerializedInputNoteParts, rusqlite::Error> {
    let script: String = row.get(0)?;
    let inputs: String = row.get(1)?;
    let vault: String = row.get(2)?;
    let serial_num: String = row.get(3)?;
    let sender_id = row.get::<usize, i64>(4)? as u64;
    let tag = row.get::<usize, i64>(5)? as u64;
    let num_assets = row.get::<usize, i64>(6)? as u64;
    let inclusion_proof: String = row.get(7)?;
    Ok((
        script,
        inputs,
        vault,
        serial_num,
        sender_id,
        tag,
        num_assets,
        inclusion_proof,
    ))
}

/// Parse a note from the provided parts.
fn parse_input_note(
    serialized_input_note_parts: SerializedInputNoteParts,
) -> Result<RecordedNote, StoreError> {
    let (script, inputs, vault, serial_num, sender_id, tag, num_assets, inclusion_proof) =
        serialized_input_note_parts;
    let script = serde_json::from_str(&script).map_err(StoreError::DataDeserializationError)?;
    let inputs = serde_json::from_str(&inputs).map_err(StoreError::DataDeserializationError)?;
    let vault = serde_json::from_str(&vault).map_err(StoreError::DataDeserializationError)?;
    let serial_num =
        serde_json::from_str(&serial_num).map_err(StoreError::DataDeserializationError)?;
    let note_metadata = NoteMetadata::new(
        AccountId::new_unchecked(Felt::new(sender_id)),
        Felt::new(tag),
        Felt::new(num_assets),
    );
    let note = Note::from_parts(script, inputs, vault, serial_num, note_metadata);

    let inclusion_proof =
        serde_json::from_str(&inclusion_proof).map_err(StoreError::DataDeserializationError)?;
    Ok(RecordedNote::new(note, inclusion_proof))
}

/// Serialize the provided input note into database compatible types.
fn serialize_input_note(
    recorded_note: &RecordedNote,
) -> Result<SerializedInputNoteData, StoreError> {
    let hash = serde_json::to_string(&recorded_note.note().hash())
        .map_err(StoreError::InputSerializationError)?;
    let nullifier = serde_json::to_string(&recorded_note.note().nullifier())
        .map_err(StoreError::InputSerializationError)?;
    let script = serde_json::to_string(&recorded_note.note().script())
        .map_err(StoreError::InputSerializationError)?;
    let vault = serde_json::to_string(&recorded_note.note().vault())
        .map_err(StoreError::InputSerializationError)?;
    let inputs = serde_json::to_string(&recorded_note.note().inputs())
        .map_err(StoreError::InputSerializationError)?;
    let serial_num = serde_json::to_string(&recorded_note.note().serial_num())
        .map_err(StoreError::InputSerializationError)?;
    let sender_id = u64::from(recorded_note.note().metadata().sender()) as i64;
    let tag = u64::from(recorded_note.note().metadata().tag()) as i64;
    let num_assets = u64::from(recorded_note.note().metadata().num_assets()) as i64;
    let inclusion_proof = serde_json::to_string(&recorded_note.proof())
        .map_err(StoreError::InputSerializationError)?;
    let recipients = serde_json::to_string(&recorded_note.note().metadata().tag())
        .map_err(StoreError::InputSerializationError)?;
    let status = String::from("committed");
    let commit_height = recorded_note.origin().block_num.inner() as i64;
    Ok((
        hash,
        nullifier,
        script,
        vault,
        inputs,
        serial_num,
        sender_id,
        tag,
        num_assets,
        inclusion_proof,
        recipients,
        status,
        commit_height,
    ))
}

// TESTS
// ================================================================================================

#[cfg(test)]
pub mod tests {
    use migration::{Migrator, MigratorTrait};
    use sea_orm::Database;
    use std::env::temp_dir;
    use uuid::Uuid;

    use miden_lib::assembler::assembler;
    use mock::mock::account;

    use super::Store;

    pub fn create_test_store_path() -> std::path::PathBuf {
        let mut temp_file = temp_dir();
        temp_file.push(format!("{}.sqlite3", Uuid::new_v4()));
        temp_file
    }

    async fn create_test_store() -> Store {
        let temp_file = create_test_store_path();
        let url = format!("sqlite://{}?mode=rwc", temp_file.to_string_lossy());
        let db = Database::connect(&url)
            .await
            .expect("Failed to setup the database");
        Migrator::up(&db, None)
            .await
            .expect("Failed to run migrations for tests");
        Store { db }
    }

    #[tokio::test]
    async fn insert_same_account_twice_fails() {
        let mut store = create_test_store().await;
        let assembler = assembler();
        let account = account::mock_new_account(&assembler);

        assert!(store.insert_account_with_metadata(&account).await.is_ok());
        assert!(store.insert_account_with_metadata(&account).await.is_err());
    }
}
