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

**Proposta inicial de workspace Rust** (confirmada em parte na prática — `mcgit-java`,
`mcgit-db` e `apps/desktop` existem desde a Sessão 2, 2026-08-16; `mcgit-minecraft` e
`mcgit-instance` desde a Sessão 3; `mcgit-core` desde a Sessão 4, 2026-08-22; os demais crates
abaixo continuam só planejados, sem código ainda):

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
| Uso do Git (dentro do Git Engine) | Chamar o binário `git` do sistema vs biblioteca (`git2`/libgit2 em Rust) vs implementação própria mínima | **Binário `git` via subprocess** ✓ (decidido por análise, Sessão 1; primeira vez exercitada com código real na Sessão 4, 2026-08-22 — `git::init` via `std::process::Command`) — mais simples, sem custo de build/linking cross-platform de uma lib C |
| Estratégia de armazenamento de `.mca` | Git puro vs Git LFS vs camada própria por região/chunk antes do Git | **Git puro, sem LFS no MVP** ✓ (decidido por análise, Sessão 1) — benchmark já mostra Git puro + `git gc` resolvendo o caso comum; LFS adicionaria uma dependência de servidor que não se justifica ainda. Reabrir se um mundo real em produção mostrar o contrário |
| Compactação do repositório (`git gc`) | Depender do auto-gc padrão do Git vs o mcgit disparar `git gc`/repack periodicamente por conta própria | **mcgit dispara `git gc` por conta própria** ✓ (decidido, Sessão 1) — sem compactar, o `.git` cresce ~5.3M por snapshot mesmo mudando só 2-3 chunks de 960; com `git gc --aggressive`, 7 snapshots ficaram do tamanho de ~1 |
| Merge entre branches de mundo | Merge tradicional do Git vs não suportar merge (só criar/descartar branch) | **Em aberto** — não assumir que merge tradicional é seguro para arquivos de mundo |
| Banco de dados local | SQLite vs outra opção | **SQLite** ✓ (decidido por análise, Sessão 1) — guarda só metadados, nunca o conteúdo dos arquivos do mundo (isso continua sendo Git + filesystem). Schema proposto: §Schema do Banco Local |
| Biblioteca de acesso ao SQLite | `rusqlite` (síncrona) vs `sqlx` (assíncrona) vs `sea-orm` (ORM assíncrono) | **`sea-orm` 2.0** ✓ (decisão final, Sessão 2, revisando a escolha inicial de `rusqlite` da mesma sessão) — o usuário pediu pra reavaliar quando a segunda tabela (`instances`) apareceu no horizonte: com ~10 tabelas planejadas no PRD, trocar de fundação agora (2 tabelas) é mais barato que trocar depois (10 tabelas de SQL cru escritas à mão). `mcgit-db` reescrito por completo: entidades via `#[derive(DeriveEntityModel)]`, enums tipados via `DeriveActiveEnum` (`JavaSource` — string desconhecida no banco agora é erro real, não fallback silencioso), migrations via `sea-orm-migration`. `Db::open`/`open_in_memory` viraram `async`; `DatabaseConnection` é internamente um pool compartilhável, então o `Mutex<Db>` da Fase 1 original **foi removido** (ver §Débitos Técnicos — um dos 4 débitos originais já fechado). Dependências: `sea-orm`/`sea-orm-migration` com features `macros, sqlx-sqlite, runtime-tokio-rustls` (traz `sqlx` por baixo, mas a API de aplicação é a do SeaORM) |
| Migração de schema do SQLite | Arquivo único idempotente (`schema.sql`) vs `rusqlite_migration` vs ORM completo (`sea-orm`) | **Resolvido junto com a decisão acima (Sessão 2)** — a troca pra SeaORM já veio com `sea-orm-migration` embutido, então o meio-termo (`rusqlite_migration`) nunca chegou a ser necessário. Migrations vivem em `crates/mcgit-db/src/migrations/`, uma por tabela (`m20260816_000001_create_java_installations.rs` reaplica o schema que o `rusqlite` já usava, incluindo o índice único parcial do `is_default` — escrito como SQL cru via `execute_unprepared`, porque o construtor de schema do SeaORM não cobre bem valor-padrão-por-função (`datetime('now')`) nem índice parcial; escrito assim de propósito, não por falta de tentar o construtor). Rastreamento de quais migrations já rodaram fica numa tabela própria (`seaql_migrations`), confirmado funcionando contra o banco real |
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
  isso por versão no próprio piston-meta). **Implementado (Sessão 3, 2026-08-16)** — ver
  §Instância + Vanilla Install abaixo.
- Nunca depender só do "Java do sistema" — instâncias diferentes podem precisar de versões
  diferentes ao mesmo tempo (Java 17 pra 1.20.x, Java 21 pra 1.21.x).
- Baixar builds do **Eclipse Temurin/Adoptium** (OpenJDK redistribuível, sem os requisitos de
  licença da Oracle JDK) quando a versão necessária não estiver instalada.
- Guardar cada versão baixada numa pasta própria do mcgit, uma por major version, reaproveitada
  entre instâncias que precisam da mesma versão.
- Permitir apontar pra um Java já instalado manualmente, pro usuário avançado.

### Java Manager — implementado (Sessão 2, 2026-08-16)

