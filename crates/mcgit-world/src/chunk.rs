use std::collections::HashMap;

use fastnbt::Value;

use crate::types::{BlockDiff, WorldError};

const BLOCKS_PER_SECTION: usize = 16 * 16 * 16;

/// Smallest number of bits needed to represent `n` distinct values
/// (`ceil(log2(n))`), e.g. 18 values -> 5 bits (2^4 = 16 is too few).
fn bits_needed(n: usize) -> usize {
    if n <= 1 {
        return 0;
    }
    (usize::BITS - (n - 1).leading_zeros()) as usize
}

/// A palette entry's full identity: its block name plus, if present, its
/// properties (e.g. `facing`, `waterlogged`) serialized in a stable key
/// order — two entries can share a `Name` but differ in `Properties` (two
/// leaf variants, a lit vs. unlit furnace), so both are needed to tell two
/// blocks apart.
fn block_identity(entry: &Value) -> Result<String, WorldError> {
    let Value::Compound(entry) = entry else {
        return Err(WorldError::Shape("palette entry is not a compound".into()));
    };
    let Some(Value::String(name)) = entry.get("Name") else {
        return Err(WorldError::Shape("palette entry missing Name".into()));
    };

    let Some(Value::Compound(properties)) = entry.get("Properties") else {
        return Ok(name.clone());
    };

    let mut pairs: Vec<(String, String)> = properties
        .iter()
        .map(|(k, v)| (k.clone(), property_value(v)))
        .collect();
    pairs.sort();

    let joined = pairs
        .into_iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join(",");
    Ok(format!("{name}[{joined}]"))
}

fn property_value(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        other => format!("{other:?}"),
    }
}

/// A palette entry's bare block name, ignoring `Properties` — unlike
/// `block_identity`, used where properties would just be noise (counting
/// "how many stone blocks", not diffing exact variants).
fn block_name(entry: &Value) -> Result<&str, WorldError> {
    let Value::Compound(entry) = entry else {
        return Err(WorldError::Shape("palette entry is not a compound".into()));
    };
    let Some(Value::String(name)) = entry.get("Name") else {
        return Err(WorldError::Shape("palette entry missing Name".into()));
    };
    Ok(name.as_str())
}

/// Decodes one section's `block_states` (palette + bit-packed indices) into
/// the 4096 block identities it holds, in Minecraft's own layout order:
/// `index = local_y*256 + local_z*16 + local_x` (y outermost, then z, x).
fn decode_section_blocks(block_states: &HashMap<String, Value>) -> Result<Vec<String>, WorldError> {
    let Some(Value::List(palette)) = block_states.get("palette") else {
        return Err(WorldError::Shape(
            "block_states.palette missing or not a list".into(),
        ));
    };
    let identities: Vec<String> = palette.iter().map(block_identity).collect::<Result<_, _>>()?;

    if identities.len() <= 1 {
        // Single-entry palette: Minecraft skips storing indices entirely —
        // every block in the section is this one type (confirmed live: an
        // all-air section has palette len 1 and a 0-length `data`).
        let identity = identities.first().cloned().unwrap_or_else(|| "minecraft:air".to_string());
        return Ok(vec![identity; BLOCKS_PER_SECTION]);
    }

    let Some(Value::LongArray(data)) = block_states.get("data") else {
        return Err(WorldError::Shape(
            "block_states.data missing for a multi-entry palette".into(),
        ));
    };

    let bits_per_block = bits_needed(identities.len()).max(4);
    let entries_per_long = 64 / bits_per_block;
    let mask = (1u64 << bits_per_block) - 1;

    let mut out = Vec::with_capacity(BLOCKS_PER_SECTION);
    for &long in data.iter() {
        let long = long as u64;
        for i in 0..entries_per_long {
            if out.len() == BLOCKS_PER_SECTION {
                break;
            }
            let index = ((long >> (i * bits_per_block)) & mask) as usize;
            let identity = identities.get(index).ok_or_else(|| {
                WorldError::Shape(format!(
                    "palette index {index} out of range ({} entries)",
                    identities.len()
                ))
            })?;
            out.push(identity.clone());
        }
    }

    if out.len() != BLOCKS_PER_SECTION {
        return Err(WorldError::Shape(format!(
            "decoded {} blocks from block_states.data, expected {BLOCKS_PER_SECTION}",
            out.len()
        )));
    }

    Ok(out)
}

