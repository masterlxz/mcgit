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
| Merge entre branches de mundo | Merge tradicional do Git vs não suportar merge (só criar/descartar branch) | **Investigado — seguro contra corrupção/perda silenciosa, mas granularidade grosseira** ✓ (Sessão 8, continuação, 2026-09-01) — o Git nunca corrompe nem perde dado sem avisar, mas o conflito é por **arquivo inteiro** (uma região `.mca` = 512×512 blocos), não por chunk/bloco: duas mudanças sem nenhuma sobreposição real dentro da mesma região ainda assim forçam escolher uma versão inteira da região, descartando a outra por completo. Resolver isso de verdade (por chunk) é trabalho da Fase 4. Ver §Git Engine, subseção "Investigação: merge entre branches" |
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

### Criar/trocar de branch — implementado (Sessão 8, 2026-09-01)

Primeiro item da Fase 6 (Branching), puxado pra frente do roadmap a pedido do usuário (ver nota
de reordenação em `PHASE.md`/`OVERVIEW.md`): priorizar aprofundar versionamento de mundo +
branches + GUI antes do resto do escopo do launcher. Escopo: só criar e trocar de branch —
comparação entre branches e a investigação de merge continuam em aberto, não fazem parte desta
leva.

- **Quatro funções novas em `git.rs`**: `current_branch()` (`git branch --show-current`,
  funciona mesmo num repo sem nenhum commit — lê o ref simbólico do HEAD, não depende do grafo
  de commits), `list_branches()` (`git branch --format=%(refname:short)`), `create_branch()`
  (`git checkout -b <nome>`) e `switch_branch()` (troca pra uma branch já existente).
- **`create_branch` não precisa de checkpoint nem de checagem de mundo aberto**: como a nova
  branch nasce apontando pro mesmo commit atual, nenhum arquivo no disco muda de conteúdo —
  diferente de `switch_branch`, que pode trazer um conteúdo diferente pra árvore de trabalho, daí
  precisar das duas mesmas guardas do `restore()`/`delete_snapshot()`.
- **Checkpoint automático antes de trocar** (confirmado com o usuário via `AskUserQuestion`):
  `switch_branch()` sempre roda `commit(world_dir, "Checkpoint before switching branches")`
  antes do `git checkout <nome>` — reaproveita o `commit()` já existente, mesmo espírito do
  backup automático do `restore()`. Sem isso, trocar de branch com mudança pendente
  frequentemente bateria no erro cru do Git ("local changes would be overwritten by checkout"),
  já que arquivos de mundo quase sempre se sobrepõem entre branches.
- **Mesma checagem de mundo aberto** (`is_currently_open`) do `restore()`/`delete_snapshot()`,
  reaproveitada diretamente por estar no mesmo módulo.
- **`BranchError`** (novo, em `types.rs`): mesmo formato de `RestoreError`/`DeleteError`
  (`WorldCurrentlyOpen` + `Git`/`Io` via `#[from]`).
- **Sem migration nova no `mcgit-db`**: a branch atual é sempre derivada ao vivo via
  `git branch --show-current`, mesma filosofia de `log()`/`is_repository()` — a tabela `worlds`
  não guarda nenhum estado de Git além de `git_enabled`.
- **Testes** (`mcgit-core`): criar branch (troca pra ela, branch antiga continua listada), criar
  branch num repo sem nenhum commit, trocar com checkpoint (mudança pendente vira commit,
  conteúdo bate com o da branch de destino), trocar sem checkpoint (sem mudança pendente),
  trocar bloqueada por mundo aberto (mesmo truque de `flock` externo dos testes de `restore`/
  `delete`), listagem refletindo todas as branches criadas. 28/28 verdes no crate.
- **Ponte Tauri**: `list_world_branches` (zipa `current_branch` + `list_branches` num único
  `BranchDto { name, is_current }` por branch), `create_world_branch` (cria e já retorna a lista
  atualizada, evitando um round-trip extra do frontend), `switch_world_branch` (retorna
  `SwitchDto { checkpoint_created, branch }`).
- **UI**: novo componente `WorldBranches.tsx`, espelhando `WorldHistory.tsx` — formulário de
  nome de branch nova (mesmo padrão do `SaveSnapshotForm.tsx`), lista de branches com a atual
  marcada, botão "Switch" por branch não-atual com confirmação inline (nunca modal) antes de
  trocar, já que trocar de branch muda visivelmente os arquivos do mundo pro jogador — mesma
  razão pela qual `restore`/`delete` também confirmam inline mesmo sem serem destrutivos no
  nível do Git.
- **Seção de branches só em Modo Avançado** (confirmado com o usuário via `AskUserQuestion`):
  `WorldList.tsx` usa `useAdvancedMode()` pra esconder a seção inteira ("Show branches" +
  `WorldBranches`) em Modo Básico — diferente do histórico de snapshots, que fica sempre
  visível e só esconde detalhes internos. Confirma o que a nota do Modo Básico/Avançado (Sessão
  7) já antecipava: branches são um recurso de Git ainda escondido, "pra quando forem
  implementadas, Fase 6+".
- **Histórico se mantém sincronizado após trocar de branch**: `git log` segue o HEAD atual, então
  o conteúdo que `list_world_history` retorna muda quando a branch muda. `handleSwitchBranch`
  em `InstanceDetailScreen.tsx` re-busca o histórico daquele mundo se o painel já estava aberto
  — mesma classe de bug de painel desatualizado já corrigida na Sessão 5 (lá era ao salvar um
  snapshot novo), prevenida desde o início desta vez em vez de descoberta depois.

**Verificação ao vivo pela GUI não foi feita nesta sessão**: a tela de desenvolvimento tinha uma
partida de xadrez ativa no navegador, que roubava o foco da janela do mcgit repetidamente
(inclusive depois de `windowfocus`/`windowraise`/`_NET_WM_STATE_ABOVE` via `xprop` — o navegador
é um cliente Wayland nativo, fora do alcance do `xdotool`/XWayland usado nas sessões anteriores),
tornando cliques e mesmo navegação por teclado pouco confiáveis. Confirmado com o usuário via
`AskUserQuestion`: aceitar os 28 testes automatizados do `mcgit-core` + typecheck limpo do
frontend como verificação desta vez, sem a checagem visual ao vivo que todas as sessões
anteriores fizeram. Vale rodar essa checagem manualmente numa sessão futura, com a tela livre,
antes de considerar este item no mesmo padrão de confiança dos anteriores.

**Atualização (mesma sessão, continuação)**: a checagem ao vivo pendente acima foi feita — a
tela ficou livre (o xadrez não estava mais aberto) durante o trabalho da "Comparação entre
branches" logo a seguir, e a verificação cobriu criar/trocar branch retroativamente também. Ver
detalhes na subseção "Comparação entre branches" abaixo.

---

### Comparação entre branches — implementado (Sessão 8, continuação, 2026-09-01)

Segundo item da Fase 6, direto na sequência de "Criar/trocar de branch". Escopo confirmado com
o usuário via `AskUserQuestion`: só compara a branch atual contra outra branch (não snapshots do
histórico), e mostra uma lista de arquivos alterados com tamanho em bytes antes/depois —
**nenhum diff de conteúdo**, já que a maioria dos arquivos de mundo (`.mca`, `level.dat`) é
binária; diff de conteúdo de verdade é trabalho da Fase 4 (Minecraft-Aware World Diffing), que
interpreta o formato.

- **`diff_branches(world_dir, from, to)`** (novo em `git.rs`): roda
  `git diff --name-status <from> <to>` pra saber quais arquivos mudaram e como (`A`/`M`/`D`);
  pra cada um, uma chamada própria de `git cat-file -s <ref>:<path>` (de cada lado que existir)
  dá o tamanho em bytes — evita depender do formato de texto humano do `git diff --stat` (que já
  mostra `Bin X -> Y bytes` pra binários, mas pensado pra terminal, não pra parsear de forma
  robusta), mantendo o mesmo estilo do resto do módulo: comandos `git` pequenos e de propósito
  único.
- **Sem detecção de rename**: não passa `-M`, não configura `diff.renames` — um arquivo
  renomeado aparece como delete + add separados, mais simples de tratar do que reconstruir
  renames, e Git não ativa isso por padrão de qualquer forma.
- **Ponte Tauri**: `diff_world_branches` compara a branch atual do mundo (derivada via
  `current_branch()`) contra uma branch informada — não pede duas branches explícitas, já que a
  UI sempre sabe qual é a atual.
- **UI**: botão "Compare" ao lado do "Switch" já existente em `WorldBranches.tsx`, por branch
  não-atual. Expande inline (nunca modal) a lista de arquivos: status + caminho + tamanho antes
  → depois (`"1.2 KB → 1.4 KB"`, `"new file, 340 bytes"`, `"deleted, was 890 bytes"`). Comparar
  não muda nada, então não precisa de confirmação como `Switch`/criar branch precisam.
- **Testes** (`mcgit-core`): arquivo adicionado, arquivo modificado, arquivo deletado, branches
  idênticas (lista vazia). 32/32 verdes no crate.

**Dois bugs reais de painel desatualizado encontrados e corrigidos durante a verificação ao
vivo** (mesma classe do bug já corrigido na Sessão 5 pro histórico, e a mesma que motivou o
cuidado proativo já tomado ao implementar `switch_branch` pro histórico logo acima):
1. O painel de branches (`main (current)` etc.) não existia de verdade — não é bug, é
   comportamento correto do Git: uma branch só existe como ref depois do primeiro commit. Mas o
   painel, se já estava aberto, não se atualizava sozinho depois desse primeiro snapshot.
