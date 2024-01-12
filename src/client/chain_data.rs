use super::Client;

#[cfg(test)]
use crate::errors::ClientError;
#[cfg(test)]
use objects::{BlockHeader, Digest};
#[cfg(test)]
use crypto::merkle::PartialMmr;

impl Client {
    #[cfg(test)]
    pub fn get_block_headers(
        &self,
        start: u32,
        finish: u32,
    ) -> Result<Vec<BlockHeader>, ClientError> {
        let mut headers = Vec::new();
        for block_number in start..=finish {
            if let Ok(block_header) = self.store.get_block_header_by_num(block_number) {
                headers.push(block_header)
            }
        }

        Ok(headers)
    }

    #[cfg(test)]
    pub fn build_partial_mmr_from_client_state(&self, block_number: u32, blocks: &[(u32, Digest)]) -> PartialMmr {
        self.store.get_partial_mmr_for_blocks(block_number, blocks).unwrap()
    } 
}
