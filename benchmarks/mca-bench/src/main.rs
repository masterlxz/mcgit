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
        _ => {
            eprintln!(
                "uso:\n  mca-bench list <arquivo.mca>\n  mca-bench inspect <arquivo.mca> <chunk_x> <chunk_z>\n  mca-bench mutate <arquivo.mca> <chunk_x> <chunk_z>"
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
    Ok(())
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