2. O painel de comparação ficava mostrando dados obsoletos depois de um novo snapshot na branch
   atual (o conteúdo comparado mudou, mas o painel não sabia).

Corrigido em `InstanceDetailScreen.tsx`: `handleSaveSnapshot` agora re-busca `branchesByWorld` e
`diffsByWorld` daquele mundo se já estavam carregados (mesmo padrão já usado pro histórico).
Além disso, `handleCreateBranch`/`handleSwitchBranch` limpam qualquer comparação aberta, já que
trocar a branch atual invalida semanticamente uma comparação computada contra a branch atual
antiga.

**Verificado ao vivo pela GUI real** (tela livre desta vez — o xadrez do início da sessão não
estava mais aberto): mundo versionado, primeiro snapshot salvo, branch "experiment" criada e
trocada (confirmado: `list_branches`/`current_branch` corretos antes e depois), mudança real no
mundo (arquivo modificado + arquivo novo) commitada na branch "experiment", comparação com
"main" mostrando corretamente `modified — level.dat — 35 bytes → 19 bytes` e
`deleted — r.0.0.mca — deleted, was 31 bytes`; painel de branches e de comparação confirmados
se auto-atualizando depois de um snapshot novo (a correção acima, testada ao vivo, não só nos
testes automatizados); comparação confirmada limpa depois de trocar de volta pra "main". Também
serviu como verificação retroativa de "Criar/trocar de branch" (pendente da sessão anterior).

---

### Investigação: merge entre branches — feita (Sessão 8, continuação, 2026-09-01)

Terceiro e último item da Fase 6, pedido explicitamente como investigação (não implementação
ainda). Seguindo o mesmo método já usado antes de implementar `delete_snapshot` (validar na mão
num repositório Git descartável antes de decidir qualquer coisa), 5 experimentos reais foram
rodados — não é uma resposta teórica.

**Resultado principal (revisado — ver correção abaixo): merge tradicional do Git nunca corrompe
nem perde dado silenciosamente, desde que o mcgit nunca tente resolver um conflito de conteúdo
sozinho — o próprio Git já se recusa a fazer isso. Mas isso não é a mesma coisa que "merge é
seguro pra combinar trabalho em paralelo": a granularidade do conflito é o arquivo inteiro (uma
região `.mca` inteira, 512×512 blocos), não o chunk/bloco que realmente mudou.** A preocupação
original (registrada desde a Fase 0) tratava merge como uma coisa só ("merge é seguro ou não?").
Os experimentos mostram que a resposta certa é por caso — e o experimento 6 abaixo, feito depois
de o usuário questionar diretamente o resultado inicial, é o que muda a conclusão de "pode
construir sem ressalva" pra "pode construir, mas com o alcance real do problema bem explicado":

1. **Arquivos diferentes alterados em cada branch** (ex.: duas regiões diferentes do mundo
   construídas em paralelo) → merge automático limpo, sem conflito, mesmo sendo tudo binário —
   Git faz merge por arquivo inteiro, não por conteúdo, quando não há sobreposição.
2. **Mesmo arquivo, as duas branches chegam no mesmo resultado exato** (hash idêntico) → merge
   automático limpo, sem conflito — não há nada pra reconciliar.
3. **Mesmo arquivo binário, conteúdo diferente nas duas branches** → Git detecta que é binário
   (mesma heurística do comando `file`) e **se recusa a tentar um merge de conteúdo** — nunca
   injeta marcadores de conflito (`<<<<<<<`/`=======`/`>>>>>>>`) dentro do arquivo. Isso só
   acontece se o Git achar que é texto (testado de propósito com um arquivo texto puro primeiro,
   pra confirmar o risco real: aí sim os marcadores foram escritos literalmente dentro do
   arquivo — por isso identificar corretamente os arquivos de mundo como binários pro Git
   importa; não deveria ser um problema hoje, já que `.mca`/`level.dat` reais sempre têm bytes
   não-texto, mas vale registrar como um requisito, não uma garantia grátis).
4. **Arquivo deletado numa branch, modificado na outra** ("modify/delete", um tipo de conflito
   diferente do de conteúdo) → também detectado e reportado com clareza pelo Git
   (`deleted by them`), arquivo modificado preservado intacto no disco, nada corrompido.
5. Em ambos os casos de conflito (3 e 4), **as duas versões completas continuam recuperáveis**
   via `git ls-files -u` (3 estágios: base/ours/theirs, cada um endereçável por hash de blob), e
   **`git merge --abort` desfaz tudo de forma limpa e completa** — confirmado comparando o
   conteúdo do arquivo antes/depois do abort.
6. **(Correção, mesma sessão, após pergunta direta do usuário) Duas mudanças SEM NENHUMA
   sobreposição real, mas dentro do MESMO arquivo** — simulado como uma "casa" escrita nos
   primeiros 100 bytes de um arquivo de 4096 bytes numa branch, e um "bloco quebrado por mob"
   escrito nos bytes 4000-4001 (ponta completamente oposta) na outra → **ainda assim conflito no
   arquivo inteiro**, mesmo padrão do experimento 3. O Git não tem como saber que as duas
   mudanças não se sobrepõem de verdade — ele só vê "o blob final é diferente nos dois lados",
   ponto. Confirmado com `git merge-tree --write-tree` (preview, sem tocar em nada):
   `CONFLICT (content): Merge conflict in r.0.0.mca`.

**O que isso significa numa mesa de jogo de verdade**: um arquivo de região (`.mca`) cobre 32×32
chunks = 512×512 blocos. O cenário "casa construída na main, um mob quebra um bloco em outro
canto da mesma região na branch" — ou pior, "duas casas em lugares diferentes mas dentro da
mesma região" — força escolher **a região inteira de uma branch ou da outra**, descartando por
completo o que a branch perdedora tinha ali, mesmo que as duas mudanças não tivessem relação
nenhuma entre si. Isso não é um bug do mcgit nem do Git — é a granularidade inerente de qualquer
merge binário-por-arquivo. Resolver de verdade (reconciliar duas mudanças reais dentro da mesma
região, chunk a chunk) exige entender o formato Anvil/NBT por dentro — exatamente o que a **Fase
4 (Minecraft-Aware World Diffing)** existe pra fazer, e que a Fase 6 (Git Engine puro) não tem
como resolver sozinha.

**Descoberta extra útil pro design**: `git merge-tree --write-tree <branch-a> <branch-b>`
(Git ≥ 2.38 — a máquina de desenvolvimento tem 2.55.0) faz um **preview do merge sem tocar a
árvore de trabalho nem o índice** — devolve só o hash da árvore resultante (sucesso) ou a lista
de arquivos conflitantes com aviso (falha), sem nunca deixar o repositório num estado de merge
pendente. Isso significa que uma futura UI de merge pode mostrar "esses N arquivos vão
conflitar, quer continuar?" **antes** de qualquer coisa tocar os arquivos do jogador — só chama
`git merge` de verdade depois que o jogador confirma.

**Design de resolução decorrente (não implementado ainda, só desenhado)**: como o Git nunca tenta
reconciliar conteúdo binário sozinho, resolver um conflito na UI do mcgit não precisa entender
NBT/Anvil — só precisa deixar o jogador escolher, **por arquivo (= região) inteiro**, qual versão
manter. Dado o achado do experimento 6, a cópia da UI precisa deixar isso explícito — não pode
soar como "resolvendo um conflito pontual", já que na prática é "escolhendo qual versão de uma
região de 512×512 blocos manter, perdendo qualquer outra mudança que a branch descartada tinha
ali":
- Detectar arquivos conflitantes: `git diff --name-only --diff-filter=U` (ou `git ls-files -u`
  se precisar diferenciar modify/modify de modify/delete).
- Idealmente, mostrar isso via `git merge-tree --write-tree` **antes** de rodar `git merge` de
  verdade — "essas N regiões vão conflitar: escolher uma versão inteira de cada, descartando a
  outra" — pra decisão ser informada antes de qualquer coisa mudar de verdade.
- "Manter a versão desta branch": `git checkout --ours -- <path>` + `git add <path>` (ou
  `git rm <path>` se o "ours" for a deleção).
- "Manter a versão da outra branch": `git checkout --theirs -- <path>` + `git add <path>` (ou
  `git rm <path>` se o "theirs" for a deleção).
- Depois de resolver todos os arquivos conflitantes: `git commit` fecha o merge.
- Cancelar a qualquer momento: `git merge --abort` (verificado seguro nos experimentos acima).

**Não implementado nesta investigação em si** — o pedido inicial foi só investigar. A conclusão
final não é um "sim, simples" — é "sim, mas com um alcance de perda real que precisa ficar óbvio
pro jogador antes de ele confirmar um merge", já que a Fase 6 sozinha (Git puro) não tem como
resolver dentro de uma região; só a Fase 4 (Minecraft-Aware World Diffing) resolveria isso de
verdade. O design acima virou código de verdade ainda na mesma sessão — ver subseção "Merge
entre branches — implementado" logo abaixo.

---

### Merge entre branches — implementado (Sessão 8, continuação, 2026-09-01)

Terceiro e último item da Fase 6, implementado na sequência direta da investigação acima (mesma
sessão), com o usuário confirmando via `AskUserQuestion` que queria seguir mesmo com o alcance
de perda real explicado. Fecha o ciclo do Git Engine puro: ativar/desativar → snapshot →
histórico → restaurar → deletar → criar/trocar branch → comparar → **merge**.

