// This file is @generated by prost-build.
/// Represents a Merkle path.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MerklePath {
    /// List of sibling node hashes, in order from the root to the leaf.
    #[prost(message, repeated, tag = "1")]
    pub siblings: ::prost::alloc::vec::Vec<super::digest::Digest>,
}
