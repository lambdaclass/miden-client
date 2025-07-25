use miden_client::note::{
    NoteExecutionHint as NativeNoteExecutionHint, NoteInputs as NativeNoteInputs,
    NoteMetadata as NativeNoteMetadata, NoteRecipient as NativeNoteRecipient,
    NoteTag as NativeNoteTag, WellKnownNote as NativeWellKnownNote,
};
use miden_lib::note::utils;
use miden_objects::note::Note as NativeNote;
use wasm_bindgen::prelude::*;

use super::{
    account_id::AccountId, felt::Felt, note_assets::NoteAssets, note_id::NoteId,
    note_metadata::NoteMetadata, note_recipient::NoteRecipient, note_type::NoteType, word::Word,
};

#[wasm_bindgen]
#[derive(Clone)]
pub struct Note(NativeNote);

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub enum WellKnownNote {
    P2ID,
    P2IDE,
    SWAP,
}

#[wasm_bindgen]
impl Note {
    #[wasm_bindgen(constructor)]
    pub fn new(
        note_assets: &NoteAssets,
        note_metadata: &NoteMetadata,
        note_recipient: &NoteRecipient,
    ) -> Note {
        Note(NativeNote::new(note_assets.into(), note_metadata.into(), note_recipient.into()))
    }

    pub fn id(&self) -> NoteId {
        self.0.id().into()
    }

    pub fn metadata(&self) -> NoteMetadata {
        (*self.0.metadata()).into()
    }

    pub fn recipient(&self) -> NoteRecipient {
        self.0.recipient().clone().into()
    }

    pub fn assets(&self) -> NoteAssets {
        self.0.assets().clone().into()
    }

    #[wasm_bindgen(js_name = "createP2IDNote")]
    pub fn create_p2id_note(
        sender: &AccountId,
        target: &AccountId,
        assets: &NoteAssets,
        note_type: NoteType,
        serial_num: &Word,
        aux: &Felt,
    ) -> Self {
        let recipient = utils::build_p2id_recipient(target.into(), serial_num.into()).unwrap();
        let tag = NativeNoteTag::from_account_id(target.into());

        let metadata = NativeNoteMetadata::new(
            sender.into(),
            note_type.into(),
            tag,
            NativeNoteExecutionHint::always(),
            (*aux).into(),
        )
        .expect("Should not fail due to valid tag");

        NativeNote::new(assets.into(), metadata, recipient).into()
    }

    #[wasm_bindgen(js_name = "createP2IDENote")]
    pub fn create_p2ide_note(
        sender: &AccountId,
        target: &AccountId,
        assets: &NoteAssets,
        note_type: NoteType,
        serial_num: &Word,
        recall_height: u32,
        aux: &Felt,
    ) -> Self {
        let note_script = NativeWellKnownNote::P2IDE.script();

        let inputs = NativeNoteInputs::new(vec![
            target.suffix().into(),
            target.prefix().into(),
            recall_height.into(),
        ])
        .unwrap();

        let recipient = NativeNoteRecipient::new(serial_num.into(), note_script, inputs);

        let tag = NativeNoteTag::from_account_id(target.into());

        let metadata = NativeNoteMetadata::new(
            sender.into(),
            note_type.into(),
            tag,
            NativeNoteExecutionHint::always(),
            (*aux).into(),
        )
        .expect("Should not fail due to valid tag");

        NativeNote::new(assets.into(), metadata, recipient).into()
    }
    #[wasm_bindgen(js_name = "createWellKnownNote")]
    pub fn create_well_known_note(
        sender: &AccountId,
        target: &AccountId,
        assets: &NoteAssets,
        note_type: NoteType,
        serial_num: &Word,
        recall_height: u32,
        aux: &Felt,
        well_known_type: WellKnownNote,
    ) -> Self {
        let known_type: NativeWellKnownNote = well_known_type.into();
        let note_script = known_type.script();

        let inputs = NativeNoteInputs::new(vec![
            target.suffix().into(),
            target.prefix().into(),
            recall_height.into(),
        ])
        .unwrap();

        let recipient = NativeNoteRecipient::new(serial_num.into(), note_script, inputs);
        let tag = NativeNoteTag::from_account_id(target.into());

        let metadata = NativeNoteMetadata::new(
            sender.into(),
            note_type.into(),
            tag,
            NativeNoteExecutionHint::always(),
            (*aux).into(),
        )
        .expect("Should not fail due to valid tag");

        NativeNote::new(assets.into(), metadata, recipient).into()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeNote> for Note {
    fn from(note: NativeNote) -> Self {
        Note(note)
    }
}

impl From<&NativeNote> for Note {
    fn from(note: &NativeNote) -> Self {
        Note(note.clone())
    }
}

impl From<Note> for NativeNote {
    fn from(note: Note) -> Self {
        note.0
    }
}

impl From<&Note> for NativeNote {
    fn from(note: &Note) -> Self {
        note.0.clone()
    }
}

impl From<WellKnownNote> for NativeWellKnownNote {
    fn from(value: WellKnownNote) -> Self {
        match value {
            WellKnownNote::P2ID => NativeWellKnownNote::P2ID,
            WellKnownNote::P2IDE => NativeWellKnownNote::P2IDE,
            WellKnownNote::SWAP => NativeWellKnownNote::SWAP,
        }
    }
}