- **`git.rs` ganha 7 funções/tipos novos**: `preview_merge()` (`git merge-tree --write-tree`,
  não toca working tree/índice), `list_merge_conflicts()` (parseia `git status --porcelain=v1`,
  6 códigos XY de conflito → `ConflictKind::{BothModified,DeletedByUs,DeletedByThem}`),
  `merge_branch()` (roda `git merge` de verdade, retorna `MergeOutcome::Merged(hash)` ou
  `ConflictsPending(Vec<ConflictedFile>)`), `resolve_conflict()` (`checkout --ours`/`--theirs`,
  com fallback pra `git rm` quando o lado escolhido é o que deletou — detectado pelo próprio
  texto de erro do Git, `"does not have (our|their) version"`), `finish_merge()` (`git commit`
  sem `add -A`, só o que já foi resolvido), `abort_merge()` (`git merge --abort`).
- **Duas guardas próprias**: mundo aberto (`is_currently_open`, mesma de sempre) e merge já em
  andamento (`is_merge_in_progress`, checa `.git/MERGE_HEAD`) — evita o erro genérico e confuso
  do Git ("Exiting because of an unresolved conflict") quando alguém tenta iniciar um segundo
  merge sem resolver o primeiro (achado real durante os experimentos da investigação).
- **Novo erro `MergeError`** (`WorldCurrentlyOpen`/`AlreadyInProgress`/`Git`/`Io`), mesmo formato
  de `RestoreError`/`BranchError`.
- **9 testes novos** (`mcgit-core`): preview limpo/conflitante, merge limpo, merge com conflito,
  merge recusado com merge já em andamento, merge recusado com mundo travado, resolver +
  finalizar (confirma commit de 2 pais via `git log --format=%P`), resolver um conflito de
  modify/delete mantendo o lado que deletou (confirma que o arquivo some), abortar restaura o
  estado exato de antes. 41/41 verdes no crate.
- **Ponte Tauri**: `preview_world_merge`, `merge_world_branch` (retorna `MergeOutcomeDto` com tag
  `Merged`/`ConflictsPending`), `resolve_world_merge_conflict`, `finish_world_merge`,
  `abort_world_merge`.
- **UI**: terceiro botão "Merge" por branch não-atual em `WorldBranches.tsx`, ao lado de
  Switch/Compare. Fluxo em 2 passos, sempre inline (nunca modal): (1) preview mostra a lista real
  de arquivos que vão conflitar, com o aviso de granularidade escrito por extenso — "N files
  would conflict: ... You'll pick one branch's full version of each — the losing side's changes
  to that whole file are discarded"; (2) se confirmado e houver conflito de verdade, entra num
  modo de resolução por arquivo ("Keep this branch's version" / "Keep the other branch's
  version"), com "Abort merge" sempre visível e "Finish merge" aparecendo só quando a lista de
  conflitos esvazia. Criar ou trocar de branch durante um merge pendente limpa esse estado local
  (a comparação ficaria sem sentido).

**Verificado ao vivo pela GUI real** (tela livre, mesma sessão da investigação): fluxo completo
com um conflito real em `level.dat` entre `main` e `experiment` — preview mostrou corretamente
"1 file would conflict: level.dat..."; merge real confirmado entrou em conflito
("1 file need to be resolved..."); resolvido escolhendo "Keep this branch's version"; "Finish
merge" fechou o merge com sucesso ("Merged \"experiment\"."). Conferido direto no disco depois:
conteúdo do arquivo bate com a versão escolhida, e `git log --format=%P` do commit de merge
mostra os dois pais reais (`a4c39e3` e `1bb8092`), confirmando uma topologia de merge genuína,
não uma simulação. Um segundo cenário de conflito foi criado e desta vez **abortado** em vez de
resolvido: `git status`/conteúdo do arquivo confirmados batendo exatamente com o estado de antes
do merge, `.git/MERGE_HEAD` ausente depois — abort realmente limpo, como os experimentos da
investigação já indicavam.

---

## Modo Básico/Avançado — implementado (Sessão 7, 2026-08-22, continuação)

`CONTEXT.md` já especificava esse toggle: "avançado expõe Git (commits/branches/remotes/diff)
— básico não". Escopo desta sessão foi o que já existe de verdade por baixo dos panos hoje —
branches, remotes e diff ainda não são features implementadas (ficam pra quando existirem,
Fase 6+); o único detalhe de Git já escondido é o **hash completo do commit** (a UI sempre
trunca pra 7 chars) e a **identidade fixa do autor** (`mcgit <mcgit@localhost>`, configurada por
`ensure_identity` antes de todo commit — ver §Git Engine), já antecipada numa nota da Sessão 4
("essa identidade vai aparecer literalmente... quando Modo Avançado existir").

- **Zero mudança em Rust/DB** — feature 100% frontend. O hash completo já vinha de `log()`
  (só estava sendo cortado no React); a identidade é sempre a mesma constante por design, não
  precisa ser buscada por commit.
- **`AdvancedModeContext.tsx`** (novo, `apps/desktop/src/context/`): Context + Provider +
  hook `useAdvancedMode()`, estado booleano persistido em `localStorage`
  (`mcgit.advancedMode`) — decisão deliberada: é uma preferência de UI do jogador, não dado de
  mundo/instância, então não precisa de tabela nova no banco (que exigiria migration). Usado
  via Context (não prop-drilling) porque o consumidor (`WorldHistory.tsx`) fica 3 níveis
  abaixo do provider na árvore de componentes (`App` → `InstanceDetailScreen` →
  `WorldList` → `WorldHistory`).
- **Toggle na navegação** (`App.tsx`): checkbox "Advanced mode" ao lado dos links existentes.
- **`WorldHistory.tsx`**: em Modo Avançado, cada linha do histórico mostra o hash completo (40
  chars) em vez do curto (7 chars), mais uma linha extra com a identidade do autor
  (`mcgit <mcgit@localhost>`); em Modo Básico, nada muda do comportamento anterior.

**Validado ao vivo pela GUI real**: hash completo e linha de autor aparecem só com o toggle
ligado, batendo exatamente com `git log --format="%H %an <%ae>"` rodado direto no mundo de
teste; toggle desligado volta ao hash curto sem autor; **persistência real confirmada**
fechando e reabrindo o app (processo do Tauri encerrado e relançado do zero) — o estado do
toggle sobreviveu via `localStorage`, não só em memória do React.

---

## Fase 4 — Minecraft-Aware World Diffing

### Diff por chunk — primeira fatia implementada (Sessão 8, quinta continuação, 2026-09-01)

Primeira fatia da Fase 4, escolhida com o usuário (`AskUserQuestion`) por atacar diretamente o
problema de granularidade achado na investigação de merge da Fase 6: a comparação entre branches
(`diff_branches`, Fase 6) só sabia dizer que um arquivo de região (`.mca`, 512×512 blocos) mudou
como um todo. Esta fatia adiciona uma camada: pra um arquivo de região marcado como "modified",
mostra **quais chunks (colunas de 16×16 blocos) especificamente mudaram** — sem decodificar
bloco-a-bloco ainda (block-states são bit-packed com paleta, decodificar isso de verdade é o
próximo passo, "Parser NBT completo").

- **Novo crate `crates/mcgit-world`** (pure lib, zero conhecimento de Git): `fastanvil` 0.32 lê
  regiões `.mca` por chunk — caminho já validado na prática pela Fase 0 (existe um binário de
  benchmark, `benchmarks/mca-bench`, testado contra um mundo real). `parse_region_coords()`
  extrai `(region_x, region_z)` do nome do arquivo (`"r.-1.0.mca"` → `(-1, 0)`, cobre
  coordenadas negativas). `diff_region_chunks(from_bytes, to_bytes, region_x, region_z)` compara
  os 1024 slots (32×32) entre duas versões dos bytes brutos do arquivo — dois `None` pula; um
  `None`/um `Some` vira `Added`/`Removed`; dois `Some` com bytes diferentes vira `Changed`;
  bytes iguais pula — reportando cada diferença em **coordenadas absolutas de chunk no mundo**
  (`region_x*32 + local_x`), não coordenadas locais à região.
- **`crates/mcgit-core` ganha `mcgit-world` como dependência**: `blob_contents()` (novo em
  `git.rs`, `git cat-file -p <ref>:<path>`, bytes crus sem conversão de texto — irmã de
  `blob_size`, que só precisa do tamanho) busca o conteúdo binário de um arquivo de região em
  duas branches; `diff_region_chunks(world_dir, from, to, path)` orquestra: extrai o nome do
  arquivo, resolve as coordenadas da região, busca os bytes dos dois lados, delega o diff pro
  `mcgit-world`.
- **Achado técnico de verificação, antes de escrever qualquer teste**: construir uma região
  válida do zero em memória (pra fixture de teste, sem depender de um arquivo `.mca` real de
  3.6MB) parecia arriscado — `Region::from_stream` sobre um buffer zerado por fora não é a API
  pensada pra isso. Conferido no código-fonte do `fastanvil` (não só nos docs): existe
  `Region::create(stream)`, construtor dedicado exatamente pra esse caso (escreve o cabeçalho de
  8KB ele mesmo, inicializa o rastreamento de setores livres corretamente — `from_stream` sobre
  um buffer manualmente zerado deixa esse rastreamento inconsistente, `vec![0]` em vez do
  `vec![2]` que `create` usa). Usando `Region::create`, os testes com fixtures sintéticas
  funcionaram de primeira, sem precisar do mundo real como fallback.