Primeiro código de produto do projeto. Crate `crates/mcgit-java` (biblioteca pura, sem Tauri nem
SQLite como dependência) + app `apps/desktop` (Tauri 2 + React/TS, scaffolded via
`npm create tauri-app@latest`) conectados por `crates/mcgit-db`. Testado de ponta a ponta pela
GUI real: scan do sistema → listar LTS do Adoptium → baixar+verificar+extrair+instalar → marcar
padrão → persistência confirmada reabrindo o app.

- **Detecção** (`mcgit-java::detect`): varre locais por plataforma (`platform::linux` — só
  Linux implementado até agora, `windows`/`macos` ficam atrás de `#[cfg(target_os)]` mas sem
  corpo ainda, pra implementar/testar quando alguém rodar numa dessas plataformas) + `PATH`,
  resolve o binário real (arquivo direto ou `<candidato>/bin/java`), roda `java -version` e
  faz o parse da saída (`version_parse` — cobre Temurin/Oracle/Corretto/OpenJDK genérico e os
  dois esquemas de versão, antigo `1.8.0_x` e novo `21.x.x`).
- **API do Adoptium** (`mcgit-java::adoptium`): `GET /v3/info/available_releases` (lista de LTS)
  e `GET /v3/assets/latest/{feature_version}/hotspot?image_type=jdk&os=...&architecture=...`
  (asset mais recente) — contrato confirmado ao vivo, ver `CONTEXT.md` §Legal & Licensing pro
  histórico da pesquisa original. Mapeamento de nomenclatura: `macos`→`mac`, `x86_64`→`x64`.
- **Instalação** (`mcgit-java::install` + `archive`): download em streaming (`reqwest`, sem
  carregar o arquivo inteiro na memória) → verificação de checksum sha256 (antes de extrair,
  nunca depois) → extração por SO (`tar`+`flate2` em Linux/macOS, `zip` no Windows, cada um só
  compilado na plataforma certa via `#[cfg]`) → localização do binário na árvore extraída (busca
  limitada, já que o nome da pasta-raiz varia por versão, ex. `jdk-21.0.12+8`).
- **Persistência** (`mcgit-db`): tabela `java_installations` estendida (ver §Schema do Banco
  Local abaixo), acesso via SeaORM 2.0 (entidade + migration — trocado do `rusqlite` original
  logo em seguida na mesma sessão, ver a tabela de Decisões de Arquitetura).
- **Ponte Tauri** (`apps/desktop/src-tauri/src/commands/java.rs`): único lugar onde
  `mcgit-java` e `mcgit-db` se conectam, como a arquitetura exige — 6 comandos
  (`scan_system_java`, `list_java_installations`, `list_installable_java_versions`,
  `install_java`, `add_manual_java`, `set_default_java`) + evento `java://install-progress`.

**Simplificações conscientes, registradas como débito técnico leve** (ver §Débitos Técnicos):
a extração de arquivo dentro de
`install::download_and_install` roda de forma síncrona/bloqueante mesmo estando dentro de uma
`async fn` (alguns segundos de stall no executor durante a extração, imperceptível numa UI que já
mostra barra de progresso, mas não é o "jeito mais puro" de fazer); os eventos de progresso de
download não têm throttle (chunks de rede pequenos geram milhares de eventos por instalação) —
não travou nada até agora, mas vale revisar se afetar performance da UI real.

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
MultiMC já usam. **Nota**: esse design é a suposição de trabalho documentada aqui, não uma
decisão revalidada — `ROADMAP.md` ainda lista "compartilhado vs. isolado" como pergunta em
aberto. A implementação abaixo herdou essa suposição sem re-decidir.

### Instância + Vanilla Install — implementado (Sessão 3, 2026-08-16)

