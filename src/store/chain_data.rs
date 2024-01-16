use std::{collections::BTreeMap, num::NonZeroUsize};

use super::Store;

use crate::errors::StoreError;

use clap::error::Result;

use crypto::merkle::{InOrderIndex, MerklePath, MmrPeaks, PartialMmr};
use objects::{BlockHeader, Digest};
use rusqlite::{params, OptionalExtension, Transaction};

type SerializedBlockHeaderData = (i64, String, String, String, String);
type SerializedBlockHeaderParts = (u64, String, String, String, String);

type SerializedChainMmrNodeData = (i64, String);
type SerializedChainMmrNodeParts = (u64, String);

impl Store {
    // CHAIN DATA
    // --------------------------------------------------------------------------------------------
    pub fn insert_block_header(
        tx: &Transaction<'_>,
        block_header: BlockHeader,
        chain_mmr_peaks: MmrPeaks,
    ) -> Result<(), StoreError> {
        let chain_mmr_peaks = chain_mmr_peaks.peaks().to_vec();
        let (block_num, header, notes_root, sub_hash, chain_mmr) =
            serialize_block_header(block_header, chain_mmr_peaks)?;

        const QUERY: &str = "\
        INSERT INTO block_headers
            (block_num, header, notes_root, sub_hash, chain_mmr_peaks)
         VALUES (?, ?, ?, ?, ?)";

        tx.execute(
            QUERY,
            params![block_num, header, notes_root, sub_hash, chain_mmr],
        )
        .map_err(StoreError::QueryError)
        .map(|_| ())
    }

    pub fn get_block_header_by_num(&self, block_number: u32) -> Result<BlockHeader, StoreError> {
        const QUERY: &str = "SELECT block_num, header, notes_root, sub_hash, chain_mmr_peaks FROM block_headers WHERE block_num = ?";
        self.db
            .prepare(QUERY)
            .map_err(StoreError::QueryError)?
            .query_map(params![block_number as i64], parse_block_headers_columns)
            .map_err(StoreError::QueryError)?
            .map(|result| {
                result
                    .map_err(StoreError::ColumnParsingError)
                    .and_then(parse_block_header)
            })
            .next()
            .ok_or(StoreError::BlockHeaderNotFound(block_number))?
    }

    pub(crate) fn insert_chain_mmr_node(
        tx: &Transaction<'_>,
        id: InOrderIndex,
        node: Digest,
    ) -> Result<(), StoreError> {
        let (id, node) = serialize_chain_mmr_node(id, node)?;

        const QUERY: &str = "INSERT INTO chain_mmr_nodes (id, node) VALUES (?, ?)";

        tx.execute(QUERY, params![id, node])
            .map_err(StoreError::QueryError)
            .map(|_| ())
    }

    pub fn insert_chain_mmr_nodes(
        tx: &Transaction<'_>,
        nodes: Vec<(InOrderIndex, Digest)>,
    ) -> Result<(), StoreError> {
        for (index, node) in nodes {
            Self::insert_chain_mmr_node(tx, index, node)?;
        }

        Ok(())
    }

    /// Returns all nodes in the table.
    pub fn get_chain_mmr_nodes(&self) -> Result<BTreeMap<InOrderIndex, Digest>, StoreError> {
        const QUERY: &str = "SELECT id, node FROM chain_mmr_nodes";
        self.db
            .prepare(QUERY)
            .map_err(StoreError::QueryError)?
            .query_map(params![], parse_chain_mmr_nodes_columns)
            .map_err(StoreError::QueryError)?
            .map(|result| {
                result
                    .map_err(StoreError::ColumnParsingError)
                    .and_then(parse_chain_mmr_nodes)
            })
            .collect()
    }

    /// Returns peaks information from the blockchain by a specific block number.
    pub fn get_chain_mmr_peaks_by_block_num(&self, block_num: u32) -> Result<MmrPeaks, StoreError> {
        const QUERY: &str = "SELECT chain_mmr_peaks FROM block_headers WHERE block_num = ?";

        let mmr_peaks = self
            .db
            .prepare(QUERY)
            .map_err(StoreError::QueryError)?
            .query_row(params![block_num], |row| {
                let peaks: String = row.get(0)?;
                Ok(peaks)
            })
            .optional()
            .map_err(StoreError::QueryError)?;

        if let Some(mmr_peaks) = mmr_peaks {
            return parse_mmr_peaks(block_num + 1, mmr_peaks);
        }

        MmrPeaks::new(0, vec![]).map_err(StoreError::MmrError)
    }

