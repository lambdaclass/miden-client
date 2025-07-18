use miden_objects::note::NoteId as NativeNoteId;
use wasm_bindgen::prelude::*;

use super::word::Word;

#[derive(Clone)]
#[wasm_bindgen]
pub struct NoteId(NativeNoteId);

#[wasm_bindgen]
impl NoteId {
    #[wasm_bindgen(constructor)]
    pub fn new(recipient_digest: &Word, asset_commitment_digest: &Word) -> NoteId {
        NoteId(NativeNoteId::new(recipient_digest.into(), asset_commitment_digest.into()))
    }

    #[wasm_bindgen(js_name = "toString")]
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeNoteId> for NoteId {
    fn from(native_note_id: NativeNoteId) -> Self {
        NoteId(native_note_id)
    }
}

impl From<&NativeNoteId> for NoteId {
    fn from(native_note_id: &NativeNoteId) -> Self {
        NoteId(*native_note_id)
    }
}

impl From<NoteId> for NativeNoteId {
    fn from(note_id: NoteId) -> Self {
        note_id.0
    }
}

impl From<&NoteId> for NativeNoteId {
    fn from(note_id: &NoteId) -> Self {
        note_id.0
    }
}
