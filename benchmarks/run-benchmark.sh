#!/usr/bin/env bash
# Fase 0 — benchmark: como o Git se comporta versionando um mundo real do Minecraft
# ao longo de várias "sessões de jogo" simuladas (mutações reais em chunks via mca-bench).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORLD_SRC="$ROOT/worlds/medieval"
BENCH_DIR="$ROOT/git-bench"
BIN="$ROOT/mca-bench/target/debug/mca-bench"
REGION_REL="world/region/r.0.0.mca"

if [[ ! -x "$BIN" ]]; then
  echo "mca-bench não compilado ainda. Rode: (cd $ROOT/mca-bench && cargo build)" >&2
  exit 1
fi

rm -rf "$BENCH_DIR"
mkdir -p "$BENCH_DIR"
cp -r "$WORLD_SRC" "$BENCH_DIR/world"

cd "$BENCH_DIR"
git init -q
git add -A
git commit -q -m "snapshot 0 (baseline)"

print_size() {
  local label="$1"
  local bytes
  bytes=$(du -sb .git | cut -f1)
  printf "%-28s %10d bytes  (%s)\n" "$label" "$bytes" "$(du -sh .git | cut -f1)"
}

echo "=== crescimento do .git por snapshot ==="
print_size "snapshot 0 (baseline)"

# Cada linha = uma "sessão de jogo": lista de pares (chunk_x chunk_z) mutados na mesma região.
SESSIONS=(
  "5 5 6 5 7 5"
  "5 6 6 6 7 6"
  "8 5 8 6 8 7"
  "10 10 11 10 12 10"
  "10 11 11 11 12 11"
  "20 20 21 20 22 20"
)

i=1
for session in "${SESSIONS[@]}"; do
  read -ra coords <<< "$session"
  for ((j = 0; j < ${#coords[@]}; j += 2)); do
    x=${coords[j]}
    z=${coords[j + 1]}
    "$BIN" mutate "$REGION_REL" "$x" "$z" >/dev/null
  done
  git add -A
  git commit -q -m "snapshot $i (sessão de jogo simulada)"
  print_size "snapshot $i"
  i=$((i + 1))
done

echo
echo "=== git count-objects -v (solto, antes de empacotar) ==="
git count-objects -v

echo
echo "=== rodando git gc --aggressive (empacota e aplica delta-compression) ==="
git gc --aggressive -q

echo
echo "=== git count-objects -v (depois do gc) ==="
git count-objects -v
print_size "final (pós gc)"

echo
echo "=== tamanho de r.0.0.mca em cada commit (working copy) ==="
git log --oneline --reverse | while read -r hash rest; do
  size=$(git cat-file -p "$hash:$REGION_REL" | wc -c)
  echo "$hash  $rest  ->  $size bytes"
done
