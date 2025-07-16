use alloc::{string::String, vec::Vec};

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{js_sys, wasm_bindgen};

use crate::{
    store::web_store::note::utils::{SerializedInputNoteData, SerializedOutputNoteData},
    sync::StateSyncUpdate,
};

use super::flattened_vec::FlattenedU8Vec;

// Sync IndexedDB Operations
#[wasm_bindgen(module = "/src/store/web_store/js/sync.js")]

extern "C" {
    // GETS
    // ================================================================================================

    #[wasm_bindgen(js_name = getSyncHeight)]
    pub fn idxdb_get_sync_height() -> js_sys::Promise;

    #[wasm_bindgen(js_name = getNoteTags)]
    pub fn idxdb_get_note_tags() -> js_sys::Promise;

    // INSERTS
    // ================================================================================================

    #[wasm_bindgen(js_name = addNoteTag)]
    pub fn idxdb_add_note_tag(
        tag: Vec<u8>,
        source_note_id: Option<String>,
        source_account_id: Option<String>,
    ) -> js_sys::Promise;

    #[wasm_bindgen(js_name = applyStateSync)]
    pub fn idxdb_apply_state_sync(state_update: JsStateSyncUpdate) -> js_sys::Promise;

    // DELETES
    // ================================================================================================
    #[wasm_bindgen(js_name = removeNoteTag)]
    pub fn idxdb_remove_note_tag(
        tag: Vec<u8>,
        source_note_id: Option<String>,
        source_account_id: Option<String>,
    ) -> js_sys::Promise;

    #[wasm_bindgen(js_name = discardTransactions)]
    pub fn idxdb_discard_transactions(transactions: Vec<String>) -> js_sys::Promise;

    #[wasm_bindgen(js_name = receiveStateSync)]
    pub fn idxdb_receive_state_sync(state_update: JsStateSyncUpdate) -> js_sys::Promise;
}

#[wasm_bindgen(getter_with_clone)]
pub struct JsStateSyncUpdate {
    pub block_num: String,
    pub flattened_new_block_headers: FlattenedU8Vec,
    pub new_block_nums: Vec<String>,
    pub flattened_partial_blockchain_peaks: FlattenedU8Vec,
    pub block_has_relevant_notes: Vec<u8>,
    pub serialized_node_ids: Vec<String>,
    pub serialized_nodes: Vec<String>,
    pub note_tags_to_remove_as_str: Vec<String>,
    pub serialized_input_notes: Vec<SerializedInputNoteData>,
    pub serialized_output_notes: Vec<SerializedOutputNoteData>,
}
