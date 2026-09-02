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

/// Whether something newly appeared or newly disappeared between two
/// versions — shared by entity and structure diffs, where (unlike a block,
/// which has a stable position to compare in place) there's no third
/// "changed" state: an entity/structure instance either exists on a side or
/// it doesn't.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Presence {
    Added,
    Removed,
}

/// One entity (mob, dropped item, ...) that appeared or disappeared between
/// two versions of the same chunk's `entities/` data, identified by its
/// stable `UUID` — not position, since entities move, so an entity that
/// merely walked around shouldn't show up as removed-then-added. `id` is
/// the entity type (e.g. `"minecraft:sheep"`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityDiff {
    pub id: String,
    pub uuid: String,
    pub presence: Presence,
}

/// One generated structure (village, trial chamber, ...) that started or
/// stopped being recorded as starting in a chunk between two versions,
/// identified by its structure type id (e.g. `"minecraft:village_plains"`)
/// — see `count_chunk_structures` for why a `structures.starts` key is a
/// reliable one-instance-per-chunk identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructureDiff {
    pub id: String,
    pub presence: Presence,
}