- **Testes**: `mcgit-world` (6) — `parse_region_coords` com coordenadas negativas e nomes
  inválidos, `diff_region_chunks` reportando chunk adicionado/removido/alterado e ignorando os
  inalterados, branches idênticas retornando lista vazia. `mcgit-core` (+1, teste de integração
  de ponta a ponta) — mundo Git de teste com região sintética commitada, editada num chunk só
  numa branch, `diff_region_chunks` confirmando que só aquele chunk aparece.
- **Ponte Tauri**: `diff_world_region_chunks` (compara a branch atual contra uma branch
  informada, pra um `path` de arquivo de região específico).
- **UI**: dentro da lista de arquivos já mostrada pelo "Compare" (Fase 6), cada linha de arquivo
  de região modificado (`change.path` começa com `"region/"` e termina em `".mca"`) ganha um
  botão extra "Show chunks", expandindo inline (sem modal) a lista de chunks alterados —
  `"(12, -5) changed"` etc., coordenadas absolutas, não locais à região.
- **Fora de escopo desta fatia**: decodificar block-states/entidades/estruturas de verdade (só
  "mudou ou não", sem dizer o quê); dimensões Nether/End (`DIM-1/region/`, `DIM1/region/`) e
  `entities/`/`poi/` (só a pasta `region/` principal); estatísticas por snapshot e visualização
  gráfica de verdade (os outros 3 itens do checklist da Fase 4).

**Verificado ao vivo pela GUI real** (tela livre): gerado um arquivo de região sintético com dois
chunks (via um exemplo Rust temporário reaproveitando a própria API pública do `mcgit-world`,
removido depois do teste), commitado na branch `main`; criada a branch `experiment`, um dos dois
chunks alterado e commitado lá; de volta em `main`, "Compare" mostrou
`modified — region/r.0.0.mca — 16.0 KB → 16.0 KB`; "Show chunks" mostrou corretamente
`(0, 0) changed` como único resultado — o chunk que não mudou não apareceu na lista, confirmando
o comportamento correto ponta a ponta (Git real + Anvil real + UI real).

### Parser NBT completo — diff bloco a bloco (Sessão 9, 2026-09-02)

Segunda fatia da Fase 4, escolhida com o usuário logo após o diff por chunk: decodificar de
verdade o `block_states` bit-packed de cada seção pra dizer **qual bloco específico mudou** (não
só "esse chunk mudou"). Investigação primeiro (modo ensino): inspecionado ao vivo um chunk real
do mundo de teste (`benchmarks/worlds/medieval`, 1.21.x) via uma extensão nova do `mca-bench`
(`inspect` passou a listar seções e suas palettes) — confirmou o formato paletted container
(`palette` + `data`) e dois casos reais que guiaram o design:

- **Palette de 1 tipo só não tem `data`** — quando a seção inteira é um único bloco (ex.: ar),
  o Minecraft nem guarda índices. Confirmado ao vivo: seções Y=6..19 do chunk real tinham
  `palette` com 1 entrada e `data` com 0 longs.
- **`bits_per_block = max(4, ceil(log2(palette.len())))`, `entries_per_long = 64 / bits_per_block`,
  sem entrada cruzar fronteira de `long`** — confirmado numericamente contra o mesmo chunk real:
  seção Y=4 com palette de 18 tipos precisa de 5 bits/bloco → 12 blocos por long →
  `ceil(4096/12) = 342` longs, batendo exatamente com os 342 longs observados no `data` real.
- **Identidade de bloco não é só `Name`** — a mesma palette tinha `minecraft:oak_leaves`
  repetido várias vezes (variantes diferentes de `Properties`, ex. `persistent`/`distance`).
  Decisão: a chave de diff é `Name` + `Properties` serializadas em ordem estável
  (`"minecraft:furnace[facing=north,lit=true]"`), não só o nome.

- **`crates/mcgit-world/src/chunk.rs`** (novo módulo): `decode_section_blocks` decodifica uma
  seção pros 4096 blocos que ela contém (ordem `index = local_y*256 + local_z*16 + local_x`,
  igual o próprio Minecraft usa), tratando o caso de palette de 1 tipo à parte.
  `decode_chunk_sections` parseia o NBT bruto de um chunk e decodifica todas as seções, indexadas
  pela Y absoluta da seção; uma seção sem `block_states` (fora do range gerado) decodifica como ar
  puro. `diff_chunk_blocks(from_nbt, to_nbt, chunk_x, chunk_z)` compara as seções presentes **nos
  dois lados** posição a posição, devolvendo `BlockDiff { x, y, z, from, to }` em coordenadas
  absolutas do mundo — **seções que só existem de um lado (mudança na altura gerada do chunk) são
  ignoradas nesta fatia**, caso raro deixado pra depois. `WorldError` ganha `Nbt` (erros de parse
  do `fastnbt`) e `Shape` (formato inesperado dentro do NBT).
- **`crates/mcgit-core`**: `fastanvil` sobe de dev-dependency pra dependency de verdade (agora é
  usado em código de produção, não só teste). `diff_chunk_blocks(world_dir, from, to, path,
  chunk_x, chunk_z)` resolve `region_x`/`region_z` do nome do arquivo (mesma lógica de
  `diff_region_chunks`), converte as coordenadas absolutas do chunk pra locais `0..32` dentro da
  região, e usa a nova função auxiliar `read_chunk_nbt` (busca a região via `blob_contents`, abre
  com `fastanvil::Region::from_stream`, extrai o NBT bruto de um chunk local) pros dois lados
  antes de delegar pro `mcgit_world::diff_chunk_blocks`.
- **Testes**: `mcgit-world` (+7, chunk.rs) — palette de 1 tipo sem `data` decodifica uniforme;
  palette de 18 tipos/5 bits (espelhando a seção real Y=4) decodifica certo, inclusive nas duas
  pontas do array; `Name`+`Properties` iguais/diferentes geram identidades diferentes; diff reporta
  só o bloco que mudou; seções só de um lado são ignoradas; um chunk lido de volta de uma região
  `.mca` real (via `Region::create`/`write_chunk`/`read_chunk`) decodifica certo, ponta a ponta.
  `mcgit-core` (+1) — histórico Git real com região sintética (palette de 2 tipos, 4096 blocos
  virados de stone pra air numa branch), `diff_chunk_blocks` reportando os 4096 blocos certos, nas
  coordenadas absolutas certas.
- **Ponte Tauri**: `diff_world_chunk_blocks` (mesmos parâmetros de `diff_world_region_chunks`, mais
  `chunk_x`/`chunk_z` — os mesmos que `diff_world_region_chunks` já reporta pra um chunk
  "changed").
- **UI**: dentro do "Show chunks" (Fase 4, primeira fatia), cada chunk com status `changed` ganha
  um botão "Show blocks" — expande inline a lista de blocos que mudaram (`"(x, y, z): de → para"`),
  limitada a 50 itens com um "...e N mais" se passar disso.
- **Fora de escopo desta fatia**: seções presentes só de um lado (chunk cuja altura gerada mudou);
  crescer a palette (só compara o que já está decodificado — não é um problema de escrita, só de
  leitura, então não se aplica aqui); estatísticas por snapshot e visualização gráfica de verdade
  (ainda os 2 itens restantes do checklist da Fase 4).

**Verificado ao vivo pela GUI real** (app rodando via `npx tauri dev` na tela do usuário — sem
ferramenta de controle de tela nesta sessão, o usuário clicou e conferiu pessoalmente): mundo real
de teste (`medieval`) copiado pra dentro de uma instância já existente, versionado, branch
`experiment` criada a partir de `main`. Editado um bloco só — chunk (0,0), seção Y=0, posição
absoluta (0,0,0) — de `minecraft:deepslate` pra `minecraft:stone`, via um comando novo no
`mca-bench` (`set-block`, sobrescreve um índice específico do `data` bit-packed reaproveitando uma
entrada já existente na palette, sem precisar recodificá-la). "Compare" → "Show chunks" → "Show
blocks" mostrou exatamente a linha esperada, uma única mudança. Usuário confirmou ("deu boa").

### Estatísticas de mundo por snapshot — blocos (Sessão 9, 2026-09-02)

Terceira fatia da Fase 4, escolhida com o usuário logo após o parser NBT completo: em vez de
comparar duas branches, contar os blocos de **um** snapshot só — "quantos de cada tipo de bloco
existem nesse mundo". Reaproveita o mesmo decoder de `block_states` construído na fatia anterior,
mas como agregação em vez de diff.

- **Decisão de design, explícita pro usuário antes de codar**: a contagem agrupa só por `Name`,
  ignorando `Properties` — diferente da identidade usada no diff bloco a bloco. Pro diff, um forno
  aceso e apagado são blocos diferentes (perder essa mudança seria um bug); pra "quantos fornos
  existem", separar por aceso/apagado só atrapalharia — o jogador quer saber "quantos `stone`",
  não quantas variantes de propriedade.
- **`crates/mcgit-world/src/chunk.rs`**: `count_section_blocks` talha uma seção por índice de
  palette em vez de identidade por posição — nunca materializa os 4096 nomes por seção (como
  `decode_section_blocks` faz pro diff), só incrementa um contador por slot de palette conforme
  decodifica cada índice bit-packed, e só no fim mapeia slot→nome pra somar no total. Mais barato
  que reaproveitar o decoder de diff, já que a palette de uma seção costuma ter só dezenas de
  entradas apesar dos 4096 blocos. `count_chunk_blocks` (`pub(crate)`) soma isso por todas as
  seções de um chunk; uma seção sem `block_states` conta como ar puro, igual o decode de diff.
