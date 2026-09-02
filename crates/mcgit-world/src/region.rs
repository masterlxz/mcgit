use std::io::Cursor;

use fastanvil::Region;

use crate::types::{ChunkDiff, ChunkStatus, WorldError};

/// Extracts `(region_x, region_z)` from a region file's name, e.g.
/// `"r.-1.0.mca"` -> `Some((-1, 0))`. Returns `None` for anything that
/// doesn't match the `r.<x>.<z>.mca` pattern Minecraft itself uses.
pub fn parse_region_coords(filename: &str) -> Option<(i32, i32)> {
    let rest = filename.strip_prefix("r.")?.strip_suffix(".mca")?;
    let (x, z) = rest.split_once('.')?;
    Some((x.parse().ok()?, z.parse().ok()?))
}

/// Compares the 32×32 chunk slots between two versions (raw bytes) of the
/// same region file. `region_x`/`region_z` (from `parse_region_coords`)
/// convert each slot's local `0..32` coordinates into the chunk's absolute
/// position in the world. Comparison is by byte-equality of each chunk's
/// decompressed NBT — no attempt is made to understand what changed inside
/// a chunk, only whether it did.
pub fn diff_region_chunks(
    from_bytes: &[u8],
    to_bytes: &[u8],
    region_x: i32,
    region_z: i32,
) -> Result<Vec<ChunkDiff>, WorldError> {
    let mut from_region = Region::from_stream(Cursor::new(from_bytes))?;
    let mut to_region = Region::from_stream(Cursor::new(to_bytes))?;

    let mut diffs = Vec::new();
    for local_z in 0..32 {
        for local_x in 0..32 {
            let from_chunk = from_region.read_chunk(local_x, local_z)?;
            let to_chunk = to_region.read_chunk(local_x, local_z)?;

            let status = match (from_chunk, to_chunk) {
                (None, None) => continue,
                (None, Some(_)) => ChunkStatus::Added,
                (Some(_), None) => ChunkStatus::Removed,
                (Some(a), Some(b)) if a == b => continue,
                (Some(_), Some(_)) => ChunkStatus::Changed,
            };

            diffs.push(ChunkDiff {
                chunk_x: region_x * 32 + local_x as i32,
                chunk_z: region_z * 32 + local_z as i32,
                status,
            });
        }
    }

    Ok(diffs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn parse_region_coords_handles_negative_coordinates() {
        assert_eq!(parse_region_coords("r.-1.0.mca"), Some((-1, 0)));
        assert_eq!(parse_region_coords("r.0.-1.mca"), Some((0, -1)));
        assert_eq!(parse_region_coords("r.-2.-1.mca"), Some((-2, -1)));
        assert_eq!(parse_region_coords("r.3.5.mca"), Some((3, 5)));
    }

    #[test]
    fn parse_region_coords_rejects_non_matching_names() {
        assert_eq!(parse_region_coords("level.dat"), None);
        assert_eq!(parse_region_coords("r.0.0.mcc"), None);
        assert_eq!(parse_region_coords("r.abc.0.mca"), None);
    }

    fn minimal_chunk_nbt(marker: i64) -> Vec<u8> {
        let mut map = HashMap::new();
        map.insert("InhabitedTime".to_string(), fastnbt::Value::Long(marker));
        fastnbt::to_bytes(&fastnbt::Value::Compound(map)).unwrap()
    }

    /// Builds a brand-new region (via `Region::create`, the API meant
    /// exactly for this — writes a valid empty 8KB Anvil header itself),
    /// writes `chunks` (local x, local z, NBT marker) into it, and returns
    /// the raw bytes.
    fn build_region(chunks: &[(usize, usize, i64)]) -> Vec<u8> {
        let mut region = Region::create(Cursor::new(Vec::new())).unwrap();
        for &(x, z, marker) in chunks {
            region.write_chunk(x, z, &minimal_chunk_nbt(marker)).unwrap();
        }
        region.into_inner().unwrap().into_inner()
    }

    #[test]
    fn diff_region_chunks_reports_added_chunk() {
        let from_bytes = build_region(&[]);
        let to_bytes = build_region(&[(2, 3, 1)]);

        let diffs = diff_region_chunks(&from_bytes, &to_bytes, 0, 0).unwrap();

        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].chunk_x, 2);
        assert_eq!(diffs[0].chunk_z, 3);
        assert_eq!(diffs[0].status, ChunkStatus::Added);
    }

    #[test]
    fn diff_region_chunks_reports_changed_chunk_and_ignores_unchanged_ones() {
        let base_bytes = build_region(&[(0, 0, 1), (5, 5, 1)]);
        let changed_bytes = build_region(&[(0, 0, 2), (5, 5, 1)]);

        // Region (1, -2): world chunk coords = (1*32 + local, -2*32 + local).
        let diffs = diff_region_chunks(&base_bytes, &changed_bytes, 1, -2).unwrap();

        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].status, ChunkStatus::Changed);
        assert_eq!(diffs[0].chunk_x, 32);
        assert_eq!(diffs[0].chunk_z, -64);
    }

    #[test]
    fn diff_region_chunks_reports_removed_chunk() {
        let base_bytes = build_region(&[(10, 10, 1)]);
        let to_bytes = build_region(&[]);

        let diffs = diff_region_chunks(&base_bytes, &to_bytes, 0, 0).unwrap();

        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].chunk_x, 10);
        assert_eq!(diffs[0].chunk_z, 10);
        assert_eq!(diffs[0].status, ChunkStatus::Removed);
    }

    #[test]
    fn diff_region_chunks_between_identical_regions_returns_empty() {
        let bytes = build_region(&[(0, 0, 1)]);

        let diffs = diff_region_chunks(&bytes, &bytes, 0, 0).unwrap();

        assert!(diffs.is_empty());
    }
}
