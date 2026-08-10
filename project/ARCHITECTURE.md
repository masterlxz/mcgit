## Decisões de Arquitetura em Aberto

| Decisão | Opções | Status |
|---|---|---|
| Linguagem principal | Rust vs Python | **Rust** ✓ (decidido na Sessão 1) — reaproveita a experiência do desktop do TruthID (Tauri); benchmarks da Fase 0 continuam servindo para validar a estratégia de armazenamento, não para reabrir a escolha de linguagem |
| Uso do Git | Chamar o binário `git` do sistema vs biblioteca (`git2`/libgit2 em Rust) vs implementação própria mínima | **Em aberto** — investigar na Fase 0, agora restrito ao ecossistema Rust |
| Estratégia de armazenamento de `.mca` | Git puro vs Git LFS vs camada própria por região/chunk antes do Git | **Em aberto** — depende de benchmarks de tamanho real de mundos |
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

## Pendências de Deploy

Não aplicável ainda. mcgit não depende de deploy de contratos ou infraestrutura própria no
MVP (Fase 1-4). A integração com TruthID/Arweave (Fase 5) reaproveita a infraestrutura já
implantada do TruthID (`../truthid/project/ARCHITECTURE.md`) — não deve exigir deploy de novos
contratos, salvo se a investigação da Fase 5 mostrar o contrário.
