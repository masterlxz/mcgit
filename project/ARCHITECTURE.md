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
| Uso do Git (dentro do Git Engine) | Chamar o binário `git` do sistema vs biblioteca (`git2`/libgit2 em Rust) vs implementação própria mínima | **Binário `git` via subprocess** ✓ (decidido por análise, Sessão 1) — mais simples, sem custo de build/linking cross-platform de uma lib C |
| Estratégia de armazenamento de `.mca` | Git puro vs Git LFS vs camada própria por região/chunk antes do Git | **Git puro, sem LFS no MVP** ✓ (decidido por análise, Sessão 1) — benchmark já mostra Git puro + `git gc` resolvendo o caso comum; LFS adicionaria uma dependência de servidor que não se justifica ainda. Reabrir se um mundo real em produção mostrar o contrário |
| Compactação do repositório (`git gc`) | Depender do auto-gc padrão do Git vs o mcgit disparar `git gc`/repack periodicamente por conta própria | **mcgit dispara `git gc` por conta própria** ✓ (decidido, Sessão 1) — sem compactar, o `.git` cresce ~5.3M por snapshot mesmo mudando só 2-3 chunks de 960; com `git gc --aggressive`, 7 snapshots ficaram do tamanho de ~1 |
| Merge entre branches de mundo | Merge tradicional do Git vs não suportar merge (só criar/descartar branch) | **Em aberto** — não assumir que merge tradicional é seguro para arquivos de mundo |
| Banco de dados local | SQLite vs outra opção | **SQLite** ✓ (decidido por análise, Sessão 1) — guarda só metadados, nunca o conteúdo dos arquivos do mundo (isso continua sendo Git + filesystem). Schema proposto: §Schema do Banco Local |
| Gerenciamento de Java | Baixar/gerenciar builds próprias vs delegar pra lib existente | **Baixar builds Eclipse Temurin/Adoptium** ✓ (decidido por análise, Sessão 1) — ver §Gerenciamento de Java |
| Integração de modpacks | Modrinth API vs CurseForge API vs ambas desde o início | **Modrinth primeiro** ✓ (decidido por análise, Sessão 1) — API mais aberta; CurseForge condicionado à revisão de ToS (§Legal & Licenciamento) |
| Fluxo de autenticação Microsoft | Detalhes exatos do OAuth | **Cadeia MS OAuth → Xbox Live → XSTS → Minecraft Services** ✓ (decidido por análise, Sessão 1) — ver §Fluxo de Autenticação Microsoft. Registro do app no Azure AD é ação prática pendente, não decisão técnica |
| Armazenamento de credenciais | Keyring nativo do SO (Windows Credential Manager / macOS Keychain / Linux Keyring) | **Padrão adotado por princípio** (mesmo approach do TruthID) — detalhes de implementação por plataforma ainda em aberto |
| Detecção de mundo aberto | Lock file do próprio Minecraft vs heurística de processo vs não detectar (só avisar) | **Em aberto** — investigar na Fase 1 (validação empírica, não decidível só por análise) |
| Unidade de upload para Arweave | Snapshot completo vs objetos Git/deltas vs regiões alteradas vs bundle de versões | **Em aberto** — Fase 7, junto com o desenho de custo |
| Mapeamento commit Git ↔ transação Arweave | Estrutura de metadados exata | **Em aberto** — Fase 7 |

---

## Fluxo de Autenticação Microsoft

Cadeia padrão usada por launchers de terceiros (o mesmo caminho que Prism Launcher, MultiMC
etc. seguem):

1. App registrado no Azure AD (Microsoft Entra) com permissão `XboxLive.signin` — **ação
   prática pendente, não decisão técnica**, precisa acontecer antes do código de login existir.
2. OAuth (device code flow, ou authorization code + PKCE embutido no app Tauri): usuário loga
   na conta Microsoft, mcgit recebe um `access_token`.
3. Troca o `access_token` da Microsoft por um token do Xbox Live (`user.auth.xboxlive.com`).
4. Troca o token do Xbox Live por um token XSTS (`xsts.auth.xboxlive.com`) — prova "usuário
   Xbox válido" pro resto da cadeia.
5. Troca o token XSTS por um token da Minecraft Services API
   (`api.minecraftservices.com/authentication/login_with_xbox`).
6. Usa esse token pra checar posse do jogo (`.../entitlements/mcstore`) e buscar o perfil
   (`.../minecraft/profile` — nome, UUID, skin atual).
7. O token da Minecraft Services é o que efetivamente autentica o cliente do jogo ao iniciar.