/// Parses a chunk's raw NBT and decodes every section's block data, keyed by
/// the section's absolute world Y (e.g. `-4` for the bottom of a 1.21.x
/// world's height range). A section with no `block_states` at all decodes as
/// pure air, matching what the game itself means by that absence.
fn decode_chunk_sections(nbt_bytes: &[u8]) -> Result<HashMap<i8, Vec<String>>, WorldError> {
    let chunk: Value = fastnbt::from_bytes(nbt_bytes)?;
    let Value::Compound(chunk) = chunk else {
        return Err(WorldError::Shape("chunk root is not a compound".into()));
    };
    let Some(Value::List(sections)) = chunk.get("sections") else {
        return Err(WorldError::Shape("chunk has no \"sections\" list".into()));
    };

    let mut out = HashMap::new();
    for section in sections {
        let Value::Compound(section) = section else {
            continue;
        };
        let Some(Value::Byte(y)) = section.get("Y") else {
            continue;
        };
        let blocks = match section.get("block_states") {
            Some(Value::Compound(block_states)) => decode_section_blocks(block_states)?,
            _ => vec!["minecraft:air".to_string(); BLOCKS_PER_SECTION],
        };
        out.insert(*y, blocks);
    }

    Ok(out)
}

/// Tallies one section's blocks by bare name into `counts`. Unlike
/// `decode_section_blocks`, this never materializes the 4096 per-position
/// identities — it decodes each packed index only to bump a per-palette-slot
/// counter, then folds those slot counts into `counts` by name at the end.
/// Cheap even for a large world, since a section's palette is usually tiny
/// (tens of entries) despite holding 4096 blocks.
fn count_section_blocks(
    block_states: &HashMap<String, Value>,
    counts: &mut HashMap<String, u64>,
) -> Result<(), WorldError> {
    let Some(Value::List(palette)) = block_states.get("palette") else {
        return Err(WorldError::Shape(
            "block_states.palette missing or not a list".into(),
        ));
    };
    let names: Vec<&str> = palette.iter().map(|entry| block_name(entry)).collect::<Result<_, _>>()?;

    if names.len() <= 1 {
        let name = names.first().copied().unwrap_or("minecraft:air");
        *counts.entry(name.to_string()).or_insert(0) += BLOCKS_PER_SECTION as u64;
        return Ok(());
    }

    let Some(Value::LongArray(data)) = block_states.get("data") else {
        return Err(WorldError::Shape(
            "block_states.data missing for a multi-entry palette".into(),
        ));
    };

    let bits_per_block = bits_needed(names.len()).max(4);
    let entries_per_long = 64 / bits_per_block;
    let mask = (1u64 << bits_per_block) - 1;

    let mut per_index = vec![0u64; names.len()];
    let mut seen = 0usize;
    for &long in data.iter() {
        if seen == BLOCKS_PER_SECTION {
            break;
        }
        let long = long as u64;
        for i in 0..entries_per_long {
            if seen == BLOCKS_PER_SECTION {
                break;
            }
            let index = ((long >> (i * bits_per_block)) & mask) as usize;
            let slot = per_index.get_mut(index).ok_or_else(|| {
                WorldError::Shape(format!(
                    "palette index {index} out of range ({} entries)",
                    names.len()
                ))
            })?;
            *slot += 1;
            seen += 1;
        }
    }

    if seen != BLOCKS_PER_SECTION {
        return Err(WorldError::Shape(format!(
            "counted {seen} blocks from block_states.data, expected {BLOCKS_PER_SECTION}"
        )));
    }

    for (name, count) in names.into_iter().zip(per_index) {
        if count > 0 {
            *counts.entry(name.to_string()).or_insert(0) += count;
        }
    }
    Ok(())
}

