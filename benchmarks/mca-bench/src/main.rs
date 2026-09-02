use std::env;
use std::fs::OpenOptions;

use anyhow::{bail, Context, Result};
use fastanvil::Region;
use fastnbt::Value;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("list") => cmd_list(&args[2..]),
        Some("inspect") => cmd_inspect(&args[2..]),
        Some("mutate") => cmd_mutate(&args[2..]),
        Some("set-block") => cmd_set_block(&args[2..]),
        _ => {
            eprintln!(
                "uso:\n  mca-bench list <arquivo.mca>\n  mca-bench inspect <arquivo.mca> <chunk_x> <chunk_z>\n  mca-bench mutate <arquivo.mca> <chunk_x> <chunk_z>\n  mca-bench set-block <arquivo.mca> <chunk_x> <chunk_z> <section_y> <local_x> <local_y> <local_z> <bloco>"
            );
            std::process::exit(1);
        }
    }
}

fn open_region(path: &str) -> Result<Region<std::fs::File>> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .with_context(|| format!("abrindo {path}"))?;
    Region::from_stream(file).with_context(|| format!("lendo região {path}"))
}

/// Lista quais dos 1024 slots (32x32) da região têm chunk gerado.
fn cmd_list(args: &[String]) -> Result<()> {
    let path = args.first().context("faltou <arquivo.mca>")?;
    let mut region = open_region(path)?;

    let mut existing = vec![];
    for z in 0..32 {
        for x in 0..32 {
            if region.read_chunk(x, z)?.is_some() {
                existing.push((x, z));
            }
        }
    }

    println!("{}: {} chunks gerados de 1024 possíveis", path, existing.len());
    for (x, z) in existing.iter().take(10) {
        println!("  chunk ({x}, {z})");
    }
    if existing.len() > 10 {
        println!("  ... e mais {}", existing.len() - 10);
    }
    Ok(())
}

/// Mostra as chaves de topo do NBT de um chunk e o valor de InhabitedTime, se existir.
fn cmd_inspect(args: &[String]) -> Result<()> {
    let (path, x, z) = parse_chunk_args(args)?;
    let mut region = open_region(&path)?;

    let raw = region
        .read_chunk(x, z)?
        .with_context(|| format!("chunk ({x},{z}) não existe em {path}"))?;
    let chunk: Value = fastnbt::from_bytes(&raw).context("parseando NBT do chunk")?;

    let Value::Compound(map) = &chunk else {
        bail!("raiz do chunk não é um Compound");
    };

    println!("chunk ({x},{z}) — {} bytes de NBT descomprimido", raw.len());
    println!("chaves de topo: {:?}", map.keys().collect::<Vec<_>>());
    match map.get("InhabitedTime") {
        Some(Value::Long(t)) => println!("InhabitedTime = {t} ticks (~{:.1} min)", *t as f64 / 20.0 / 60.0),
        Some(other) => println!("InhabitedTime tem tipo inesperado: {other:?}"),
        None => println!("InhabitedTime não encontrado na raiz"),
    }

    inspect_sections(map);

    match map.get("block_entities") {
        Some(Value::List(list)) => println!("block_entities: {} item(ns)", list.len()),
        _ => println!("block_entities: ausente"),
    }
    if let Some(Value::List(list)) = map.get("block_entities") {
        for entry in list.iter().take(5) {
            println!("  {entry:?}");
        }
    }
    match map.get("structures") {
        Some(v) => println!("structures (raw): {v:?}"),
        None => println!("structures: ausente"),
    }
    if let Some(Value::List(list)) = map.get("Entities") {
        println!("Entities: {} item(ns)", list.len());
        for entry in list.iter().take(5) {
            println!("  {entry:?}");
        }
    }
    Ok(())
}

/// Mostra, pra cada seção vertical (16 blocos de altura) do chunk, a Y da seção e a estrutura
/// de `block_states`: tamanho da `palette` (os tipos de bloco presentes) e tamanho do array
/// `data` (os índices compactados apontando pra palette) — exatamente o que o diff por bloco
/// (Fase 4) precisa decodificar pra ir além de "mudou ou não".
fn inspect_sections(chunk: &std::collections::HashMap<String, Value>) {
    let Some(Value::List(sections)) = chunk.get("sections") else {
        println!("sem lista \"sections\" no chunk");
        return;
    };

    println!("{} seções verticais:", sections.len());
    for section in sections {
        let Value::Compound(section) = section else {
            continue;
        };
        let y = match section.get("Y") {
            Some(Value::Byte(y)) => y.to_string(),
            other => format!("{other:?}"),
        };

        let Some(Value::Compound(block_states)) = section.get("block_states") else {
            println!("  Y={y}: sem block_states (provavelmente só ar)");
            continue;
        };

        let palette_len = match block_states.get("palette") {
            Some(Value::List(p)) => p.len(),
            _ => 0,
        };
        let data_len = match block_states.get("data") {
            Some(Value::LongArray(d)) => d.iter().count(),
            _ => 0,
        };
        println!("  Y={y}: palette com {palette_len} tipo(s) de bloco, data com {data_len} long(s)");

        if let Some(Value::List(palette)) = block_states.get("palette") {
            for entry in palette {
                if let Value::Compound(entry) = entry {
                    if let Some(Value::String(name)) = entry.get("Name") {
                        println!("    - {name}");
                    }
                }
            }
        }
    }
}