- **`crates/mcgit-world/src/region.rs`**: `count_region_blocks` abre uma região e soma
  `count_chunk_blocks` de todo chunk gerado (dos 1024 slots possíveis) — o análogo, pra um lado
  só, do `diff_region_chunks` que já existia.
- **`crates/mcgit-core`**: `world_block_stats(world_dir, git_ref)` lista todo arquivo dentro de
  `region/` num snapshot via `git ls-tree -r --name-only <ref> -- region/` (função nova,
  `list_files`, generalizável pra outros prefixos no futuro), busca o conteúdo de cada um
  (`blob_contents`, já existente), soma as contagens de `mcgit_world::count_region_blocks` de
  todos os arquivos, e devolve ordenado do mais comum pro menos comum (empate quebrado por nome,
  pra ordem estável).
- **Testes**: `mcgit-world` (+7) — seção de palette única, palette multi-entrada (espelhando a
  seção real de 18 tipos/5 bits da fatia anterior), `Properties` diferentes colapsando no mesmo
  nome, soma entre seções de um chunk, soma entre chunks de uma região, região vazia devolvendo
  contagem vazia. `mcgit-core` (+1) — histórico Git real com dois arquivos de região diferentes
  (`r.0.0.mca` todo pedra, `r.1.0.mca` todo terra), `world_block_stats` somando os dois e
  ordenando corretamente.
- **Ponte Tauri**: `world_block_stats(instance_id, folder_name, commit_hash)` — recebe o hash do
  snapshot diretamente (não branch atual + branch alvo, como o diff), já que estatística é sobre
  **um** snapshot, não uma comparação.
- **UI**: botão "Show stats" por snapshot em `WorldHistory` (tela de histórico, ao lado de
  Restore/Delete) — diferente do "Show chunks"/"Show blocks" da Fase 6/4 (que só existem no Modo
  Avançado, dentro de branches), esta fica disponível também no Modo Básico, já que ver
  estatísticas do mundo atual não exige entender branches. Lista limitada a 20 tipos de bloco
  (já vem ordenada do mais comum, então o corte cobre a parte útil), com contagem formatada
  (`1.234 × minecraft:stone`).
- **Fora de escopo desta fatia**: entidades e estruturas (só blocos) — fechado na continuação
  logo abaixo; dimensões Nether/End (`DIM-1`/`DIM1`) seguem fora de escopo.

### Estatísticas de mundo por snapshot — entidades e estruturas (Sessão 9, continuação, 2026-09-02)

Fecha o item de estatísticas da Fase 4, contando os dois tipos que a fatia anterior deixou de
fora. Antes de codar, investigado ao vivo contra um chunk real do `medieval` (extensão temporária
em `mca-bench inspect`, removida depois de confirmar o formato) — dois achados moldaram o design:

- **Entidades vivem em pasta separada.** Desde a 1.17, mobs/itens/projéteis não ficam mais dentro
  do chunk NBT de `region/` — moraram pra uma pasta própria, `entities/`, com arquivos `.mca` no
  mesmo formato Anvil, mas com raiz de chunk diferente: uma lista `Entities`, sem `sections`. Cada
  entidade tem um campo `id` (ex. `minecraft:sheep`) — o resto (posição, vida, ...) é ruído pra uma
  contagem por tipo, mesmo princípio do `block_name` que já ignora `Properties`.
- **Estruturas geradas ficam dentro do chunk de `region/`, na chave `structures.starts`.** O
  importante: cada estrutura aparece como "start" só no chunk onde começou a gerar — os outros
  chunks que ela atravessa só têm uma referência de volta (`References`), não um segundo start.
  Confirmado contra um trial chamber real que ocupa vários chunks. Consequência: somar as chaves
  de `starts` (por tipo, ex. `minecraft:trial_chambers`) em todos os arquivos de região do mundo já
  dá a contagem certa de instâncias, sem duplicar.

- **`crates/mcgit-world/src/chunk.rs`**: `count_chunk_structures` (chunk de `region/`) lê
  `structures.starts` e incrementa por chave; ausência de `structures`/`starts` conta como "sem
  estruturas aqui", não erro — ao contrário de `count_chunk_blocks`, que exige `sections` (a
  ausência ali indicaria um formato de chunk que o mcgit não entende). `count_chunk_entities`
  (chunk de `entities/`) lê `Entities` e incrementa por `id`.
- **`crates/mcgit-world/src/region.rs`**: `count_region_structures`/`count_region_entities` — mesmo
  laço de 32×32 chunks de `count_region_blocks`, delegando a cada uma das funções acima.
  **Guarda nova, achada na verificação ao vivo (ver abaixo)**: um arquivo de região de **0 bytes**
  é tratado como "sem entradas" antes mesmo de tentar abrir via `Region::from_stream` — as três
  funções de contagem (`count_region_blocks` incluída, retroativamente) ganharam essa guarda.
- **`crates/mcgit-core`**: `world_structure_stats`/`world_entity_stats`, listando `region/` e
  `entities/` respectivamente via `list_files` (já existente, genérica desde a fatia anterior).
  A soma/ordenação (mais comum primeiro, empate por nome) era código idêntico repetido em três
  funções — extraído pra `aggregate_region_stats`, uma função privada que recebe qual contador de
  região usar (`count_one_region: impl Fn(&[u8]) -> Result<HashMap<String, u64>, GitError>`) e
  cuida do resto (buscar blobs, somar, ordenar). `world_block_stats` também passou a usar essa
  função compartilhada.
- **Bug real achado na verificação ao vivo GUI, não no plano**: `world_entity_stats` quebrou contra
  o mundo `medieval` de verdade com `git command failed: ... UnexpectedEof, "failed to fill whole
  buffer"`. Investigado direto (não só lendo o erro): `entities/r.-2.-1.mca` é um arquivo de
  **0 bytes de verdade** — o próprio Minecraft escreve isso como placeholder pra uma região sem
  nada gerado ali ainda (confirmado também em `poi/` do mesmo mundo — mesmo padrão, pasta
  diferente). `fastanvil::Region::from_stream` espera um cabeçalho Anvil de 8KB completo e não
  tolera um arquivo vazio. Corrigido tratando `region_bytes.is_empty()` como "zero entradas" antes
  de chamar `Region::from_stream`, nas três funções de `region.rs` (blocos incluído, pelo mesmo
  risco, mesmo não tendo sido exercitado ali por sorte de dados). `diff_region_chunks` (Fase 6/4)
  tem o mesmo risco estrutural — não corrigido aqui (fora do pedido desta sessão), registrado como
  débito técnico.
- **Testes**: `mcgit-world` (+3 exercitando o caso novo + 2 de arquivo de 0 bytes retroativos pras
  três funções de contagem), `mcgit-core` (+2, histórico Git real com múltiplos arquivos de
  `region/`/`entities/`, confirmando soma entre arquivos e ordenação).
- **Ponte Tauri**: `world_structure_stats`/`world_entity_stats(instance_id, folder_name,
  commit_hash)`, mesma assinatura de `world_block_stats`.
- **UI**: o painel "Show stats" (já existente) ganhou duas seções novas, Structures e Entities,
  buscadas em paralelo (`Promise.all`) junto com Blocks no mesmo clique — não são três botões
  separados, é o mesmo "Show stats" mostrando mais coisa. Renderização das três seções unificada
  num componente `StatsSection` (mesma forma `{name, count}` nos três casos).
- **Verificado ao vivo pela GUI real** (`GDK_BACKEND=x11`/`xdotool`/`spectacle`, tela livre): antes
  do fix, erro genuíno na tela ao clicar "Show stats" contra o mundo `medieval`; depois do fix,
  as três seções populadas com dados reais — Structures (18 mineshafts, 5 trial chambers, ocean
  ruins, ruined portals, shipwreck, monument, village) e Entities (110 sheep, 99 chickens, 60
  pigs, 40 cows, mobs, item, bat, ...).

**Verificado ao vivo pela GUI real** (mesmo app já rodando via `npx tauri dev`, hot-reload pegou
as mudanças de back e frontend sem precisar reiniciar): usuário abriu "Show history" → "Show
stats" no snapshot mais recente do mundo `TestWorld` (o mundo `medieval` real, com vários tipos de
bloco de verdade — pedra, minérios, madeira etc.) e confirmou que apareceu uma lista de blocos de
verdade, ordenada do mais comum pro menos comum ("deu boa").

### Mapa visual de chunks — visualização de alterações (Sessão 9, terceira continuação, 2026-09-02)

Fecha o último item em aberto da Fase 4, pedido direto pelo usuário ("segue para o visual") logo
após a fatia de estatísticas. Escopo confirmado via `AskUserQuestion` antes de codar: só o mapa de
chunks por arquivo de região (o "mapa, destaque visual" que `PHASE.md` já previa) — visualizar os
blocos alterados dentro de um chunk continua como lista de texto (já funciona desde o parser NBT
completo), um salto de complexidade maior (3D, múltiplas camadas Y) fora do pedido desta fatia. E
o clique num chunk `changed` da grade deveria expandir a lista de blocos abaixo — mesmo
comportamento de antes, só trocando o gatilho de um botão de texto pra célula da grade.