    /// Given a set of input notes we want to consume at a certain block number (all notes belong
    /// to prior blocks), this builds a PartialMmr sufficient to prove the block inclusion of the
    /// notes corresponding blocks and a dictionary containing the tracked node numbers along with
    /// their hash.
    pub fn get_partial_mmr_for_blocks(
        &self,
        block_num: u32,
        blocks: &[(u32, objects::Digest)],
    ) -> Result<PartialMmr, StoreError> {
        let current_peaks = dbg!(self.get_chain_mmr_peaks_by_block_num(block_num)).unwrap();

        let mut partial_mmr = PartialMmr::from_peaks(current_peaks);
        let chain_mmr_authentication_nodes = dbg!(self.get_chain_mmr_nodes()).unwrap();

        dbg!(partial_mmr.forest());
        dbg!(block_num);

        for (block_number, node_hash) in blocks {
            let mut nodes = Vec::new();
            let mut idx = InOrderIndex::from_leaf_pos(*block_number as usize);

            while let Some(node) = chain_mmr_authentication_nodes.get(&idx.sibling()) {
                nodes.push(*node);
                idx = idx.parent();
            }
            dbg!(block_number);
            dbg!(partial_mmr
                .add(*block_number as usize, *node_hash, &MerklePath::new(nodes)))
                .map_err(StoreError::MmrError)?;
        }

        Ok(partial_mmr)
    }
}

// HELPERS
// ================================================================================================

fn parse_mmr_peaks(forest: u32, peaks_nodes: String) -> Result<MmrPeaks, StoreError> {
    let mmr_peaks_nodes: Vec<Digest> =
        serde_json::from_str(&peaks_nodes).map_err(StoreError::JsonDataDeserializationError)?;

    dbg!(&mmr_peaks_nodes);
    dbg!(forest);

    MmrPeaks::new(forest as usize, mmr_peaks_nodes).map_err(StoreError::MmrError)
}

fn serialize_block_header(
    block_header: BlockHeader,
    chain_mmr_peaks: Vec<Digest>,
) -> Result<SerializedBlockHeaderData, StoreError> {
    let block_num = block_header.block_num();
    let header =
        serde_json::to_string(&block_header).map_err(StoreError::InputSerializationError)?;
    let notes_root = serde_json::to_string(&block_header.note_root())
        .map_err(StoreError::InputSerializationError)?;
    let sub_hash = serde_json::to_string(&block_header.sub_hash())
        .map_err(StoreError::InputSerializationError)?;
    let chain_mmr_peaks =
        serde_json::to_string(&chain_mmr_peaks).map_err(StoreError::InputSerializationError)?;

    Ok((
        block_num as i64,
        header,
        notes_root,
        sub_hash,
        chain_mmr_peaks,
    ))
}

fn parse_block_headers_columns(
    row: &rusqlite::Row<'_>,
) -> Result<SerializedBlockHeaderParts, rusqlite::Error> {
    let block_num: i64 = row.get(0)?;
    let header: String = row.get(1)?;
    let notes_root: String = row.get(2)?;
    let sub_hash: String = row.get(3)?;
    let chain_mmr: String = row.get(4)?;

    Ok((block_num as u64, header, notes_root, sub_hash, chain_mmr))
}

fn parse_block_header(
    serialized_block_header_parts: SerializedBlockHeaderParts,
) -> Result<BlockHeader, StoreError> {
    let (_, header, _, _, _) = serialized_block_header_parts;

    serde_json::from_str(&header).map_err(StoreError::JsonDataDeserializationError)
}

fn serialize_chain_mmr_node(
    id: InOrderIndex,
    node: Digest,
) -> Result<SerializedChainMmrNodeData, StoreError> {
    let id: u64 = id.into();
    let node = serde_json::to_string(&node).map_err(StoreError::InputSerializationError)?;
    Ok((id as i64, node))
}

fn parse_chain_mmr_nodes_columns(
    row: &rusqlite::Row<'_>,
) -> Result<SerializedChainMmrNodeParts, rusqlite::Error> {
    let id: i64 = row.get(0)?;
    let node = row.get(1)?;
    Ok((id as u64, node))
}

fn parse_chain_mmr_nodes(
    serialized_chain_mmr_node_parts: SerializedChainMmrNodeParts,
) -> Result<(InOrderIndex, Digest), StoreError> {
    let (id, node) = serialized_chain_mmr_node_parts;

    let id = InOrderIndex::new(NonZeroUsize::new(id as usize).unwrap());
    let node: Digest =
        serde_json::from_str(&node).map_err(StoreError::JsonDataDeserializationError)?;
    Ok((id, node))
}
