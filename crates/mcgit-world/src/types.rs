use thiserror::Error;

#[derive(Debug, Error)]
pub enum WorldError {
    #[error("could not read region data: {0}")]
    Anvil(#[from] fastanvil::Error),
}

/// Whether a chunk was added, removed, or changed between two versions of a
/// region file. Says nothing about *what* changed inside the chunk — that's
/// a future, deeper increment (block-states are packed and need real NBT
/// interpretation to decode).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkStatus {
    Added,
    Removed,
    Changed,
}

/// One chunk that differs between two versions of a region, addressed by
/// its absolute chunk coordinates in the world (not local 0..32 coordinates
/// within the region file).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkDiff {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub status: ChunkStatus,
}
