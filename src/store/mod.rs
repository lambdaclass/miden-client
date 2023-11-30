mod models;
pub mod schema;

use super::{errors::StoreError, AccountStub, ClientConfig};
use crypto::{utils::collections::BTreeMap, Word};
use objects::{
    accounts::{Account, AccountCode, AccountId, AccountStorage, AccountVault},
    assembly::AstSerdeOptions,
    assets::Asset,
    notes::{Note, NoteMetadata, RecordedNote},
    Digest, Felt,
};
// use rusqlite::{params, Connection};
use rusqlite::params;

// from diesel guide, move to better place
use diesel::prelude::*;
use diesel::{sqlite::SqliteConnection, Connection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use miden_lib::*;
use models::*;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./src/store/migrations");

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
    db: SqliteConnection,
}

impl Store {
    // CONSTRUCTORS
    // --------------------------------------------------------------------------------------------

    /// Returns a new instance of [Store] instantiated with the specified configuration options.
    pub fn new(config: StoreConfig) -> Result<Self, StoreError> {
        let mut db: SqliteConnection = SqliteConnection::establish(&config.path).unwrap(); // TODO: handle error
        db.run_pending_migrations(MIGRATIONS).unwrap();

        Ok(Self { db })
    }

    // ACCOUNTS
    // --------------------------------------------------------------------------------------------

    pub fn get_accounts(&mut self) -> Result<Vec<AccountStub>, StoreError> {
        use schema::accounts::dsl::*;

        Ok(accounts
            .select(Accounts::as_select())
            .load(&mut self.db)
            .unwrap() // TODO: handle unwrap
            .iter()
            .map(|a| a.to_account_stub().unwrap()) // TODO: handle unwrap
            .collect())
    }

    pub fn insert_account_with_metadata(&mut self, account: &Account) -> Result<(), StoreError> {
        // make this atomic
        self.insert_account_code(account.code())?;
        self.insert_account_storage(account.storage())?;
        self.insert_account_vault(account.vault())?;
        self.insert_account(&account)?;

        Ok(())
    }

    pub fn insert_account(&mut self, account: &Account) -> Result<(), StoreError> {
        use schema::accounts;

        let account = NewAccount::from_account(account).unwrap();
        diesel::insert_into(accounts::table)
            .values(account)
            .returning(Accounts::as_returning())
            .get_result(&mut self.db)
            .map_err(StoreError::QueryError)?;

        Ok(())
    }

    pub fn insert_account_code(&mut self, account_code: &AccountCode) -> Result<(), StoreError> {
        let new_account_code = NewAccountCode::from_account_code(account_code).unwrap();
        diesel::insert_into(schema::account_code::table)
            .values(new_account_code)
            .execute(&mut self.db)
            .map_err(StoreError::QueryError)?;

        Ok(())
    }

    pub fn insert_account_storage(
        &mut self,
        account_storage: &AccountStorage,
    ) -> Result<(), StoreError> {
        let new_account_storage = NewAccountStorage::from_account_storage(account_storage).unwrap();
        diesel::insert_into(schema::account_storage::table)
            .values(new_account_storage)
            .execute(&mut self.db)
            .map_err(StoreError::QueryError)?;

        Ok(())
    }

    pub fn insert_account_vault(&mut self, account_vault: &AccountVault) -> Result<(), StoreError> {
        let new_account_vault = NewAccountVault::from_account_vault(account_vault).unwrap();
        diesel::insert_into(schema::account_vaults::table)
            .values(new_account_vault)
            .execute(&mut self.db)
            .map_err(StoreError::QueryError)?;

        Ok(())
    }

    // NOTES
    // --------------------------------------------------------------------------------------------

    /// Retrieves the input notes from the database
    pub fn get_input_notes(&self) -> Result<Vec<RecordedNote>, StoreError> {
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
        todo!()
    }

    /// Retrieves the input note with the specified hash from the database
    pub fn get_input_note_by_hash(&self, _hash: Digest) -> Result<RecordedNote, StoreError> {
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
        todo!()
    }

    /// Inserts the provided input note into the database
    pub fn insert_input_note(&self, _recorded_note: &RecordedNote) -> Result<(), StoreError> {
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
        todo!()
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
    use diesel::{Connection, SqliteConnection};
    use miden_lib::assembler::assembler;
    use mock::mock::account;
    use std::env::temp_dir;
    use uuid::Uuid;

    use super::{Store, MIGRATIONS};
    use diesel_migrations::MigrationHarness;

    pub fn create_test_store_path() -> std::path::PathBuf {
        let mut temp_file = temp_dir();
        temp_file.push(format!("{}.sqlite3", Uuid::new_v4()));
        temp_file
    }

    fn create_test_store() -> Store {
        let temp_file = create_test_store_path();
        let mut db = SqliteConnection::establish(temp_file.to_str().unwrap()).unwrap();
        db.run_pending_migrations(MIGRATIONS).unwrap();

        Store { db }
    }

    #[test]
    pub fn insert_same_account_twice_fails() {
        let mut store = create_test_store();
        let assembler = assembler();
        let account = account::mock_new_account(&assembler);

        assert!(store.insert_account_with_metadata(&account).is_ok());
        assert!(store.insert_account_with_metadata(&account).is_err());
    }
}
