## Escopo do projeto (Sessão 1, revisão 2)

`mcgit` deixou de ser só uma ferramenta de versionamento e passou a ser o nome do **launcher
completo** (auth, instâncias, Java, mods, modpacks, skins, mundos, versionamento, backup,
Arweave, TruthID). O que era "o mcgit inteiro" na v1.0 do PRD (`CONTEXT.md`) agora é
especificamente o módulo **Git Engine / World Manager** dentro de um produto maior — nada do
trabalho da Fase 0 original (benchmark de Git, estrutura de mundo, NBT) foi descartado, só
reencaixado. Ver `CONTEXT.md` v2.0 para o PRD completo e `PHASE.md` para o roadmap reconciliado.

---

## Arquitetura de Módulos (alto nível — interfaces detalhadas são entregável da Fase 0)

```text
mcgit
│
├── Authentication          (Microsoft OAuth agora; abstração pra TruthID depois)
├── Minecraft Version Manager
├── Java Manager
├── Instance Manager
├── Mod Manager
├── Modpack Manager          (Modrinth / CurseForge — ver CONTEXT.md §Legal & Licensing)
├── Skin Manager
├── World Manager
├── Git Engine                (o "mcgit" da v1.0 — snapshot/restore/branch)
├── Backup Engine
├── Arweave Storage            (StorageProvider — Fase 7)
├── TruthID Integration         (AuthenticationProvider — Fase 7, sem acoplamento cedo)
└── Game Runner                  (processo do Java/Minecraft, multiplataforma)
```

Duas abstrações importam desde o design inicial, mesmo com só um lado implementado cedo:

```text
StorageProvider                    AuthenticationProvider
├── LocalStorage    (MVP)          ├── Microsoft   (MVP)
├── CloudStorage     (Fase 5)      └── TruthID       (Fase 7)
└── ArweaveStorage     (Fase 7)
```

**Proposta inicial de workspace Rust** (a confirmar/ajustar na Fase 0, ao desenhar as
interfaces reais entre módulos):

```text
mcgit/
├── crates/
│   ├── mcgit-core         (Git Engine — versionamento de mundo; era "mcgit-core" da v1.0)
│   ├── mcgit-auth          (Microsoft OAuth; trait AuthenticationProvider)
│   ├── mcgit-java           (detecção/instalação de Java)
│   ├── mcgit-instance        (Instance Manager)
│   ├── mcgit-mods             (Mod/Modpack Manager — clients Modrinth/CurseForge)
│   ├── mcgit-storage            (trait StorageProvider — local/cloud/arweave)
│   ├── mcgit-db                  (camada de acesso ao SQLite local)
│   └── mcgit-runner                (Game Runner — processo multiplataforma)
├── apps/
│   ├── cli                (binário `mcgit`, casca fina sobre as crates acima)
│   └── desktop               (Tauri + React/TypeScript, mesma casca, GUI é o produto principal)
```

---

## Decisões de Arquitetura em Aberto

