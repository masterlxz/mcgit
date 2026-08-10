## Decisões de Arquitetura em Aberto

| Decisão | Opções | Status |
|---|---|---|
| Linguagem principal | Rust vs Python | **Rust** ✓ (decidido na Sessão 1) — reaproveita a experiência do desktop do TruthID (Tauri); benchmarks da Fase 0 continuam servindo para validar a estratégia de armazenamento, não para reabrir a escolha de linguagem |
| Uso do Git | Chamar o binário `git` do sistema vs biblioteca (`git2`/libgit2 em Rust) vs implementação própria mínima | **Em aberto** — investigar na Fase 0, agora restrito ao ecossistema Rust |
| Estratégia de armazenamento de `.mca` | Git puro vs Git LFS vs camada própria por região/chunk antes do Git | **Git puro parece viável** (ver benchmark da Sessão 1 abaixo) — Git LFS ainda não testado como comparação; decisão final pendente de mais rodadas (mundos maiores, múltiplas regiões, muitas sessões) |
| Compactação do repositório (`git gc`) | Depender do auto-gc padrão do Git vs o mcgit disparar `git gc`/repack periodicamente por conta própria | **Em aberto, mas indício forte a favor de rodar gc próprio** — ver benchmark abaixo: sem compactar, o `.git` cresce ~5.3M por snapshot mesmo mudando só 2-3 chunks de 960; com `git gc --aggressive`, 7 snapshots ficaram do tamanho de ~1 |
| Merge entre branches de mundo | Merge tradicional do Git vs não suportar merge (só criar/descartar branch) | **Em aberto** — spec original é explícita: não assumir que merge tradicional é seguro para arquivos de mundo |
| Interface | CLI vs TUI vs GUI | **CLI primeiro** ✓ (decidido) — binário `mcgit` consumindo uma crate core (`mcgit-core`); GUI é Fase 7 |
| Stack de GUI (Fase 7) | Tauri+React/TS vs outra opção | **Tauri + Rust (backend) + React/TypeScript (frontend)** ✓ (decidido na Sessão 1) — mesma stack do `truthid/desktop`. O app Tauri é uma casca fina sobre `mcgit-core`, a mesma crate usada pelo binário CLI — nenhuma lógica de versionamento duplicada entre CLI e GUI |
| Detecção de mundo aberto | Lock file do próprio Minecraft vs heurística de processo vs não detectar (só avisar) | **Em aberto** — investigar na Fase 0/1 |
| Unidade de upload para Arweave | Snapshot completo vs objetos Git/deltas vs regiões alteradas vs bundle de versões | **Em aberto** — Fase 5, junto com o desenho de custo |
| Mapeamento commit Git ↔ transação Arweave | Estrutura de metadados exata | **Em aberto** — Fase 5 |

---

## Débitos Técnicos de Arquitetura

Nenhum ainda — projeto em Fase 0, sem código escrito.

---

## Benchmark: Git puro vs mundo real (Sessão 1)

**Setup**: mundo real do usuário ("Medieval", 40M, cedido para uso como cobaia —
`benchmarks/worlds/medieval/`, fora do controle de versão). Ferramenta descartável
`benchmarks/mca-bench` (Rust, usa `fastnbt`/`fastanvil`) simula "sessões de jogo" incrementando
`InhabitedTime` (campo NBT real, atualizado pelo próprio Minecraft) em 2-3 chunks por sessão,
na região `r.0.0.mca` (6.8M, 960/1024 chunks gerados). Script completo em
`benchmarks/run-benchmark.sh`.

**Resultado**:

| | Tamanho do `.git` |
|---|---|
| Snapshot 0 (baseline, mundo inteiro) | 28.7M |
| Snapshots 1-6 (objetos soltos, sem compactar) | +~5.3M por snapshot (61M no total) |
| Depois de `git gc --aggressive` | 27.3M — 7 snapshots no tamanho de ~1 |

**Conclusões**:
1. **Objetos soltos (`git add`/`git commit` sem gc) nunca fazem delta** — cada commit que toca
   o `.mca` grava o arquivo quase inteiro de novo como blob novo. Isso é esperado do modelo do
   Git, mas significa que o `.git` incha rápido em uso normal sem compactação.
2. **Hipótese inicial parcialmente refutada**: eu esperava que a recompressão zlib por chunk
   "embaralhasse" os bytes a ponto do delta do Git não achar nada parecido entre versões do
   `.mca`. Na prática, cada chunk ocupa setores de 4KiB próprios e independentes dentro do
   arquivo — mutar 2-3 chunks de 960 deixa ~99,7% dos bytes do arquivo intactos e no mesmo
   offset, e o delta do Git (packfile, baseado em copiar trechos grandes iguais) explora isso
   muito bem: 7 versões do arquivo comprimiram para quase o tamanho de 1.
3. **Restauração testada e correta**: `git checkout` de um snapshot antigo reproduziu o
   `r.0.0.mca` bit-a-bit idêntico ao original (hash SHA-256 conferido).

**Ressalvas / próximos passos** (não conclusivo ainda):
- Testado só com edições pequenas (metadado, não bloco de verdade) e só numa região de um
  mundo. Falta: edições de bloco reais (packed block-states), mundo maior, muitas sessões (10s-100s),
  múltiplas regiões mudando por sessão, e comparação direta com Git LFS.
- O padrão de realocação de setores usado aqui é o do `fastanvil` (nosso writer de teste), não
  necessariamente idêntico ao do servidor Java oficial — a suposição de "bytes ficam estáveis
  no mesmo offset entre saves" precisa ser revalidada depois com um mundo salvo pelo próprio
  jogo/servidor real.

---

## Pendências de Deploy

Não aplicável ainda. mcgit não depende de deploy de contratos ou infraestrutura própria no
MVP (Fase 1-4). A integração com TruthID/Arweave (Fase 5) reaproveita a infraestrutura já
implantada do TruthID (`../truthid/project/ARCHITECTURE.md`) — não deve exigir deploy de novos
contratos, salvo se a investigação da Fase 5 mostrar o contrário.