Segundo slice de código de produto do projeto, construído sobre a fundação SeaORM da sessão
anterior. Entrega "criar uma instância e ter o Vanilla baixado, verificado e pronto em disco" —
**não inclui lançar o jogo** (Game Runner, item separado da Fase 1, depende de sessão
autenticada da Minecraft Services API, e o login Microsoft está pausado — ver `PENDING.md` #1).
Testado de ponta a ponta pela GUI real: criar instância → resolver Java automaticamente
(reaproveitou o JDK 25 já instalado, sem baixar de novo) → baixar client jar + libraries +
assets → instância marcada `ready`.

- **Contrato do `piston-meta`** verificado ao vivo nesta sessão (não reaproveitado de memória
  — uma tentativa anterior de desenhar essa feature tinha "confirmado" o contrato, mas o plano
  foi sobrescrito antes de implementar e a informação se perdeu). Confirmado: `libraries[]` usa
  um array `rules` (`{action, os: {name}}`, última regra que bate manda) pra restringir por SO —
  manifestos modernos não usam mais `downloads.classifiers` de natives, então não precisa de
  lógica de "extrair classifier zip".
- **`crates/mcgit-minecraft`** (biblioteca pura, sem Tauri/SQLite, mesmo princípio do
  `mcgit-java`): `manifest` (busca+parse do manifesto e do JSON por versão), `libraries` (filtro
  de SO via `rules`), `assets` (índice de assets, URL/caminho de cache content-addressed),
  `install` (orquestra tudo — client jar, libraries e assets em paralelo com concorrência
  limitada via `buffer_unordered`, verificação sha1 por arquivo, cache-hit por hash antes de
  baixar de novo, throttle de progresso — resolvendo de saída o débito de throttle que o Java
  Manager tem hoje). Testes: fixtures reais capturadas ao vivo + um teste `#[ignore]` que baixa
  de verdade uma versão pequena (`rd-132211`, ~2009, ~49MB) contra a API real.
- **`crates/mcgit-instance`** (biblioteca pura, sem DB/Tauri/rede): scaffolding de
  `instances/<id>/minecraft/{7 subpastas}` + leitura/escrita de `instance.json`.
- **Tabela `instances`** (`mcgit-db`): primeira relação real do projeto via SeaORM
  (`belongs_to` pra `java_installations`, `ON DELETE SET NULL`) — ver §Schema do Banco Local.
- **Ponte Tauri** (`apps/desktop/src-tauri/src/commands/instance.rs`): 3 comandos
  (`list_instances`, `list_mc_versions`, `create_vanilla_instance`) + evento
  `instance://install-progress`. `create_vanilla_instance` orquestra manifesto → linha no banco
  (`status='installing'`) → scaffold de pastas → resolver Java (reaproveita instalação validada
  ou baixa uma nova, via `mcgit-java` sem modificar esse crate) → download do Vanilla → escreve
  `instance.json` final → marca `ready`. Falha em qualquer etapa marca `failed` em vez de deixar
  a linha desaparecer ou num estado ambíguo — linha e pasta parcial ficam em disco pra
  diagnóstico.
- **Um detalhe de compilação que vale registrar**: o macro de comandos do Tauri rejeitou a
  primeira versão de `download_libraries`/`download_assets` em `mcgit-minecraft::install` com
  erros de lifetime (`implementation of FnOnce is not general enough`) — closures assíncronas
  dentro de `.map()` capturando referências emprestadas de um `Vec<&T>`, um limite conhecido de
  inferência do rustc. Corrigido clonando os itens antes (`Vec<T>` em vez de `Vec<&T>`,
  `.into_iter()` em vez de `.iter()`).

**Validado ao vivo** (não só compilado): instância criada de verdade pra Minecraft 26.2 pela GUI
real — `client.jar` com exatamente 39.193.383 bytes (bate com o valor real capturado direto da
API), ~468MB de assets e ~77MB de libraries no cache compartilhado, `instance.json` com
`java_installation_path` resolvido, linha no banco com `status='ready'` e `java_installation_id`
preenchido.

### Tela inicial (home screen) — implementado (Sessão 7, 2026-08-22, continuação)

Item de UI puro, sem mudança em Rust/DB. `InstanceList.tsx` deixou de ser uma lista crua de
links e virou uma lista de **cards** (nome como link pra tela de detalhe, versão do MC, e um
botão "Jogar"). Decisões confirmadas com o usuário: (1) o mockup do `CONTEXT.md` mostra uma
única instância em destaque ("My Minecraft") — o card se repete por instância em vez de assumir
só uma, já que múltiplas instâncias já existem na prática; (2) o botão "Jogar" **aparece**
(bate com o mockup) mas fica desabilitado com "Available after Microsoft login" — o Game Runner
que de fato abriria o jogo ainda não existe (depende do login MS, `PENDING.md` #1). O texto do
botão também reflete o `status` da instância (`installing`/`failed` mostram texto próprio, só
`ready` mostra o aviso de login). Verificado ao vivo pela GUI real: card renderiza certo,
navegação pro detalhe continua funcionando, botão "Jogar" confirmado não-clicável (sem
navegação/efeito ao clicar).

---

## Git Engine

Módulo que versiona o conteúdo de um mundo (`saves/<folder>/`) com Git, sem exigir que o
jogador saiba que Git existe — a UI mostra "Ativar versionamento" / "Desativar versionamento"
por mundo, nunca `git init`/`commit`.

### Ativar/desativar versionamento — implementado (Sessão 4, 2026-08-22)

Primeiro slice de código do Git Engine (`mcgit-core`), aplicando a decisão da Fase 0 de chamar o
binário `git` do sistema via subprocess em vez de `git2`/libgit2.

- **`crates/mcgit-core`** (biblioteca pura, mesmo princípio de `mcgit-java`/`mcgit-minecraft`):
  só duas funções por enquanto — `git::init(world_dir)` (roda `git init` via
  `std::process::Command`) e `git::is_repository(world_dir)` (checa se `.git/` existe). `init` é
  deliberadamente idempotente sem checar `is_repository` primeiro — `git init` num repositório já
  inicializado é um no-op seguro por garantia do próprio Git, então checar antes seria uma
  validação redundante.
- **Tabela `worlds`** (`mcgit-db`): primeira tabela do projeto cuja chave estrangeira é
  `NOT NULL` (`instance_id`, `ON DELETE CASCADE`) — diferente de `instances.java_installation_id`
  (`SET NULL`, opcional). Faz sentido aqui porque um `world` sem instância não tem significado
  (não é "Java indisponível", é "não existe mais"); testado de verdade (`deleting_instance_
  cascades_to_worlds`), não só assumido pelo SQL. Índice único em `(instance_id, folder_name)`
  evita duas linhas pro mesmo mundo. `db_world::set_git_enabled` é find-or-create: cobre tanto
  "ativar pela primeira vez" (insere) quanto "reativar depois de desativar" (atualiza a mesma
  linha) sem duplicar.
- **Ponte Tauri** (`commands/world.rs`): `list_worlds` cruza duas fontes — o filesystem manda
  (`saves/*` com `level.dat` é a lista real de mundos que existem) e o banco só complementa
  `git_enabled` pros que já têm uma linha (mundo nunca versionado não tem linha, e isso é
  equivalente a `git_enabled=false`, não um erro). `enable_world_versioning` roda o `git init`
  bloqueante dentro de `spawn_blocking` antes de gravar no banco — mesmo cuidado que faltou no
  Java Manager (ver §Débitos Técnicos, item de extração síncrona), aplicado aqui desde o início.
- **"Desativar" é só um flag, nunca uma exclusão**: `disable_world_versioning` atualiza
  `git_enabled=false` e não toca no `.git/` nem em nenhum commit — histórico continua intacto no
  disco, e reativar mais tarde só volta o flag pra `true`. Decisão de produto deliberada (mundo
  do jogador não perde histórico por um toggle de UI).
- **UI**: botão de ativar/desativar por mundo na tela de detalhe da instância
  (`InstanceDetailScreen.tsx` → `WorldList.tsx`).

Escopo desta sessão é só o `git init` — criar snapshot (`git commit`), ver histórico e restaurar
ainda não existem (próximos itens da Fase 1, ver `PHASE.md`).

### Criar versão/snapshot — implementado (Sessão 4, 2026-08-22, continuação)

Segundo slice do Git Engine, direto em cima do `git init` acima. Entrega "salvar o estado atual
do mundo como uma versão", exposto na UI como "Save snapshot" — sem ainda ter "ver histórico"
nem "restaurar" (próximos itens do checklist, não fechados por essa feature: o hash do commit
não é escondido em lugar nenhum, e o histórico não é duplicado no SQLite, exatamente pra não
travar esses dois).

- **`crates/mcgit-core`**: a duplicação de "rodar um comando git e checar erro" (até então só em
  `init`) virou um helper privado `run(world_dir, args)` — o gatilho pra extrair foi real, não
  especulativo: `commit()` precisa de 5 invocações novas (`config` x2, `add`, `status`,
  `commit`, `rev-parse`). Novo enum `CommitOutcome { Created(String), NothingToCommit }` — "nada
  mudou desde o último snapshot" é tratado como resultado válido, não erro, então a UI consegue
  mostrar uma mensagem neutra em vez de um erro em vermelho pra uma ação que na verdade funcionou
  (só não teve o que fazer). Sequência de `commit()`: `ensure_identity` → `git add -A` → `git
  status --porcelain` (stdout vazio = nada mudou, retorna cedo) → `git commit -m <mensagem>` →
  `git rev-parse HEAD` (pega o hash, já que `git commit` não devolve isso de um jeito fácil de
  ler pelo código).
- **Identidade Git fixa por design**: `ensure_identity` configura `git config --local user.name
  mcgit` / `user.email mcgit@localhost` antes de todo commit — `--local` sempre ganha de
  `--global`/`--system` na resolução do próprio Git, então funciona numa máquina que nunca
  configurou identidade nenhuma (o caso comum: um jogador que nunca usou Git), sem nunca tocar
  na configuração pessoal do jogador se ele tiver uma. **Verificado ao vivo, não só assumido**:
  simulado um ambiente sem `HOME`/config global (`env -i HOME=/tmp/inexistente
  GIT_CONFIG_NOSYSTEM=1 git commit`) — falha sem identidade nenhuma ("Please tell me who you
  are"), funciona depois de aplicar os mesmos dois `git config --local` que `ensure_identity`
  roda. Essa identidade vai aparecer literalmente assim (`mcgit <mcgit@localhost>`) no `git log`
  quando "Modo Avançado" existir — decisão confirmada com o usuário, não assumida.
- **Sem dependência nova**: `message: &str` nunca chega vazio em `commit()` — `git commit -m ""`
  falha de propósito ("Aborting commit due to empty commit message"), então quem garante
  não-vazio é o frontend (`new Date().toLocaleString()`, uma linha de JS), não o Rust. Isso
  mantém `mcgit-core` só com `thiserror` como dependência, mesmo depois dessa feature.
- **Testes** (`mcgit-core`, mesmo estilo dos 3 de `init`): commit com mudança real → `Created`
  com hash de 40 hex chars; commit repetido sem mudança → `NothingToCommit`; repositório
  recém-`init`ado e vazio → `NothingToCommit` na primeira tentativa. 6/6 testes verdes.
- **Ponte Tauri**: `SnapshotDto { created, commit_hash }` + `create_world_snapshot`, mesmo
  formato de `enable_world_versioning` (roda `mcgit_core::git::commit` dentro de
  `spawn_blocking`). Sem mudança em `AppState`, sem escrita no banco, sem migration.
- **UI**: `SaveSnapshotForm.tsx` (novo, espelha `AddManualJavaForm.tsx` — inline `<form>`, sem
  modal) com um campo de texto opcional; se vazio, usa o timestamp como mensagem. Canal de
  feedback novo em `InstanceDetailScreen.tsx` (`status`, texto normal) separado do `error`
  (vermelho) — decisão deliberada pra "nada mudou" não parecer uma falha.

**Validado ao vivo pela GUI real** (mundo fake criado manualmente, mesma limitação de sempre —
Game Runner ainda não existe): habilitar versionamento → `.git` criado → "Save snapshot" com
mensagem customizada → commit real confirmado via `git log` (autor e mensagem corretos) → "Save
snapshot" de novo sem mudança → "Nothing changed since the last snapshot." (não erro), nenhum
commit vazio criado → campo em branco + mudança real → mensagem por timestamp usada de fato.

### Ver histórico de versões — implementado (Sessão 5, 2026-08-22)

Terceiro slice do Git Engine. Entrega uma timeline amigável dos snapshots de um mundo — não um
dump cru de `git log` — lendo o Git ao vivo, exatamente como o item anterior deixou preparado
("o histórico não é duplicado no SQLite").

- **`crates/mcgit-core`**: `log(world_dir) -> Result<Vec<Snapshot>, GitError>`, reaproveitando o
  chokepoint `run()`. Formato pedido ao Git: `git log --pretty=format:%H\x1f%aI\x1f%s` — campos
  separados por `\x1f` (unit separator, um caractere de controle que nunca aparece numa
  mensagem de commit normal), um commit por linha (Git insere `\n` entre commits sozinho nesse
  modo de formatação), `%s` é sempre uma única linha por definição (só o assunto do commit), o
  que torna o parse por linha seguro. Dois casos viram lista vazia, não erro: mundo nunca
  `git init`ado (checado via `is_repository` antes de rodar `git log`) e mundo versionado sem
  nenhum snapshot ainda (Git recusa `git log` num repo sem commits com "does not have any
  commits yet" no stderr — esse texto específico é capturado e vira `Ok(vec![])`, mesmo espírito
  do `CommitOutcome::NothingToCommit`). Ordem de retorno já vem do próprio Git: mais recente
  primeiro. Sem dependência nova — `date` fica como string ISO 8601 crua (`%aI`), igual ao
  padrão já usado de empurrar formatação de data pro frontend.
- **Testes** (`mcgit-core`, mesmo estilo dos anteriores): mundo nunca inicializado → vazio; repo
  inicializado sem commits → vazio; 1 commit → 1 `Snapshot` com hash de 40 chars; 2 commits →
  ordem mais-recente-primeiro confirmada. 10/10 testes verdes no crate.
- **Ponte Tauri**: `list_world_history` (mesmo formato dos outros comandos de mundo —
  `spawn_blocking`, path resolvido via `scaffold::instance_root(...).join("minecraft").join("saves")`,
  erro tipado convertido pra `String` no boundary). O `SnapshotDto` que já existia (resultado de
  `create_world_snapshot`: `created`/`commit_hash`) foi renomeado pra `SnapshotResultDto` pra
  abrir espaço pro novo `SnapshotDto` do histórico (`hash`/`date`/`message`) — só usado dentro do
  próprio arquivo, renomear não teve efeito em mais nada.
- **UI**: botão "Show history"/"Hide history" por mundo em `WorldList.tsx` (só quando
  `git_enabled`), carregado sob demanda — só dispara `listWorldHistory` no primeiro clique que
  expande, resultado fica guardado em `historyByWorld` (novo estado em
  `InstanceDetailScreen.tsx`) pra não rebuscar toda vez que expande/recolhe. Decisões de UX
  confirmadas com o usuário: botão sob demanda (não sempre visível) e mostrar todos os
  snapshots de uma vez (sem paginação por ora). Novo componente `WorldHistory.tsx` (recebe
  `snapshots` via prop, sem `invoke` direto — mesmo padrão de `SaveSnapshotForm`), mostra hash
  curto (7 chars), data via `new Date(date).toLocaleString()`, e a mensagem.
- **Bug encontrado e corrigido durante a verificação ao vivo**: salvar um snapshot novo com o
  painel de histórico já aberto não atualizava a lista (ficava mostrando o estado antigo até
  fechar/reabrir o painel). Corrigido em `handleSaveSnapshot`: se `historyByWorld[folderName]`
  já foi carregado antes (painel já foi aberto ao menos uma vez) e o snapshot foi criado de
  verdade, o histórico daquele mundo é recarregado automaticamente.

**Validado ao vivo pela GUI real** (mesma técnica de automação da Sessão 4 — `GDK_BACKEND=x11` +
`xdotool` + `spectacle`, mundo fake criado na pasta `saves/` de uma instância já existente,
removido ao final): botão de histórico ausente num mundo nunca versionado; "No snapshots yet."
num mundo versionado sem nenhum snapshot; 1 snapshot aparece com hash/data/mensagem corretos;
2º e 3º snapshots aparecem na ordem certa (mais recente primeiro); salvar um snapshot com o
painel já aberto atualiza a lista sozinho, sem precisar fechar/reabrir.

### Restaurar uma versão — implementado (Sessão 6, 2026-08-22)

Última peça do ciclo básico do Git Engine (ativar → snapshot → histórico → **restaurar**). O
`CONTEXT.md` já especificava dois requisitos de segurança antes desta sessão (seção "Snapshot /
History / Restore" + "Security Requirements"): checagem de mundo aberto ("when possible") e
checkpoint de segurança antes de restaurar ("non-negotiable"). Os dois foram fechados de
verdade, não adiados.

- **Nunca destrutivo**: `restore()` não é um `git reset --hard` (que rebobinaria o histórico e
  tornaria commits mais novos órfãos). Em vez disso: `git checkout <hash> -- .` traz os
  arquivos de volta ao estado antigo (atualiza working tree **e** index), e isso é gravado como
  um commit novo em cima do histórico existente — nada é apagado, o próprio restore é sempre
  desfazível restaurando de novo.
- **Checagem de mundo aberto, implementada agora** (decisão confirmada com o usuário: não
  adiar pro Game Runner). Minecraft trava um lock exclusivo em `session.lock` dentro da pasta
  do mundo enquanto ele está carregado — `is_currently_open()` tenta adquirir esse mesmo lock
  (`std::fs::File::try_lock`/`unlock`, API nativa do Rust estabilizada recentemente — **zero
  dependência nova**, nem precisou do `fs4` cogitado no planejamento) e trata "não consegui
  travar" como "mundo aberto". Se o arquivo nem existe ainda (mundo nunca aberto), retorna
  "não aberto" direto, sem tentar nada.
- **Checkpoint de segurança + restauração, ambos via `commit()` já existente**:
  `restore(world_dir, commit_hash)` primeiro salva o que estiver pendente (`commit(world_dir,
  "Backup before restoring")` — `NothingToCommit` se já estava tudo salvo, não é problema),
  depois faz o checkout e commita o resultado (`commit(world_dir, "Restored to <hash curto>")`
  — também pode ser `NothingToCommit`, se restaurar pro estado em que já estava). `RestoreOutcome
  { backup, restore }` carrega os dois `CommitOutcome`.
- **`RestoreError`** (novo, em `types.rs`): `WorldCurrentlyOpen` (não é um erro de Git, por
  isso não vive em `GitError`), mais `Git`/`Io` via `#[from]` pra propagar os erros de baixo
  sem boilerplate.
- **Testes** (`mcgit-core`): restaurar traz o conteúdo antigo de volta e cria um commit;
  restaurar com mudança pendente faz backup primeiro; restaurar pro estado atual não cria
  nenhum commit novo; hash inválido propaga erro; `session.lock` travado por outro `File` no
  próprio teste (simulando o Minecraft real) bloqueia o restore sem alterar nada. 16/16 testes
  verdes no crate.
- **Ponte Tauri**: `restore_world_version` (mesmo formato dos outros comandos de mundo) +
  `RestoreDto { backup_created, restored }`.
- **UI**: confirmação **inline** por snapshot em `WorldHistory.tsx` (não um modal, mantendo a
  convenção do projeto) — clicar "Restore" mostra o aviso do mockup do `CONTEXT.md` ("This will
  replace the world's current state.") com "Cancel"/"Create Backup and Restore" na própria
  linha do snapshot. `InstanceDetailScreen.tsx` monta a mensagem de status a partir do
  resultado ("Created a backup and restored to `<hash>`." / "Restored to `<hash>`." / "Already
  at this version.") e recarrega o histórico daquele mundo (mesmo padrão de auto-refresh da
  feature anterior).
- **Achado real durante a verificação ao vivo**: o próprio `session.lock` (criado pra checagem
  de mundo aberto) estava sendo pego pelo `git add -A` do `commit()` e virando um arquivo
  rastreado — não quebra o restore, mas suja o histórico com ruído de um arquivo transitório.
  Primeira tentativa de correção (um `.gitignore` rastreado, commitado automaticamente no
  `git init`) criava um problema pior: uma entrada "Start versioning this world" aparecendo na
  timeline do jogador como se fosse um snapshot real, contrariando o próprio princípio "timeline
  amigável, não ruído de sistema". Correção final: `.git/info/exclude` (mecanismo nativo do Git
  pra exclusões só-locais) — nunca precisa de commit, então um mundo recém-versionado continua
  com histórico zerado até o jogador salvar de verdade, exatamente como antes.

**Validado ao vivo pela GUI real** (mesma técnica de sempre, dois mundos fake diferentes —
um reaproveitado de sessões anteriores, outro criado do zero pra confirmar a exclusão do
`session.lock` desde o primeiro `git init`): 3 snapshots criados, restaurar pro mais antigo
(sem mudança pendente → sem backup, só o commit de restore) confirmado por conteúdo de arquivo
e `git log` reais; restaurar de novo com uma mudança pendente no meio (→ backup real, conteúdo
do backup conferido via `git show`); restaurar pro estado atual → nenhum commit novo,
"Already at this version."; `session.lock` travado por um processo `flock` externo simulando o
Minecraft real → restore bloqueado com a mensagem certa, zero commits criados, liberado o lock
→ restore volta a funcionar; mundo novo confirmando que `session.lock` nunca é rastreado mesmo
existindo no disco no momento do snapshot.

### Deletar uma versão — implementado (Sessão 7, 2026-08-22)

Fecha o ciclo básico do Git Engine (ativar → snapshot → histórico → restaurar →
**deletar**). Ao contrário de tudo que veio antes, esta é a primeira operação de verdade
destrutiva: o snapshot deletado deixa de existir.

- **Por que não `git rebase`/`filter-branch`**: essas ferramentas recalculam e reaplicam o
  *diff* de cada commit sobre uma nova base — exatamente o tipo de operação que o próprio
  `PHASE.md` (Fase 6 — Branching) já registra como não validado como seguro pra arquivos
  binários de mundo ("não assumir que merge é seguro"). `delete_snapshot()` evita esse risco
  por completo com uma técnica diferente: um commit do Git já é uma **foto completa** dos
  arquivos (a "árvore"), não um delta — então "deletar" um commit do meio da cadeia é só
  religar o ponteiro de pai dos commits seguintes direto pro commit anterior ao deletado,
  reconstruindo cada um com `git commit-tree` reaproveitando a MESMA árvore que ele já tinha.
  Nunca se pergunta ao Git "como resolver essa mudança" — não há diff, não há merge, não há
  conflito possível, nem em binário.
- **Validado manualmente antes de virar código**: nesta sessão, os 4 casos abaixo foram
  testados na mão num repositório Git descartável (não só pensados em teoria) antes de
  qualquer linha de Rust ser escrita:
  1. **Commit do meio** (A→B→C, deletar B): vira A→C, C reconstruído com a mesma árvore/data/
     mensagem (hash novo, porque o pai mudou), arquivos no disco intocados.
  2. **Commit mais recente/topo**: precisa de `git reset --hard` no final pra atualizar os
     arquivos de verdade — é o único caso que muda algo no disco.
  3. **Raiz com descendentes**: o primeiro descendente vira uma nova raiz (`commit-tree` sem
     `-p`), resto da cadeia intacto.
  4. **Único commit existente** (raiz e topo ao mesmo tempo): `git update-ref -d` remove a
     referência do branch — o repo volta pro mesmo estado de "inicializado, zero commits" que
     `log()` já trata (mesma mensagem "does not have any commits yet" do Git), arquivos no
     disco ficam como estão.
- **Datas preservadas**: cada `commit-tree` roda com `GIT_AUTHOR_DATE`/`GIT_COMMITTER_DATE`
  setados pro valor original daquele commit — um snapshot sobrevivente nunca muda de data na
  timeline só porque um outro foi deletado em algum ponto da cadeia.
- **Mesma checagem de mundo aberto** (`is_currently_open`, já existente) do `restore()`,
  aplicada antes de qualquer coisa.
- **`DeleteError`** (novo, em `types.rs`): mesmo formato do `RestoreError`
  (`WorldCurrentlyOpen` + `Git`/`Io` via `#[from]`).
- **Testes** (`mcgit-core`): os 4 casos acima, mais mundo travado e hash inválido. 22/22
  verdes no crate.
- **Ponte Tauri**: `delete_world_snapshot` (mesmo formato dos outros comandos de mundo,
  retorna só sucesso/erro — não há ambiguidade de resultado como em `restore`).
- **UI**: botão "Delete" ao lado do "Restore" em `WorldHistory.tsx`, confirmação inline (nunca
  modal). Decisão confirmada com o usuário: deletar o snapshot mais recente **é permitido**
  (não fica restrito a snapshots antigos), mas com aviso diferente, já que também reseta os
  arquivos do mundo. **Achado durante a verificação ao vivo**: esse aviso ("...reset pro estado
  do snapshot anterior") ficava impreciso quando o snapshot deletado era o único que existia —
  não há "anterior" nesse caso. Corrigido com um terceiro texto específico ("This is your only
  snapshot. Deleting it removes all version history for this world (your current files won't
  be touched).").

**Validado ao vivo pela GUI real** (mesma técnica de sempre): 3 snapshots criados; deletar o
do meio (conteúdo e data do snapshot seguinte confirmados intocados); deletar o mais recente
(aviso específico confirmado, conteúdo do arquivo revertido de verdade pro snapshot anterior);
deletar o único snapshot restante ("No snapshots yet." depois, aviso de "único snapshot"
confirmado antes); `session.lock` travado por `flock` externo → delete bloqueado, histórico
intacto, liberado o lock → delete volta a funcionar.

---

## Schema do Banco Local (SQLite) — proposta inicial

```sql
instances(id, name, mc_version, loader, loader_version, java_version, jvm_args, created_at)
accounts(id, ms_account_id, minecraft_uuid, username, refresh_token_ref, last_login_at)
worlds(id, instance_id, name, path, git_enabled, created_at)
mods(id, instance_id, source, project_id, version_id, filename, enabled)
modpacks(id, source, project_id, version_id, installed_at)
java_installations(id, major_version, vendor, path, source, is_default, created_at)
backups(id, world_id, target, created_at, size_bytes)
git_repositories(id, world_id, path)
arweave_uploads(id, world_id, snapshot_commit_hash, tx_id, uploaded_at, cost_estimate)
skins(id, account_id, name, source_path, applied_at)
settings(key, value)
```

`refresh_token_ref` é uma referência ao keyring do SO, não o token em si — a tabela nunca guarda
segredo em texto puro. Conteúdo de mundo/mods nunca entra no banco, só metadados; os arquivos
ficam no filesystem (e, pra mundos versionados, dentro do próprio `.git`).

**`java_installations` implementada e estendida** (Sessão 2, 2026-08-16 — a única tabela com
código real até agora): ganhou `source` (`'managed'|'detected'|'manual'` — necessário porque
detecção, download e entrada manual escrevem na mesma tabela e não podem se confundir) e
`is_default` (boolean), com **unicidade de "só um padrão" garantida por um índice único parcial
do SQLite** (`CREATE UNIQUE INDEX ... WHERE is_default = 1`), não por disciplina da aplicação.
`path` é `UNIQUE` e é a chave de dedup entre os três fluxos (upsert via busca-então-branco
inserir/atualizar). Acesso via SeaORM 2.0 (entidade + migration) — decisão final registrada na
tabela de Decisões de Arquitetura abaixo, revisada da escolha inicial (`rusqlite`) na mesma
sessão.

**`instances` implementada** (Sessão 3, 2026-08-16) — schema real, diferente do esboço acima:

```sql
instances(id, name, mc_version, loader, loader_version, java_installation_id, jvm_args, status, created_at)
```

Diferenças deliberadas do esboço original: (1) `java_installation_id` é uma FK real pra
`java_installations(id)` (`ON DELETE SET NULL`) em vez de um `java_version` solto — primeiro uso
de `DeriveRelation`/`belongs_to` no projeto; (2) coluna `status`
(`'installing'|'ready'|'failed'`) adicionada — não existia no esboço, mas é necessária porque a
instalação do Vanilla é uma operação de rede longa e interrompível, e uma linha existe antes de
qualquer arquivo existir em disco; (3) `loader` é um enum tipado (`DeriveActiveEnum`, só
`'vanilla'` por enquanto) em vez de `TEXT` livre, mesmo padrão de `JavaSource` — Fase 3
(Fabric/Forge/NeoForge) só precisa adicionar variantes. Pasta em disco nomeada pelo `id`
autoincrement do banco, não UUID (sem precedente de UUID em nenhuma outra tabela do projeto).

**`worlds` implementada** (Sessão 4, 2026-08-22) — schema real, diferente do esboço acima:

```sql
worlds(id, instance_id, folder_name, git_enabled, created_at)
```

Diferenças deliberadas do esboço original: (1) `folder_name` no lugar de `name`+`path` — o nome
da pasta dentro de `saves/` já é o identificador (não precisa de path absoluto guardado, a
instância já sabe onde fica sua própria pasta); (2) `instance_id` é `NOT NULL` com
`ON DELETE CASCADE` (não `SET NULL` como em `instances.java_installation_id`) — um `world` órfão
de instância não faz sentido, ver §Git Engine; (3) índice único em `(instance_id, folder_name)`
em vez de um `id`/path global único, já que o mesmo nome de pasta pode existir em instâncias
diferentes.

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

Registrados a partir da implementação do Java Manager (Sessão 2, 2026-08-16 — primeiro código
de produto do projeto; o benchmark de Git da Fase 0 era ferramenta de pesquisa descartável, não
conta aqui):

- **`platform::windows`/`platform::macos` não implementados** — só `platform::linux` existe;
  os outros dois ficam atrás de `#[cfg(target_os)]` sem corpo. Não bloqueia nada no Linux (o
  compilador nem tenta compilá-los), mas precisa ser feito antes de rodar em Windows/macOS.
- **Extração de arquivo bloqueia dentro de uma `async fn`** — `install::download_and_install`
  roda a extração (`tar`/`zip`) de forma síncrona mesmo estando numa função `async`, sem
  `spawn_blocking`. Pode travar o executor do Tokio por alguns segundos numa instalação grande.
  Não gerou problema observado até agora (a UI já mostra barra de progresso, então uns segundos
  de "Extracting" parado não chama atenção), mas é candidato a revisão se a experiência real
  mostrar travamento perceptível de outras telas enquanto uma instalação roda.
- **Eventos de progresso de download sem throttle** — chunks de rede pequenos (observado: 1-16KB
  por chunk num download de ~200MB) geram milhares de eventos `java://install-progress` por
  instalação. Funcionou sem problema no teste real, mas não tem limite de taxa — se a UI ficar
  pesada com isso, precisa agrupar por tempo (ex.: só emitir a cada 100ms) ou por percentual.
- ~~`Mutex<Db>` travado direto nas funções `async` dos comandos Tauri, sem `spawn_blocking`~~ —
  **resolvido (Sessão 2, migração pra SeaORM)**: `DatabaseConnection` já é um pool interno seguro
  pra chamar de várias tasks assíncronas ao mesmo tempo, o `Mutex` virou desnecessário e foi
  removido de `AppState`.
- **"Scan for Java" não redescobre instalações gerenciadas (`source='managed'`) depois de um
  reset de banco** — descoberto testando a migração pro SeaORM na prática (o banco de teste foi
  resetado, o JDK 25 continuava em disco, mas "Scan" não achou nada, porque ele só varre locais
  do *sistema* — `/usr/lib/jvm`, `JAVA_HOME`, `PATH` — nunca a pasta gerenciada do próprio mcgit).
  Contornado na hora com "Add manual Java" apontando pro binário direto (funciona, mas registra
  como `source='manual'`, não `'managed'` — semanticamente errado). Não bloqueou a verificação
  desta sessão, mas é um gap real de UX: um usuário que reinstale o app (ou perca o banco por
  qualquer motivo) não teria como recuperar instalações gerenciadas sem apontar manualmente pra
  cada uma. Correção natural: `scan_system_java` também varrer `state.java_dir` além dos locais
  de sistema. Não corrigido nesta sessão — fora do escopo do plano (migração de fundação, não
  correção de feature).
- **`delete_instance` não existe** (Sessão 3, 2026-08-16) — decisão de escopo deliberada, não
  esquecimento: o plano da feature Instância + Vanilla Install cobria só criar/listar. Uma
  instância `failed` ou que o usuário não quer mais fica no banco e em disco sem jeito de
  remover pela UI ainda.

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