A senha do usuário nunca passa pelo mcgit — só tokens OAuth curtos, renováveis via
`refresh_token`. Armazenar **só** o `refresh_token`, criptografado no keyring do SO (nunca em
texto puro no SQLite — ver `CONTEXT.md` §Security Requirements).

---

## Gerenciamento de Java

- Detectar a versão necessária a partir do manifesto de versão do Minecraft (Mojang informa
  isso por versão no próprio piston-meta).
- Nunca depender só do "Java do sistema" — instâncias diferentes podem precisar de versões
  diferentes ao mesmo tempo (Java 17 pra 1.20.x, Java 21 pra 1.21.x).
- Baixar builds do **Eclipse Temurin/Adoptium** (OpenJDK redistribuível, sem os requisitos de
  licença da Oracle JDK) quando a versão necessária não estiver instalada.
- Guardar cada versão baixada numa pasta própria do mcgit, uma por major version, reaproveitada
  entre instâncias que precisam da mesma versão.
- Permitir apontar pra um Java já instalado manualmente, pro usuário avançado.

---

## Gerenciamento de Instâncias

Cada instância é uma pasta isolada:

```text
instances/
└── <instance-id>/
    ├── instance.json      (metadados — espelha uma linha da tabela `instances` no SQLite)
    └── minecraft/          (.minecraft isolado: mods/, resourcepacks/, shaderpacks/, saves/, screenshots/, logs/, config/)
```

Bibliotecas/assets/natives compartilhados (o client jar do Minecraft 1.21.1 não muda entre
instâncias) ficam num cache global fora da pasta da instância, referenciado por hash/versão —
evita duplicar GBs de arquivos idênticos entre instâncias parecidas, sem quebrar o isolamento
do que realmente precisa ser isolado (config/mods/saves). Mesma solução que Prism Launcher e
MultiMC já usam.

---

## Schema do Banco Local (SQLite) — proposta inicial

```sql
instances(id, name, mc_version, loader, loader_version, java_version, jvm_args, created_at)
accounts(id, ms_account_id, minecraft_uuid, username, refresh_token_ref, last_login_at)
worlds(id, instance_id, name, path, git_enabled, created_at)
mods(id, instance_id, source, project_id, version_id, filename, enabled)
modpacks(id, source, project_id, version_id, installed_at)
java_installations(id, major_version, vendor, path)
backups(id, world_id, target, created_at, size_bytes)
git_repositories(id, world_id, path)
arweave_uploads(id, world_id, snapshot_commit_hash, tx_id, uploaded_at, cost_estimate)
skins(id, account_id, name, source_path, applied_at)
settings(key, value)
```

`refresh_token_ref` é uma referência ao keyring do SO, não o token em si — a tabela nunca guarda
segredo em texto puro. Conteúdo de mundo/mods nunca entra no banco, só metadados; os arquivos
ficam no filesystem (e, pra mundos versionados, dentro do próprio `.git`).

---

## Interfaces Internas Principais (traits Rust — esboço)

```rust
trait StorageProvider {
    fn upload(&self, data: &[u8], meta: &UploadMeta) -> Result<UploadReceipt>;
    fn download(&self, id: &str) -> Result<Vec<u8>>;
}

trait AuthenticationProvider {
    fn login(&self) -> Result<AuthSession>;
    fn refresh(&self, session: &AuthSession) -> Result<AuthSession>;
}
```

Detalhamento completo (campos de `UploadMeta`/`AuthSession`, tipos de erro) fica pra quando o
código dessas crates começar a ser escrito — aqui só fixamos o contrato mínimo que o resto do
sistema depende.

---

## Arquitetura Multiplataforma

Camada de abstração fina sobre 4 pontos reais de variação entre Linux/Windows/macOS:

- **Filesystem**: crate `dirs`/`directories` pra resolver diretórios de config/dados por SO —
  nunca caminho hardcoded.
- **Java/processo**: `std::process::Command` já abstrai o lançamento; o que varia é onde
  procurar Java já instalado (registry no Windows, `/usr/lib/jvm` no Linux etc.) — isolado num
  módulo `platform::java_locations()`.
- **Credenciais**: keyring nativo por SO via crate `keyring` (Windows Credential Manager /
  macOS Keychain / Linux Secret Service) — mesmo padrão que o TruthID já usa no desktop.
- **Empacotamento**: Tauri já resolve build multiplataforma (MSI/NSIS Windows, DMG macOS,
  AppImage/deb Linux) — não é um problema novo.

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
