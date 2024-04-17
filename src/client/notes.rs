use miden_objects::{crypto::rand::FeltRng, notes::NoteId};

use super::{rpc::NodeRpcClient, Client};
use crate::{
    errors::{ClientError, StoreError},
    store::{InputNoteRecord, NoteFilter, OutputNoteRecord, Store},
};

impl<N: NodeRpcClient, R: FeltRng, S: Store> Client<N, R, S> {
    // INPUT NOTE DATA RETRIEVAL
    // --------------------------------------------------------------------------------------------

    /// Returns input notes managed by this client.
    pub fn get_input_notes(&self, filter: NoteFilter) -> Result<Vec<InputNoteRecord>, ClientError> {
        self.store.get_input_notes(filter).map_err(|err| err.into())
    }

    /// Returns the input note with the specified hash.
    pub fn get_input_note(&self, note_id: NoteId) -> Result<InputNoteRecord, ClientError> {
        self.store
            .get_input_notes(NoteFilter::Unique(note_id))
            .map_err(<StoreError as Into<ClientError>>::into)?
            .pop()
            .ok_or(ClientError::StoreError(StoreError::NoteNotFound(note_id)))
    }

    // OUTPUT NOTE DATA RETRIEVAL
    // --------------------------------------------------------------------------------------------

    /// Returns output notes managed by this client.
    pub fn get_output_notes(
        &self,
        filter: NoteFilter,
    ) -> Result<Vec<OutputNoteRecord>, ClientError> {
        self.store.get_output_notes(filter).map_err(|err| err.into())
    }

    /// Returns the output note with the specified hash.
    pub fn get_output_note(&self, note_id: NoteId) -> Result<OutputNoteRecord, ClientError> {
        self.store
            .get_output_notes(NoteFilter::Unique(note_id))?
            .pop()
            .ok_or(ClientError::StoreError(StoreError::NoteNotFound(note_id)))
    }

    // INPUT NOTE CREATION
    // --------------------------------------------------------------------------------------------

    /// Imports a new input note into the client's store.
    pub fn import_input_note(&mut self, note: InputNoteRecord) -> Result<(), ClientError> {
        self.store.insert_input_note(&note).map_err(|err| err.into())
    }
}