**Pura mudança de frontend — nenhuma linha de Rust tocada.** Os dados já existiam desde "diff por
chunk" (primeira fatia da Fase 4): `diff_world_region_chunks` já devolve `{chunk_x, chunk_z,
status}` pra cada chunk que difere entre duas branches; faltava só desenhar isso como grade em vez
de lista.

- **`apps/desktop/src/features/world/RegionChunkMap.tsx`** (novo componente): recebe a lista de
  `ChunkDiff` de um arquivo de região junto com `regionX`/`regionZ` (extraídos do nome do arquivo,
  ex. `"region/r.-1.0.mca"` → `[-1, 0]`, via uma função `parseRegionCoords` no lado TypeScript que
  espelha `mcgit_world::parse_region_coords` do lado Rust — mesma regra `r.<x>.<z>.mca`, só que
  nunca precisou existir em JS até agora porque nada no frontend converia coordenada absoluta pra
  posição dentro da região). Converte cada `chunk_x`/`chunk_z` absoluto em coordenada local
  `0..32` (`chunk_x - region_x*32`), monta um mapa de busca por coordenada local, e desenha uma
  grade CSS de 32×32 células de 11px — a maioria das células fica em branco/cinza-claro (não dá
  pra distinguir "não mudou" de "nunca foi gerado" só com os dados que `diff_region_chunks`
  devolve, e não tem necessidade prática de distinguir isso aqui). Cor por status
  (`added`=verde, `removed`=vermelho, `changed`=amarelo), `title` nativo do navegador pra mostrar
  coordenada+status no hover (sem precisar de tooltip customizado), clique só ativo em células
  `changed` (`removed`/`added` não têm diff de blocos comparável — um lado nem tem o chunk).
- **`apps/desktop/src/features/world/WorldBranches.tsx`**: o `.map((change) => (...))` de
  arrow-function-com-retorno-implícito virou função com corpo (`{ ... return (...) }`) pra poder
  computar `regionCoords`/`thisRegionChunkDiff`/`thisChunkBlockDiff` como variáveis antes do JSX —
  a lista `<ul>` de `(x, z) status` foi substituída por `<RegionChunkMap>`, e o painel de blocos
  (que antes vivia dentro do `<li>` de cada chunk da lista) virou um bloco único abaixo da grade,
  mostrado quando `openBlocksFor` aponta pra algum chunk — mesmo estado (`openBlocksFor`, já
  existente) reaproveitado, só a UI ao redor mudou. `describeChunkDiff` (só formatava o texto da
  lista antiga) foi removida por não ter mais uso.
- **Verificado ao vivo pela GUI real** (`GDK_BACKEND=x11`/`xdotool`/`spectacle`, tela livre):
  comparação `main` ↔ `experiment` do mundo `TestWorld` (o mesmo mundo `medieval` de teste, com o
  bloco editado em sessões anteriores) — grade de 32×32 renderizada corretamente com um único
  chunk `(0, 0)` amarelo no canto, legenda (Added/Removed/Changed) visível abaixo, clique na
  célula expandindo corretamente `Blocks changed in chunk (0, 0): (0, 0, 0):
  minecraft:deepslate[axis=y] → minecraft:stone` — o mesmo dado que a lista de texto já mostrava
  antes, só que agora alcançado clicando na célula em vez de um botão. `npx tsc --noEmit` limpo
  (não há testes automatizados de componente React no projeto ainda — verificação é só via
  typecheck + GUI real, mesmo padrão de todas as fatias de UI anteriores).

### Diff por chunk — fechando a lacuna de entidades/estruturas/Nether-End (Sessão 10, 2026-09-02)

A primeira fatia do diff por chunk (Sessão 8) só cobria blocos, e só dentro de `region/` — o
Nether (`DIM-1/region/`), o End (`DIM1/region/`) e as entidades (`entities/`) nunca apareciam no
"Show chunks" nem tinham nenhuma visão por chunk. Esta fatia fecha as três lacunas de uma vez,
mantendo o design de todo o resto da Fase 4: nada tenta resolver conflito ou reconstruir estado,
só relatar o que mudou.

**Nether/End — conserto só de frontend.** `diff_region_chunks`/`diff_chunk_blocks` do lado Rust
já eram agnósticos de pasta — só olham o nome do arquivo (`r.<x>.<z>.mca`) via
`parse_region_coords`, nunca o caminho completo. O único bloqueio era o filtro de UI
(`isRegionFile` em `WorldBranches.tsx`) que só aceitava caminhos começando com `region/`.
Renomeado pra `regionFileKind(path)`, que casa `(?:DIM-1\/|DIM1\/)?(region|entities)\/.*\.mca$` e
devolve qual dos dois formatos de chunk o arquivo tem (`"blocks"` ou `"entities"`) — cobrindo as
três pastas de dimensão de uma view só, sem duplicar a lógica de detecção.

**Diff de entidades por chunk — identidade por `UUID`, não por posição.** Investigado ao vivo
(extensão temporária em `mca-bench inspect`, aplicada a um chunk real de `entities/` do mundo
`medieval`) que toda entidade carrega um campo `UUID` — `IntArray` de 4 inteiros, ex.
`[-1428584928, -1209581274, -1358458702, -773260624]` — estável entre versões do mesmo mundo.
Isso muda a forma do diff em relação a bloco: bloco não tem identidade própria, só posição (por
isso `diff_chunk_blocks` compara posição a posição); entidade tem identidade e pode estar em
qualquer posição, então o diff certo é por conjunto de UUIDs — quem só existe do lado `from`
(`removed`), quem só existe do lado `to` (`added`). Uma entidade presente nos dois lados não
aparece no diff nem que tenha se movido ou mudado de vida/stats — posição/estado não fazem parte
da identidade de diffing (mesmo princípio de `count_chunk_entities`, que ignora tudo exceto
`id`, só que aqui também precisa do `UUID`).

- `entity_identity`/`decode_chunk_entities`/`diff_chunk_entities` em `mcgit-world::chunk` — o
  identificador de diffing é `(id, UUID stringificado)`; `UUID` ausente faz a entidade ser
  ignorada (mesma leniência de `count_chunk_entities`) em vez de erro, já que uma entidade
  malformada isolada não deveria travar o diff do chunk inteiro. Tipos novos em `types.rs`:
  `Presence` (`Added`/`Removed`, compartilhado com o diff de estruturas abaixo — diferente de
  `ChunkStatus`, que tem um terceiro estado `Changed` que não faz sentido aqui, já que uma
  entidade ou existe ou não existe de cada lado) e `EntityDiff { id, uuid, presence }`.
- `mcgit_core::git::diff_chunk_entities` reaproveita `read_chunk_nbt` sem mudança nenhuma — a
  função já era agnóstica de formato de chunk (só extrai bytes brutos via `fastanvil::Region`),
  então funciona igual pra um caminho de `entities/` como já funcionava pra `region/`.
- Comando Tauri `diff_world_chunk_entities`, DTO `EntityDiffDto`.
- UI: `WorldBranches.tsx` decide, por `regionFileKind`, o que buscar ao clicar numa célula
  `changed` da grade — `region/` busca blocos **e** estruturas (a seguir) em paralelo,
  `entities/` busca só entidades. Seção nova "Entities changed in chunk (x, z)" com uma linha por
  entidade (`added`/`removed` — `minecraft:sheep`). `RegionChunkMap.tsx` ganha um prop
  `detailLabel` (`"blocks and structures"` ou `"entities"`) só pra trocar o texto da legenda
  ("Click a changed chunk to see its ___"), sem nenhuma outra mudança — a grade em si já era
  agnóstica de conteúdo, só desenha status por célula.

**Diff de estruturas por chunk — conjunto de chaves de `structures.starts`.** Mesmo princípio já
confirmado nas estatísticas da Fase 4 (cada estrutura gerada aparece como "start" só no chunk
onde começou), agora comparado lado a lado em vez de somado: `decode_chunk_structure_starts`
extrai o conjunto de chaves de `structures.starts` de cada lado, `diff_chunk_structures` devolve
a diferença simétrica como `added`/`removed` por tipo de estrutura (`StructureDiff { id,
presence }`). Confirmado ao vivo (mesma extensão temporária do `mca-bench`) que uma entrada de
`starts` é diretamente o id da estrutura como chave (`"minecraft:mineshaft"`), sem nenhum
wrapper `"id"`/`"INVALID"` por baixo pra filtrar — o mesmo shape que `count_chunk_structures` já
assumia, agora verificado contra um mineshaft real gerado no mundo `medieval` (não só chunks com
`starts` vazio, que foi tudo que as fatias anteriores tinham inspecionado ao vivo). Comando Tauri
`diff_world_chunk_structures`, DTO `StructureDiffDto`, seção "Structures changed in chunk (x, z)"
sempre ao lado de "Blocks changed" pra arquivos de `region/` (mesmo quando vazia — mostra "No
structures differ." explicitamente, mesmo padrão de "No blocks differ..." já usado).

**Testes**: 8 novos em `mcgit-world::chunk` (entidades: added/removed por UUID, ignora entidade
inalterada nos dois lados, chunk vazio; estruturas: added/removed por chave, ignora uma presente
nos dois lados, chunk sem a chave `structures` nenhuma) e 10 novos em `mcgit-core::git` (as duas
novas funções contra um histórico Git real, construído com `fastanvil::Region::create` do mesmo
jeito que os testes de `diff_chunk_blocks`/`world_*_stats` já faziam) — 89 testes no workspace,
todos verdes; `npx tsc --noEmit` limpo.

**Verificado ao vivo pela GUI real** (`GDK_BACKEND=x11`/`xdotool`/`spectacle`, `npx tauri dev` na
tela livre): três edições controladas no mundo `TestWorld`, cada uma isolando uma das três
lacunas. (1) Nether: como o mundo de teste real não tinha Nether gerado, um `DIM-1/region/
r.0.0.mca` sintético foi criado (extensão temporária no `mca-bench`, `create-region-with-block`)
com um chunk de palette de 2 tipos (`netherrack`/`soul_sand`), commitado, depois o bloco (0,0,0)
trocado numa branch de teste — grade mostrando o chunk `(0, 0)` do Nether corretamente, "Blocks
changed in chunk (0, 0): minecraft:netherrack → minecraft:soul_sand" e "Structures changed in
chunk (0, 0): No structures differ." lado a lado. (2) Entidades: a entidade real já presente em
`entities/r.-1.0.mca` (um `minecraft:chest_minecart` com `UUID` real, achado numa investigação
anterior desta mesma sessão) removida na branch de teste (`mca-bench remove-entity`) — "Entities
changed in chunk (-28, 0): removed — minecraft:chest_minecart", legenda da grade corretamente
dizendo "its entities" em vez de "its blocks and structures". (3) Estruturas: um
`structures.starts` novo adicionado a um chunk de `region/r.0.0.mca` que não tinha nenhum
(`mca-bench add-structure-start`) — "Blocks changed in chunk (1, 0): No blocks differ..." e
"Structures changed in chunk (1, 0): added — minecraft:village_plains", confirmando que as duas
seções convivem corretamente mesmo quando só uma delas tem conteúdo. As três extensões
temporárias do `mca-bench` (`create-region-with-block`, `remove-entity`, `add-structure-start`)
foram revertidas depois do uso, e o `TestWorld` (branch de teste + região sintética do Nether)
foi limpo de volta ao estado anterior à sessão.

---

## Identidade Visual & Design System (Sessão 10, continuação, 2026-09-02)

Primeira passada de UX de verdade no app — até aqui a GUI (Tauri + React) era o template padrão
do Vite, sem tema nem componentes visuais próprios (`App.css` nunca tocado desde o scaffold da
Sessão 2). Pedida pelo usuário logo depois de fechar a Fase 4 ("passada de UX/identidade
visual"), registrada como pendência de sequenciamento desde a Sessão 9 (ver
`project_mcgit_ux_polish` na memória).

**Escopo confirmado com o usuário via `AskUserQuestion` antes de codar** (duas perguntas): (1) o
tom exato do vermelho — três direções mostradas (crimson sóbrio, vivo/redstone, terroso/tijolo)
com preview de hex + descrição de sensação; escolhido **vivo/redstone**; (2) fatiamento — em vez
do app inteiro numa sessão só, **fundação (design system) + telas principais** (Instances,
Instance detail — só o chrome da página, não `WorldList`/`WorldHistory`/`WorldBranches`/stats/
chunk map, que ficam pra uma fatia seguinte —, Java), deixando o resto herdar a base
automaticamente.

### Paleta e tokens

`apps/desktop/src/App.css` reescrito do zero como um design system pequeno baseado em CSS custom
properties, claro por padrão com override completo em `@media (prefers-color-scheme: dark)` —
mesmo mecanismo que o template já usava, sem toggle explícito de tema (não pedido).

```
Claro:  bg #faf7f6  surface #ffffff  border #e4dad8  text #201a1a  text-muted #6b5f5d
        primary #e11d2e → hover #c81726 → active #b01220  (contraste branco em cima)