/// Tallies every section of a chunk's block data by bare name into `counts`.
pub(crate) fn count_chunk_blocks(nbt_bytes: &[u8], counts: &mut HashMap<String, u64>) -> Result<(), WorldError> {
    let chunk: Value = fastnbt::from_bytes(nbt_bytes)?;
    let Value::Compound(chunk) = chunk else {
        return Err(WorldError::Shape("chunk root is not a compound".into()));
    };
    let Some(Value::List(sections)) = chunk.get("sections") else {
        return Err(WorldError::Shape("chunk has no \"sections\" list".into()));
    };

    for section in sections {
        let Value::Compound(section) = section else {
            continue;
        };
        match section.get("block_states") {
            Some(Value::Compound(block_states)) => count_section_blocks(block_states, counts)?,
            _ => {
                *counts.entry("minecraft:air".to_string()).or_insert(0) += BLOCKS_PER_SECTION as u64;
            }
        }
    }

    Ok(())
}

/// Tallies generated structures (villages, trial chambers, ...) in a
/// `region/` chunk by type, into `counts`. Each structure instance is
/// recorded once, in the single chunk where it started generating —
/// `structures.starts` — never in the other chunks it merely spans (those
/// only carry a back-reference to the start), so summing `starts` across
/// every chunk in the world already gives an accurate per-type count with
/// no double-counting. Missing `structures`/`starts` (should be rare, but
/// not guaranteed on every chunk) is treated as "no structures here", not
/// an error — unlike `count_chunk_blocks`, where a missing `sections` list
/// means the chunk NBT itself is a shape we don't understand.
pub(crate) fn count_chunk_structures(nbt_bytes: &[u8], counts: &mut HashMap<String, u64>) -> Result<(), WorldError> {
    let chunk: Value = fastnbt::from_bytes(nbt_bytes)?;
    let Value::Compound(chunk) = chunk else {
        return Err(WorldError::Shape("chunk root is not a compound".into()));
    };
    let Some(Value::Compound(structures)) = chunk.get("structures") else {
        return Ok(());
    };
    let Some(Value::Compound(starts)) = structures.get("starts") else {
        return Ok(());
    };

    for id in starts.keys() {
        *counts.entry(id.clone()).or_insert(0) += 1;
    }
    Ok(())
}

/// Tallies living entities (mobs, dropped items, projectiles, ...) in an
/// `entities/` chunk by their `id` (e.g. `"minecraft:sheep"`), into
/// `counts` — the rest of an entity's NBT (position, health, ...) is noise
/// for a per-type count, same principle as `block_name` ignoring
/// `Properties`. Since 1.17, entities live in their own region files under
/// `entities/`, with a different chunk root shape than `region/`'s
/// (`Entities` list instead of `sections`) — see `count_chunk_blocks` for
/// that other shape. A missing or empty `Entities` list is treated as
/// zero entities, not an error.
pub(crate) fn count_chunk_entities(nbt_bytes: &[u8], counts: &mut HashMap<String, u64>) -> Result<(), WorldError> {
    let chunk: Value = fastnbt::from_bytes(nbt_bytes)?;
    let Value::Compound(chunk) = chunk else {
        return Err(WorldError::Shape("chunk root is not a compound".into()));
    };
    let Some(Value::List(entities)) = chunk.get("Entities") else {
        return Ok(());
    };

    for entity in entities {
        let Value::Compound(entity) = entity else {
            continue;
        };
        if let Some(Value::String(id)) = entity.get("id") {
            *counts.entry(id.clone()).or_insert(0) += 1;
        }
    }
    Ok(())
}

