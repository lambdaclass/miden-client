use crypto::{
    merkle::{MmrPeaks, PartialMmr},
    utils::Serializable,
};
use miden_node_proto::{mmr::MmrDelta, responses::AccountHashUpdate};

use objects::{notes::NoteInclusionProof, BlockHeader, Digest};
use rusqlite::params;

use crate::{
    errors::StoreError,
    store::accounts::{parse_accounts, parse_accounts_columns},
};

use super::Store;

impl Store {
    // STATE SYNC
    // --------------------------------------------------------------------------------------------

    /// Returns the note tags that the client is interested in.
    pub fn get_note_tags(&self) -> Result<Vec<u64>, StoreError> {
        const QUERY: &str = "SELECT tags FROM state_sync";

        self.db
            .prepare(QUERY)
            .map_err(StoreError::QueryError)?
            .query_map([], |row| row.get(0))
            .expect("no binding parameters used in query")
            .map(|result| {
                result
                    .map_err(StoreError::ColumnParsingError)
                    .and_then(|v: String| {
                        serde_json::from_str(&v).map_err(StoreError::JsonDataDeserializationError)
                    })
            })
            .next()
            .expect("state sync tags exist")
    }

    /// Adds a note tag to the list of tags that the client is interested in.
    pub fn add_note_tag(&mut self, tag: u64) -> Result<bool, StoreError> {
        let mut tags = self.get_note_tags()?;
        if tags.contains(&tag) {
            return Ok(false);
        }
        tags.push(tag);
        let tags = serde_json::to_string(&tags).map_err(StoreError::InputSerializationError)?;

        const QUERY: &str = "UPDATE state_sync SET tags = ?";
        self.db
            .execute(QUERY, params![tags])
            .map_err(StoreError::QueryError)
            .map(|_| ())?;

        Ok(true)
    }

    /// Returns the block number of the last state sync block
    pub fn get_latest_block_num(&self) -> Result<u32, StoreError> {
        const QUERY: &str = "SELECT block_num FROM state_sync";

        self.db
            .prepare(QUERY)
            .map_err(StoreError::QueryError)?
            .query_map([], |row| row.get(0))
            .expect("no binding parameters used in query")
            .map(|result| {
                result
                    .map_err(StoreError::ColumnParsingError)
                    .map(|v: i64| v as u32)
            })
            .next()
            .expect("state sync block number exists")
    }

    pub fn apply_state_sync(
        &mut self,
        current_block_num: u32,
        block_header: BlockHeader,
        nullifiers: Vec<Digest>,
        account_updates: Vec<AccountHashUpdate>,
        mmr_delta: Option<MmrDelta>,
        committed_notes: Vec<(Digest, NoteInclusionProof)>,
    ) -> Result<(), StoreError> {
        // get current nodes on table
        // we need to do this here because creating a sql tx borrows a mut reference
        let current_peaks = self.get_chain_mmr_peaks_by_block_num(current_block_num)?;

        let tx = self
            .db
            .transaction()
            .map_err(StoreError::TransactionError)?;

        // Check if the returned account hashes match latest account hashes in the database
        for account_update in account_updates {
            if let (Some(account_id), Some(account_hash)) =
                (account_update.account_id, account_update.account_hash)
            {
                let account_id_int: u64 = account_id.clone().into();
                const ACCOUNT_HASH_QUERY: &str = "SELECT hash FROM accounts WHERE id = ?";

                if let Some(Ok((acc_stub, _acc_seed))) = tx
                    .prepare(ACCOUNT_HASH_QUERY)
                    .map_err(StoreError::QueryError)?
                    .query_map(params![account_id_int as i64], parse_accounts_columns)
                    .map_err(StoreError::QueryError)?
                    .map(|result| {
                        result
                            .map_err(StoreError::ColumnParsingError)
                            .and_then(parse_accounts)
                    })
                    .next()
                {
                    if account_hash != acc_stub.hash().into() {
                        return Err(StoreError::AccountHashMismatch(
                            account_id.try_into().unwrap(),
                        ));
                    }
                }
            }
        }

        // update state sync block number
        const BLOCK_NUMBER_QUERY: &str = "UPDATE state_sync SET block_num = ?";
        tx.execute(BLOCK_NUMBER_QUERY, params![block_header.block_num()])
            .map_err(StoreError::QueryError)?;

        // update spent notes
        for nullifier in nullifiers {
            const SPENT_QUERY: &str =
                "UPDATE input_notes SET status = 'consumed' WHERE nullifier = ?";
            let nullifier = nullifier.to_string();
            tx.execute(SPENT_QUERY, params![nullifier])
                .map_err(StoreError::QueryError)?;
        }

        // update chain mmr nodes on the table
        // get all elements from the chain mmr table
        if let Some(mmr_delta) = mmr_delta {
            // build partial mmr from the nodes - partial_mmr should be on memory as part of our store

            let mut partial_mmr: PartialMmr = if current_block_num == 0 {
                // first block we receive so we are good to create a blank partial mmr for this
                MmrPeaks::new(0, vec![])
                    .map_err(StoreError::MmrError)?
                    .into()
            } else {
                dbg!(PartialMmr::from_peaks(current_peaks))
            };

            // apply the delta
            let mmr_delta: crypto::merkle::MmrDelta = mmr_delta.try_into().unwrap();

            let new_authentication_nodes =
                partial_mmr.apply(mmr_delta).map_err(StoreError::MmrError)?;

            Store::insert_chain_mmr_nodes(&tx, new_authentication_nodes)?;

            Store::insert_block_header(&tx, block_header, dbg!(partial_mmr.peaks()))?;
        }

        // update tracked notes
        for (note_id, inclusion_proof) in committed_notes {
            const SPENT_QUERY: &str =
                "UPDATE input_notes SET status = 'committed', inclusion_proof = ? WHERE note_id = ?";

            let inclusion_proof = Some(inclusion_proof.to_bytes());
            tx.execute(SPENT_QUERY, params![inclusion_proof, note_id.to_string()])
                .map_err(StoreError::QueryError)?;
        }

        // TODO: We would need to mark transactions as committed here as well

        // commit the updates
        tx.commit().map_err(StoreError::QueryError)?;

        Ok(())
    }
}