/// Simula "alguém jogou aqui": incrementa InhabitedTime e regrava o chunk na região.
/// É uma mutação pequena e real (o próprio Minecraft atualiza esse campo o tempo todo),
/// sem precisar decodificar os block-states empacotados pra fazer uma edição de bloco de verdade.
fn cmd_mutate(args: &[String]) -> Result<()> {
    let (path, x, z) = parse_chunk_args(args)?;
    let mut region = open_region(&path)?;

    let raw = region
        .read_chunk(x, z)?
        .with_context(|| format!("chunk ({x},{z}) não existe em {path}"))?;
    let mut chunk: Value = fastnbt::from_bytes(&raw).context("parseando NBT do chunk")?;

    let before = bump_inhabited_time(&mut chunk)?;

    let out = fastnbt::to_bytes(&chunk).context("serializando NBT do chunk")?;
    let after_len = out.len();
    region.write_chunk(x, z, &out)?;

    println!(
        "chunk ({x},{z}) de {path}: InhabitedTime {before} -> {} | NBT descomprimido {} -> {} bytes",
        before + 20,
        raw.len(),
        after_len
    );
    Ok(())
}

fn bump_inhabited_time(chunk: &mut Value) -> Result<i64> {
    let Value::Compound(map) = chunk else {
        bail!("raiz do chunk não é um Compound");
    };
    let Some(Value::Long(t)) = map.get_mut("InhabitedTime") else {
        bail!("campo InhabitedTime não encontrado na raiz do chunk (formato pode ser diferente do esperado)");
    };
    let before = *t;
    *t += 20; // +20 ticks = +1 segundo de "tempo habitado", como se alguém tivesse passado por ali
    Ok(before)
}

/// Sets one block inside an existing section to a block type that's already
/// present in that section's palette — a controlled, real edit for manual
/// testing (e.g. of the Fase 4 block-diff feature), without needing to grow
/// or re-encode the palette itself.
fn cmd_set_block(args: &[String]) -> Result<()> {
    let (path, chunk_x, chunk_z) = parse_chunk_args(args)?;
    let section_y: i8 = args.get(3).context("faltou <section_y>")?.parse().context("section_y inválido")?;
    let local_x: usize = args.get(4).context("faltou <local_x>")?.parse().context("local_x inválido (0-15)")?;
    let local_y: usize = args.get(5).context("faltou <local_y>")?.parse().context("local_y inválido (0-15)")?;
    let local_z: usize = args.get(6).context("faltou <local_z>")?.parse().context("local_z inválido (0-15)")?;
    let target_name = args.get(7).context("faltou <bloco> (ex.: minecraft:air)")?.clone();

    let mut region = open_region(&path)?;
    let raw = region
        .read_chunk(chunk_x, chunk_z)?
        .with_context(|| format!("chunk ({chunk_x},{chunk_z}) não existe em {path}"))?;
    let mut chunk: Value = fastnbt::from_bytes(&raw).context("parseando NBT do chunk")?;

    let Value::Compound(chunk_map) = &mut chunk else {
        bail!("raiz do chunk não é um Compound");
    };
    let Some(Value::List(sections)) = chunk_map.get_mut("sections") else {
        bail!("chunk não tem lista \"sections\"");
    };
    let section = sections
        .iter_mut()
        .find_map(|s| {
            let Value::Compound(s) = s else { return None };
            matches!(s.get("Y"), Some(Value::Byte(y)) if *y == section_y).then_some(s)
        })
        .with_context(|| format!("seção Y={section_y} não encontrada"))?;
    let Some(Value::Compound(block_states)) = section.get_mut("block_states") else {
        bail!("seção Y={section_y} não tem block_states (provavelmente só ar — sem palette pra reaproveitar)");
    };

    let Some(Value::List(palette)) = block_states.get("palette") else {
        bail!("block_states sem palette");
    };
    let palette_len = palette.len();
    let target_index = palette
        .iter()
        .position(|entry| matches!(entry, Value::Compound(e) if matches!(e.get("Name"), Some(Value::String(n)) if n == &target_name)))
        .with_context(|| format!("\"{target_name}\" não está na palette desta seção (tipos presentes precisam já existir — sem crescer a palette aqui)"))?;

    let bits_per_block = bits_needed(palette_len).max(4);
    let entries_per_long = 64 / bits_per_block;
    let mask: u64 = (1u64 << bits_per_block) - 1;
    let local_index = local_y * 256 + local_z * 16 + local_x;
    let long_idx = local_index / entries_per_long;
    let shift = (local_index % entries_per_long) * bits_per_block;

    let Some(Value::LongArray(data)) = block_states.get_mut("data") else {
        bail!("block_states sem data (palette de 1 tipo só — nada a sobrescrever)");
    };
    let before = (data[long_idx] as u64 >> shift) & mask;
    data[long_idx] = ((data[long_idx] as u64 & !(mask << shift)) | ((target_index as u64) << shift)) as i64;

    let out = fastnbt::to_bytes(&chunk).context("serializando NBT do chunk")?;
    region.write_chunk(chunk_x, chunk_z, &out)?;

    println!(
        "chunk ({chunk_x},{chunk_z}) seção Y={section_y}, bloco local ({local_x},{local_y},{local_z}): índice de palette {before} -> {target_index} (\"{target_name}\")"
    );
    Ok(())
}

fn bits_needed(n: usize) -> usize {
    if n <= 1 {
        return 0;
    }
    (usize::BITS - (n - 1).leading_zeros()) as usize
}

fn parse_chunk_args(args: &[String]) -> Result<(String, usize, usize)> {
    let path = args.first().context("faltou <arquivo.mca>")?.clone();
    let x: usize = args
        .get(1)
        .context("faltou <chunk_x>")?
        .parse()
        .context("chunk_x inválido (0-31)")?;
    let z: usize = args
        .get(2)
        .context("faltou <chunk_z>")?
        .parse()
        .context("chunk_z inválido (0-31)")?;
    Ok((path, x, z))
}