| Decisão | Opções | Status |
|---|---|---|
| Linguagem principal | Rust vs Python | **Rust** ✓ (decidido na Sessão 1) — reaproveita a experiência do desktop do TruthID (Tauri); confirmado válido pro launcher inteiro na revisão de escopo (Sessão 1, mesmo dia) |
| Interface | CLI vs GUI como produto principal | **GUI primeiro** ✓ (decisão revisada na Sessão 1) — um launcher é fundamentalmente uma experiência gráfica; **inverte** a decisão anterior "CLI primeiro" da v1.0 (que fazia sentido pra uma ferramenta de linha de comando isolada, não pra um launcher). CLI continua existindo em paralelo (`mcgit init/commit/log/restore` + comandos de launcher), mas é opcional — nunca bloqueante pro jogador comum |
| Stack de GUI | Tauri+React/TS vs outra opção | **Tauri + Rust (backend) + React/TypeScript (frontend)** ✓ (decidido na Sessão 1, confirmado na revisão de escopo) — mesma stack do `truthid/desktop`; atende os requisitos de §24/26 do prompt original (controle de processo, filesystem, segurança, multiplataforma) |
| Uso do Git (dentro do Git Engine) | Chamar o binário `git` do sistema vs biblioteca (`git2`/libgit2 em Rust) vs implementação própria mínima | **Em aberto** — investigar na Fase 0, restrito ao ecossistema Rust |
| Estratégia de armazenamento de `.mca` | Git puro vs Git LFS vs camada própria por região/chunk antes do Git | **Git puro parece viável** (ver benchmark da Sessão 1 abaixo) — Git LFS ainda não testado como comparação; decisão final pendente de mais rodadas (mundos maiores, múltiplas regiões, muitas sessões) |
| Compactação do repositório (`git gc`) | Depender do auto-gc padrão do Git vs o mcgit disparar `git gc`/repack periodicamente por conta própria | **Em aberto, mas indício forte a favor de rodar gc próprio** — ver benchmark abaixo: sem compactar, o `.git` cresce ~5.3M por snapshot mesmo mudando só 2-3 chunks de 960; com `git gc --aggressive`, 7 snapshots ficaram do tamanho de ~1 |
| Merge entre branches de mundo | Merge tradicional do Git vs não suportar merge (só criar/descartar branch) | **Em aberto** — não assumir que merge tradicional é seguro para arquivos de mundo |
| Banco de dados local | SQLite vs outra opção | **SQLite tentativo, não travado** — guarda metadados (instâncias, contas, mundos, mods, modpacks, instalações de Java, backups, repos Git, uploads Arweave, skins, settings); nunca o conteúdo dos arquivos do mundo em si (isso continua sendo Git + filesystem). Schema real é entregável da Fase 0 |
| Gerenciamento de Java | Baixar/gerenciar builds próprias (ex.: Eclipse Temurin) vs delegar pra uma lib existente | **Em aberto** — Fase 0 |
| Integração de modpacks | Modrinth API vs CurseForge API vs ambas desde o início | **Em aberto** — depende da revisão de licenciamento/ToS (ver `CONTEXT.md` §Legal & Licensing); Modrinth tem API mais aberta, pode ser o primeiro alvo |
| Fluxo de autenticação Microsoft | Detalhes exatos do OAuth (device code flow vs outro) | **Em aberto** — Fase 0, depende de revisar os requisitos oficiais da Microsoft/Mojang pra launchers de terceiros |
| Armazenamento de credenciais | Keyring nativo do SO (Windows Credential Manager / macOS Keychain / Linux Keyring) | **Padrão adotado por princípio** (mesmo approach do TruthID) — detalhes de implementação por plataforma ainda em aberto |
| Detecção de mundo aberto | Lock file do próprio Minecraft vs heurística de processo vs não detectar (só avisar) | **Em aberto** — investigar na Fase 0/1 |
| Unidade de upload para Arweave | Snapshot completo vs objetos Git/deltas vs regiões alteradas vs bundle de versões | **Em aberto** — Fase 7, junto com o desenho de custo |
| Mapeamento commit Git ↔ transação Arweave | Estrutura de metadados exata | **Em aberto** — Fase 7 |

---

## Legal & Licenciamento — bloqueia código, não bloqueia pesquisa/docs

Ver `CONTEXT.md` §Legal & Licensing Considerations para a lista completa (requisitos da
Microsoft/Mojang pra launchers de terceiros, ToS da CurseForge vs Modrinth, redistribuição de
mods, uso da API de skins). Regra prática: **nenhum código que toque autenticação Microsoft,
CurseForge ou API de skins deve ser escrito antes dessa revisão acontecer** — é um item da
Fase 0.

---

## Débitos Técnicos de Arquitetura

Nenhum ainda — projeto em Fase 0, sem código de produto escrito (o benchmark abaixo é
ferramenta de pesquisa descartável, não código do launcher).

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

Não aplicável ainda. mcgit não depende de deploy de contratos ou infraestrutura própria nas
fases iniciais do launcher (ver `PHASE.md`). A integração com TruthID/Arweave (Fase 7)
reaproveita a infraestrutura já implantada do TruthID (`../truthid/project/ARCHITECTURE.md`) —
não deve exigir deploy de novos contratos, salvo se a investigação da Fase 7 mostrar o
contrário.