Escuro: bg #1b1414  surface #241a1a  border #3d2c2b  text #f3e9e8  text-muted #c2afac
        primary #ff4c57 → hover #ff6871 → active #e11d2e  (mais claro, precisa de mais luz
        contra um fundo escuro pra manter a mesma legibilidade que o tom do modo claro)
```

`--color-danger`/`--color-success`/`--color-warning` mantidos como já estavam implicitamente
usados (ex.: `RegionChunkMap.tsx` já tinha `#c62828`/`#2e7d32`/`#f9a825` hard-coded pro mapa de
chunks da Fase 4) — não redefinidos como tokens novos nesta fatia porque essa tela específica não
foi tocada (fica pra fatia seguinte), mas os valores foram escolhidos deliberadamente distintos o
bastante do vermelho de marca pra não colidir visualmente quando essa tela ganhar os tokens de
verdade depois.

### Componentes base (afetam o app inteiro de graça)

Sem nenhuma mudança de markup, todo `<button>`/`<input>`/`<select>`/`<a>`/`<h1-3>`/`<code>`
existente no app passa a herdar tipografia, espaçamento, cor de foco (anel vermelho via
`box-shadow` + `color-mix()`) e um hover/active discreto (borda fica vermelha, sem preencher —
reserva o preenchimento sólido pra ação primária). Isso é o que faz `WorldList`/`WorldHistory`/
`WorldBranches` (não tocados nesta fatia) já saírem visualmente mais limpos na verificação ao
vivo, mesmo sem receber nenhuma classe nova.

Duas variantes semânticas de botão, aplicadas manualmente onde fazem sentido (não é global,
precisa da classe):

- **`.btn-primary`** — preenchimento vermelho sólido, texto branco. A ação que a tela mais quer
  que o jogador tome (Create instance, Install, ...). Estado `:disabled` sempre cai pro cinza
  neutro, nunca fica "vermelho desabilitado" (evita parecer um erro/aviso).
- **`.btn-danger`** — contorno vermelho de perigo (`--color-danger`, tom diferente do
  `--color-primary` de marca), preenche sólido só no hover. Não usado ainda nesta fatia (as
  ações destrutivas do app — Disable versioning, Delete snapshot, Abort merge — vivem todas
  dentro de `WorldList`, fora do escopo desta sessão), mas já definido no design system pra
  quando a fatia seguinte chegar lá.

### Telas aplicadas

- **`App.tsx`**: nav vira uma barra de verdade — wordmark "mcgit" em vermelho à esquerda (link
  pra home), links de navegação + toggle de Modo Avançado agrupados à direita, separados por uma
  borda inferior sutil.
- **`InstanceManagerScreen.tsx`/`InstanceList.tsx`/`CreateInstanceForm.tsx`**: a lista de
  instâncias vira uma grade de cards (`.card-grid`/`.card`, `grid-template-columns:
  repeat(auto-fill, minmax(220px, 1fr))` — mais perto do mockup original do `CONTEXT.md` do que
  a lista `<ul>` plana de antes) em vez de estilo inline por item; formulário de criação virou
  sua própria seção "Create instance" abaixo da lista (com `<hr class="section-divider">`), não
  mais espremido logo depois — decisão de layout, não só de cor. Botão "Create instance" ganha
  `.btn-primary`.
- **`InstanceDetailScreen.tsx`**: só o chrome da página (link "← Instances", cabeçalho, banners
  de erro/status via `.banner`/`.banner-error`/`.banner-status` em vez de `style={{color:
  "crimson"}}` inline) — o que `WorldList` renderiza dentro continua exatamente como estava.
- **`JavaManagerScreen.tsx`/`JavaInstallationList.tsx`/`JavaVersionPicker.tsx`/
  `AddManualJavaForm.tsx`**: lista de instalações vira `.install-list` (uma linha por
  instalação, ação à direita — layout de lista, não de card, já que uma instalação de Java tem
  menos "identidade visual" que uma instância pra merecer um card próprio); "default" em verde
  (`--color-success`) em vez de texto puro; página reorganizada em três seções com `<h2>`
  próprios ("Install a Java version", "Add manually") em vez de tudo solto sequencialmente;
  botão "Install" ganha `.btn-primary`.
- **`InstallProgressBar.tsx`/`InstanceInstallProgressBar.tsx`**: `<progress>` nativo estilizado
  via `::-webkit-progress-bar`/`::-webkit-progress-value` (barra vermelha de marca sobre trilha
  neutra, cantos arredondados), envolto num `.progress-block` com borda e fundo sutil.

### Verificação

`npx tsc --noEmit` limpo (nenhuma prop nova, só classes CSS e reorganização de JSX). Nenhum
Rust tocado. Verificado ao vivo pela GUI real (`GDK_BACKEND=x11`/`xdotool`, `npx tauri dev`) nas
três telas do escopo — Instances (grade de cards, formulário de criação, botão primário),
Instance detail (cabeçalho/banners, mundo listado herdando os estilos base sem nenhuma classe
nova), Java (lista de instalações, seções, dropdown com anel de foco vermelho, botão "Install"
preenchido de vermelho quando uma versão é selecionada). **Achado de metodologia, não bug do
app**: a primeira leva de screenshots (`spectacle -b -n`, captura de desktop inteiro) mostrava
blocos vermelhos/verdes sólidos "vazando" através da janela do app, alinhados pixel a pixel com
as linhas de diff (removido/adicionado) do editor de código atrás dela — parecia uma
transparência real da janela. Recapturar com `spectacle -b -a` (captura só o buffer da janela
ativa, não o compositor de desktop inteiro) mostrou o app renderizado perfeitamente opaco, sem
nenhum buraco — confirma que era um artefato de composição do desktop (KDE/Wayland via
XWayland, mesma pilha usada pra forçar o WebKitGTK a abrir) capturado no momento exato do
`spectacle -n`, não um bug de CSS/transparência real da aplicação. Vale lembrar: se um "buraco"
aparecer numa screenshot de área de trabalho completa desta app novamente, testar com `-a`
antes de investigar como se fosse um bug de renderização.