/// Diffs two versions of the same chunk's block data, block by block.
/// `chunk_x`/`chunk_z` are the chunk's absolute coordinates (as reported by
/// `diff_region_chunks`), used to turn each local position into an absolute
/// world block position.
///
/// Sections present on only one side (a rare case — the chunk's generated
/// height range itself changed) are skipped in this first slice; only
/// sections present on both sides are compared.
pub fn diff_chunk_blocks(
    from_nbt: &[u8],
    to_nbt: &[u8],
    chunk_x: i32,
    chunk_z: i32,
) -> Result<Vec<BlockDiff>, WorldError> {
    let from_sections = decode_chunk_sections(from_nbt)?;
    let to_sections = decode_chunk_sections(to_nbt)?;

    let mut diffs = Vec::new();
    for (section_y, from_blocks) in &from_sections {
        let Some(to_blocks) = to_sections.get(section_y) else {
            continue;
        };

        for local_index in 0..BLOCKS_PER_SECTION {
            if from_blocks[local_index] == to_blocks[local_index] {
                continue;
            }
            let local_y = local_index / 256;
            let local_z = (local_index % 256) / 16;
            let local_x = local_index % 16;

            diffs.push(BlockDiff {
                x: chunk_x * 16 + local_x as i32,
                y: *section_y as i32 * 16 + local_y as i32,
                z: chunk_z * 16 + local_z as i32,
                from: from_blocks[local_index].clone(),
                to: to_blocks[local_index].clone(),
            });
        }
    }

    Ok(diffs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fastanvil::Region;
    use std::io::Cursor;

    fn palette_entry(name: &str) -> Value {
        let mut map = HashMap::new();
        map.insert("Name".to_string(), Value::String(name.to_string()));
        Value::Compound(map)
    }

    fn palette_entry_with_properties(name: &str, properties: &[(&str, &str)]) -> Value {
        let mut map = HashMap::new();
        map.insert("Name".to_string(), Value::String(name.to_string()));
        let mut props = HashMap::new();
        for (k, v) in properties {
            props.insert(k.to_string(), Value::String(v.to_string()));
        }
        map.insert("Properties".to_string(), Value::Compound(props));
        Value::Compound(map)
    }

    /// Bit-packs `indices` (each < `palette_len`) the same way Minecraft
    /// does: `bits_needed(palette_len).max(4)` bits per entry, entries never
    /// straddling a `long` boundary.
    fn pack_indices(indices: &[usize], palette_len: usize) -> Vec<i64> {
        let bits_per_block = bits_needed(palette_len).max(4);
        let entries_per_long = 64 / bits_per_block;

        indices
            .chunks(entries_per_long)
            .map(|chunk| {
                let mut long: u64 = 0;
                for (i, &index) in chunk.iter().enumerate() {
                    long |= (index as u64) << (i * bits_per_block);
                }
                long as i64
            })
            .collect()
    }

    fn section_nbt(y: i8, palette: Vec<Value>, indices: Option<Vec<usize>>) -> Value {
        let mut block_states = HashMap::new();
        let palette_len = palette.len();
        block_states.insert("palette".to_string(), Value::List(palette));
        if let Some(indices) = indices {
            let packed = pack_indices(&indices, palette_len);
            block_states.insert(
                "data".to_string(),
                Value::LongArray(fastnbt::LongArray::new(packed)),
            );
        }

        let mut section = HashMap::new();
        section.insert("Y".to_string(), Value::Byte(y));
        section.insert("block_states".to_string(), Value::Compound(block_states));
        Value::Compound(section)
    }

    fn chunk_nbt_bytes(sections: Vec<Value>) -> Vec<u8> {
        let mut chunk = HashMap::new();
        chunk.insert("sections".to_string(), Value::List(sections));
        fastnbt::to_bytes(&Value::Compound(chunk)).unwrap()
    }

    /// A single all-air section, spot-checked against the real shape seen
    /// live in the medieval test world: palette len 1, no `data` at all.
    #[test]
    fn decodes_single_entry_palette_without_data_as_uniform_blocks() {
        let sections = vec![section_nbt(6, vec![palette_entry("minecraft:air")], None)];
        let bytes = chunk_nbt_bytes(sections);

        let decoded = decode_chunk_sections(&bytes).unwrap();
        let blocks = &decoded[&6];
        assert_eq!(blocks.len(), BLOCKS_PER_SECTION);
        assert!(blocks.iter().all(|b| b == "minecraft:air"));
    }

    /// Mirrors the real Y=4 section observed live: 18-entry palette needs 5
    /// bits/block, so this also exercises the "index doesn't divide 64
    /// evenly, some long is only partially used" path.
    #[test]
    fn decodes_multi_entry_palette_with_bit_packing() {
        let names: Vec<String> = (0..18).map(|i| format!("minecraft:block_{i}")).collect();
        let palette: Vec<Value> = names.iter().map(|n| palette_entry(n)).collect();

        let mut indices = vec![0usize; BLOCKS_PER_SECTION];
        indices[0] = 5;
        indices[4095] = 17;

        let sections = vec![section_nbt(4, palette, Some(indices.clone()))];
        let bytes = chunk_nbt_bytes(sections);

        let decoded = decode_chunk_sections(&bytes).unwrap();
        let blocks = &decoded[&4];
        assert_eq!(blocks.len(), BLOCKS_PER_SECTION);
        assert_eq!(blocks[0], "minecraft:block_5");
        assert_eq!(blocks[4095], "minecraft:block_17");
        assert_eq!(blocks[1], "minecraft:block_0");
    }

    #[test]
    fn same_name_different_properties_are_different_identities() {
        let lit = block_identity(&palette_entry_with_properties(
            "minecraft:furnace",
            &[("lit", "true"), ("facing", "north")],
        ))
        .unwrap();
        let unlit = block_identity(&palette_entry_with_properties(
            "minecraft:furnace",
            &[("facing", "north"), ("lit", "false")],
        ))
        .unwrap();

        assert_ne!(lit, unlit);
        assert_eq!(lit, "minecraft:furnace[facing=north,lit=true]");
    }

    #[test]
    fn diff_chunk_blocks_reports_only_the_block_that_changed() {
        let from_sections = vec![section_nbt(
            0,
            vec![palette_entry("minecraft:stone"), palette_entry("minecraft:air")],
            Some(vec![0; BLOCKS_PER_SECTION]),
        )];
        let mut to_indices = vec![0usize; BLOCKS_PER_SECTION];
        to_indices[0] = 1; // local (x=0, y=0, z=0) becomes air

        let to_sections = vec![section_nbt(
            0,
            vec![palette_entry("minecraft:stone"), palette_entry("minecraft:air")],
            Some(to_indices),
        )];

        let from_bytes = chunk_nbt_bytes(from_sections);
        let to_bytes = chunk_nbt_bytes(to_sections);

        let diffs = diff_chunk_blocks(&from_bytes, &to_bytes, 2, -3).unwrap();

        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].x, 2 * 16);
        assert_eq!(diffs[0].y, 0);
        assert_eq!(diffs[0].z, -3 * 16);
        assert_eq!(diffs[0].from, "minecraft:stone");
        assert_eq!(diffs[0].to, "minecraft:air");
    }

    #[test]
    fn diff_chunk_blocks_skips_sections_present_on_only_one_side() {
        let from_sections = vec![section_nbt(0, vec![palette_entry("minecraft:stone")], None)];
        let to_sections = vec![
            section_nbt(0, vec![palette_entry("minecraft:stone")], None),
            section_nbt(1, vec![palette_entry("minecraft:dirt")], None),
        ];

        let from_bytes = chunk_nbt_bytes(from_sections);
        let to_bytes = chunk_nbt_bytes(to_sections);

        let diffs = diff_chunk_blocks(&from_bytes, &to_bytes, 0, 0).unwrap();
        assert!(diffs.is_empty());
    }

    /// End-to-end sanity check against a real `.mca` region built the same
    /// way `region.rs`'s tests do (via `Region::create`), confirming the
    /// chunk bytes read back out of a region round-trip through the decoder.
    #[test]
    fn decodes_a_chunk_read_back_out_of_a_real_region() {
        let sections = vec![section_nbt(0, vec![palette_entry("minecraft:stone")], None)];
        let chunk_bytes = chunk_nbt_bytes(sections);

        let mut region = Region::create(Cursor::new(Vec::new())).unwrap();
        region.write_chunk(0, 0, &chunk_bytes).unwrap();
        let region_bytes = region.into_inner().unwrap().into_inner();

        let mut region = Region::from_stream(Cursor::new(region_bytes)).unwrap();
        let raw = region.read_chunk(0, 0).unwrap().unwrap();

        let decoded = decode_chunk_sections(&raw).unwrap();
        assert_eq!(decoded[&0], vec!["minecraft:stone".to_string(); BLOCKS_PER_SECTION]);
    }

    #[test]
    fn count_chunk_blocks_tallies_a_single_entry_palette_section() {
        let sections = vec![section_nbt(6, vec![palette_entry("minecraft:air")], None)];
        let bytes = chunk_nbt_bytes(sections);

        let mut counts = HashMap::new();
        count_chunk_blocks(&bytes, &mut counts).unwrap();

        assert_eq!(counts.len(), 1);
        assert_eq!(counts["minecraft:air"], BLOCKS_PER_SECTION as u64);
    }

    /// Mirrors the bit-packing test's real Y=4 shape (18-entry palette, 5
    /// bits/block) but checks the tally instead of per-position identities.
    #[test]
    fn count_chunk_blocks_tallies_a_multi_entry_palette_section() {
        let names: Vec<String> = (0..18).map(|i| format!("minecraft:block_{i}")).collect();
        let palette: Vec<Value> = names.iter().map(|n| palette_entry(n)).collect();

        let mut indices = vec![0usize; BLOCKS_PER_SECTION]; // all block_0 except two positions
        indices[0] = 5;
        indices[1] = 5;

        let sections = vec![section_nbt(4, palette, Some(indices))];
        let bytes = chunk_nbt_bytes(sections);

        let mut counts = HashMap::new();
        count_chunk_blocks(&bytes, &mut counts).unwrap();

        assert_eq!(counts["minecraft:block_5"], 2);
        assert_eq!(counts["minecraft:block_0"], (BLOCKS_PER_SECTION - 2) as u64);
        assert_eq!(counts.get("minecraft:block_17"), None, "never appears, shouldn't show up at all");
    }

    #[test]
    fn count_chunk_blocks_ignores_properties_unlike_diffing() {
        let sections = vec![section_nbt(
            0,
            vec![
                palette_entry_with_properties("minecraft:furnace", &[("lit", "true")]),
                palette_entry_with_properties("minecraft:furnace", &[("lit", "false")]),
            ],
            Some(vec![0; BLOCKS_PER_SECTION]),
        )];
        let bytes = chunk_nbt_bytes(sections);

        let mut counts = HashMap::new();
        count_chunk_blocks(&bytes, &mut counts).unwrap();

        // Both palette entries share the bare name "minecraft:furnace" — the
        // lit/unlit distinction that matters for diffing collapses here.
        assert_eq!(counts.len(), 1);
        assert_eq!(counts["minecraft:furnace"], BLOCKS_PER_SECTION as u64);
    }

    #[test]
    fn count_chunk_blocks_sums_across_multiple_sections() {
        let sections = vec![
            section_nbt(0, vec![palette_entry("minecraft:stone")], None),
            section_nbt(1, vec![palette_entry("minecraft:stone")], None),
        ];
        let bytes = chunk_nbt_bytes(sections);

        let mut counts = HashMap::new();
        count_chunk_blocks(&bytes, &mut counts).unwrap();

        assert_eq!(counts["minecraft:stone"], (BLOCKS_PER_SECTION * 2) as u64);
    }
}
