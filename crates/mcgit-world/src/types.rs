use thiserror::Error;

#[derive(Debug, Error)]
pub enum WorldError {
    #[error("could not read region data: {0}")]
    Anvil(#[from] fastanvil::Error),
    #[error("could not read chunk NBT: {0}")]
    Nbt(#[from] fastnbt::error::Error),
    #[error("unexpected chunk NBT shape: {0}")]
    Shape(String),
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

/// One block that differs between two versions of a chunk, addressed by its
/// absolute world position. `from`/`to` are full block identities (e.g.
/// `"minecraft:oak_leaves[distance=1,persistent=false]"`) — the block name
/// plus its properties, since two blocks with the same name but different
/// properties (a lit vs. unlit furnace, waterlogged vs. not) are genuinely
/// different blocks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockDiff {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub from: String,
    pub to: String,
}