**Estado ao final desta fatia**: fundação (paleta, tipografia, botões/inputs base) e as três
telas principais (Instances, Instance detail chrome, Java) aplicadas e verificadas. Pendente pra
uma fatia seguinte: `WorldList`/`WorldHistory`/`WorldBranches`/painel de stats/mapa de chunks da
Fase 4 (hoje só herdam a base, sem layout/classe própria), ícone do app, favicon.

### Segunda continuação — World versioning (mesma sessão, 2026-09-02)

Fecha o restante deixado pendente acima: `WorldList`/`WorldHistory`/`WorldBranches`/o painel de
stats/o mapa de chunks da Fase 4 ganham layout e classes próprias, e os botões destrutivos
(`Disable versioning`, `Delete`/`Delete snapshot`, `Abort merge`) finalmente usam `.btn-danger`
— a variante já existia no design system da primeira fatia mas não tinha nenhum uso real ainda.

Classes novas em `App.css`: `.world-list`/`.world-item` (um card por mundo, mesma linguagem
visual de `.card`/`.install-list`), `.subsection` (painel indentado com borda esquerda — história,
branches, resultado de compare, diff por chunk — "isso pertence à linha acima" sem virar outro
card pesado aninhado), `.snapshot-list`/`.branch-list` (linhas separadas por borda inferior),
`.confirm-box` (o momento "tem certeza?" antes de restore/delete/switch/merge — sempre existiu
como texto solto antes de um par de botões, agora é uma caixa com fundo/borda), `.stats-columns`
(o painel "Show stats" vira 3 colunas de card — Blocks/Structures/Entities — em vez de uma lista
`<ul>` aninhada única), `.chunk-map-legend`/`.swatch` (legenda do mapa de chunks da Fase 4).

`RegionChunkMap.tsx` (Fase 4): as três cores do mapa (`added`/`removed`/`changed`) trocam de hex
literal (`#2e7d32`/`#c62828`/`#f9a825`) pra `var(--color-success)`/`var(--color-danger)`/
`var(--color-warning)` — os únicos tokens do design system ainda não usados em nenhum lugar até
essa fatia — pra que o mapa também respeite claro/escuro como o resto do app, em vez de ficar com
uma cor fixa que só por sorte combinava com o tema escuro.

Decisões de qual botão vira `.btn-primary` vs `.btn-danger` vs neutro, aplicando o mesmo
princípio já documentado na primeira fatia (primário = a ação que a tela mais quer, perigo =
ação destrutiva ou que reverte progresso): `Save snapshot`/`Enable versioning`/`Create branch`/
`Create Backup and Restore`/`Checkpoint and Switch`/`Merge`/`Merge anyway`/`Finish merge` viram
`.btn-primary`; `Disable versioning`/`Delete`/`Delete snapshot`/`Abort merge` viram `.btn-danger`;
`Restore`, `Switch`, `Compare`, `Show/Hide *`, `Cancel`, resolução de conflito (`Keep this/the
other branch's version`) ficam neutros (não são a ação principal do momento, nem destrutivas).

**Verificado ao vivo pela GUI real** (`GDK_BACKEND=x11`/`xdotool`/`spectacle -a`, `npx tauri dev`,
mundo `TestWorld` já existente — só navegação, nenhum snapshot/branch novo criado): card do mundo
com `Save snapshot` vermelho sólido e `Disable versioning` vermelho contornado lado a lado;
"Show history" expandindo um `.subsection` com o snapshot existente, `Delete` em contorno
vermelho; "Show stats" abrindo as 3 colunas de card com contagens reais (blocos, estruturas,
entidades); "Show branches" abrindo `.subsection` com o formulário de criar branch e a branch
`experiment` listada; "Compare" abrindo o resultado como `.subsection`; "Show chunks" abrindo o
mapa 32×32 com a legenda em `.chunk-map-legend`, célula `changed` colorida pelo token
`--color-warning` (tema escuro: amarelo claro `#ffd166`); clique na célula expandindo "Blocks
changed in chunk (0, 0)"/"Structures changed in chunk (0, 0)" como `<h4>` dentro de outro
`.subsection`, mesmo dado de sempre (`minecraft:deepslate[axis=y] → minecraft:stone`). Nenhuma
mudança de dado/estado foi feita no `TestWorld` durante a verificação (confirmado via `git
status`/`git log` no fixture depois — árvore limpa, mesmos dois commits de antes).

`npx tsc --noEmit` limpo. Nenhum Rust tocado.

**Estado ao final desta segunda continuação**: a passada de UX cobre agora fundação + todas as
telas principais (Instances, Instance detail, Java, World versioning completo — histórico,
branches, stats, diff por chunk da Fase 4). Só falta ícone do app e favicon, registrados como
pendência menor, não urgente.

### Terceira continuação — Ícone do app e favicon (mesma sessão, 2026-09-02)

Fecha a última pendência da passada de UX. Símbolo desenhado do zero (não gerado por IA de
imagem — SVG escrito à mão): um "quadrado arredondado" (squircle, `rx=180` num canvas
1024×1024) preenchido com o vermelho de marca (`#e11d2e`) contendo o glifo universal de
"git branch" — três nós (círculos brancos) ligados por uma linha reta (o branch principal) e
uma curva que sai dela até um terceiro nó (a branch derivada) — em vez de uma letra genérica
("M" sozinho foi a primeira tentativa, descartada por não comunicar nada específico do
produto). A escolha comunica as duas metades do nome "mcgit" (Minecraft + git) numa imagem só,
e o desenho (nós + curvas grossas, sem detalhe fino) foi verificado de propósito em 32px e 16px
pra confirmar que continua legível nesses tamanhos pequenos — onde a maioria dos ícones de app
realmente aparece (barra de tarefas, alt-tab, favicon de aba).

Fonte do desenho salva em `apps/desktop/src-tauri/icons/icon-source.svg` (não gerada
automaticamente, é o arquivo de verdade pra qualquer ajuste futuro). Todo o resto do conjunto de
ícones (`32x32.png`, `128x128.png`, `128x128@2x.png`, `icon.icns`, `icon.ico`, `icon.png`, os
`Square*Logo.png`/`StoreLogo.png` do instalador Windows/Appx) foi gerado a partir dela com
`npx tauri icon <fonte.png>` — comando oficial do Tauri CLI que já produz todos os formatos
multi-resolução corretos (`.ico` com 6 tamanhos embutidos, `.icns` nativo do macOS) sem precisar
compor cada arquivo à mão. O gerador também criou pastas `icons/android/`/`icons/ios/` por
padrão — removidas, já que `CONTEXT.md` define mobile fora de escopo (`Cross-Platform
Requirements` é só Linux/Windows/macOS desktop) e `tauri.conf.json` não referencia essas pastas.

Favicon da build web (`apps/desktop/public/favicon.svg`, referenciado em `index.html`) usa o
mesmo SVG — SVG como favicon é suportado por todo navegador/WebView moderno e escala sem
serrilhado em qualquer tamanho de aba, ao contrário de gerar um `.ico` separado só pra isso.
`<title>` do `index.html` também trocado de "Tauri + React + Typescript" (sobra do scaffold,
nunca corrigido) pra "mcgit" — a janela do app já tinha o título certo via `tauri.conf.json`
desde o início, só a aba do dev server (Vite) é que ainda mostrava o nome genérico. Assets não
usados do scaffold padrão (`public/vite.svg`, `public/tauri.svg`, `src/assets/react.svg` — nenhum
importado em lugar nenhum, confirmado por busca antes de apagar) removidos junto.

**Verificado**: `cargo build --workspace` aceita os novos `icon.ico`/`icon.icns` sem erro;
`file` confirma os dois como formatos válidos (`.ico` com ícones de 32×32 e 16×16 reais dentro,
`.icns` tipo `ic10`); `npx tsc --noEmit` limpo. Ao vivo pela GUI real (`GDK_BACKEND=x11`, `npx
tauri dev`): `xprop -id <window> _NET_WM_ICON` confirma que a janela carrega um ícone de verdade
(dados de pixel reais, não o ícone padrão do Tauri/engrenagem) — o painel do KDE usado nesta
sessão não tinha applet de lista de janelas visível na área capturada pra confirmar visualmente
o ícone na barra de tarefas, mas a propriedade X11 correta estar populada já confirma que o
pipeline de geração e o `tauri.conf.json` (`bundle.icon`) estão certos; qualquer ambiente com
lista de janelas visível mostraria o ícone normalmente.

Fecha a pendência: a passada de UX está 100% completa (fundação + todas as telas + identidade
de ícone/favicon).

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
- **`diff_region_chunks`/`blob_size` não tratam um arquivo `.mca` de 0 bytes** (achado na
  Sessão 9, continuação, 2026-09-02, ao corrigir o mesmo problema em `count_region_*`) — um
  arquivo de região vazio é um placeholder real que o próprio Minecraft escreve (visto em
  `entities/`/`poi/` do mundo `medieval`), não corrupção. `world_entity_stats`/`world_block_stats`/
  `world_structure_stats` já tratam isso corretamente (bytes vazios = "sem entradas"), mas
  `diff_region_chunks` (Fase 4/6, comparação de branches) ainda chamaria `Region::from_stream`
  direto num lado vazio e quebraria com o mesmo `UnexpectedEof`. Não corrigido agora — fora do
  pedido desta sessão (era sobre estatísticas, não diff) — mas é o mesmo risco estrutural, só
  ainda não exercitado ao vivo por sorte de dados (nenhum arquivo de `region/` no mundo de teste
  ficou vazio nas comparações já feitas).

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
