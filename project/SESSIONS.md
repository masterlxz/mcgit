# Log de Sessões

## Sessão 1 — 2026-08-10

**Contexto**: início do projeto. Já existia um documento de ideação
(`minecraft-git-versioner-idea-v2.md`) com a visão, problema, arquitetura conceitual, fluxo
básico, roadmap sugerido (Fase 0-7) e uma lista extensa de perguntas técnicas em aberto.

**O que foi feito**:
- Lida a especificação original completa.
- Lida a estrutura da pasta `project/` do TruthID (`../truthid/project/`) como referência de
  formato: `INDEX.md`, `OVERVIEW.md`, `CONTEXT.md`, `GUIDELINES.md`, `ARCHITECTURE.md`,
  `PENDING.md`, `PHASE.md`, `ROADMAP.md`, `SESSIONS.md`.
- Perguntas de maturação feitas ao usuário e decisões tomadas:
  - **Modo de trabalho**: modo ensino (devagar) — explicar conceitos antes de codar, um de
    cada vez, com analogias, esperando confirmação antes de avançar. Registrado em `GUIDELINES.md`.
  - **Linguagem do MVP**: inicialmente deixada em aberto (decidir na Fase 0 via benchmark).
    Revisado na mesma sessão: usuário propôs travar **Rust com Tauri no backend e React/TS no
    frontend**, reaproveitando a stack do `truthid/desktop`. Decisão: **Rust travado como
    linguagem** (não espera mais o benchmark da Fase 0), e **Tauri + React/TS travado como
    stack de GUI da Fase 7** — mas o MVP (Fase 1) continua CLI-only, seguindo o princípio
    "CLI primeiro" já registrado. Arquitetura ajustada para separar a lógica em uma crate
    `mcgit-core`, consumida tanto pelo binário CLI (`mcgit-cli`, Fase 1) quanto pelo app Tauri
    (Fase 7), sem duplicar lógica de versionamento entre os dois. Registrado em
    `ARCHITECTURE.md`, `OVERVIEW.md` e `PHASE.md` (Fase 0 e Fase 7).
  - **Repositório git**: iniciar `git init` já nesta sessão, como projeto próprio e separado
    do `truthid`.
  - **Monetização**: open source puro (MIT), no mesmo espírito do protocolo TruthID. Intenção
    declarada de eventualmente construir um negócio em cima, mas sem mudar a mentalidade open
    source do core. Registrado em `ROADMAP.md` §Monetização.
- Criada a estrutura `project/` completa (este conjunto de arquivos), espelhando o formato do
  TruthID mas com conteúdo adaptado ao estágio real do projeto (Fase 0, nada implementado
  ainda — sem pendências, sem débitos técnicos, sem fases concluídas).
- Criados `README.md`, `LICENSE` (MIT) e `.gitignore` na raiz do repositório.
- Repositório git inicializado.

- Iniciada a Fase 0 (Pesquisa) na mesma sessão:
  - Usuário cedeu um mundo real e antigo ("Medieval", 40M, tinha backup fora) como cobaia —
    copiado para `benchmarks/worlds/medieval/` (fora do controle de versão).
  - Confirmado empiricamente que `region/` domina o tamanho do mundo (38M de 40M).
  - Explicado o formato `.mca` (Anvil): header de 1024 offsets/timestamps + chunks comprimidos
    individualmente em setores de 4KiB próprios; NBT como formato de dados do jogo.
  - Criada `benchmarks/mca-bench` (ferramenta Rust descartável, `fastnbt`+`fastanvil`) capaz de
    listar/inspecionar/mutar chunks reais de um `.mca` (mutação usada: incrementar
    `InhabitedTime`, campo NBT real que o próprio Minecraft atualiza).
  - Criado `benchmarks/run-benchmark.sh`: cria um repo git descartável com o mundo, commita um
    baseline, simula 6 "sessões de jogo" (mutações reais em 2-3 chunks por sessão na região
    `r.0.0.mca`), e mede o crescimento do `.git`.
  - **Resultado**: sem compactar, o `.git` cresce ~5.3M por snapshot (objetos soltos nunca
    fazem delta). Depois de `git gc --aggressive`, 7 snapshots ficam do tamanho de ~1 (27.3M) —
    o delta do Git aproveita bem o fato de que só ~0,3% dos bytes do arquivo mudam por sessão
    (chunks não tocados ficam byte-idênticos e no mesmo offset). Isso **contraria parcialmente**
    a hipótese inicial (de que a recompressão zlib por chunk atrapalharia o delta). Detalhes,
    ressalvas e o que ainda falta testar: `ARCHITECTURE.md` §Benchmark.
  - Restauração testada: `git checkout` de um snapshot antigo reproduziu o `.mca` bit-a-bit
    idêntico ao original (conferido por hash SHA-256).
  - `PHASE.md` (Fase 0) e `ARCHITECTURE.md` atualizados com os itens concluídos e os dados do
    benchmark. `.gitignore` ajustado para ignorar `benchmarks/worlds/` e `benchmarks/git-bench/`.
- **Documento de ideação original removido**: `minecraft-git-versioner-idea-v2.md` (raiz do
  repositório) apagado pelo usuário durante a sessão, e removido de vez a pedido dele — decisão
  explícita de que **`project/` é a única base do projeto daqui pra frente**, sem um documento
  de brainstorm separado convivendo com os docs "vivos". O conteúdo relevante que só existia lá
  (mockup de GUI) foi movido para `ROADMAP.md`; o resto já tinha sido incorporado a `CONTEXT.md`,
  `ARCHITECTURE.md` e `PHASE.md` mais cedo nesta mesma sessão. `INDEX.md` e `README.md`
  atualizados para não referenciar mais o arquivo removido.

**Estado da sessão nesse ponto**: Fase 0 em andamento. Feito: estrutura de mundo, formato
`.mca`, NBT, benchmark de Git puro com mundo real (crescimento + restauração). Falta: repetir o
benchmark com mundo maior e mais sessões, testar edição de bloco real (não só metadado),
avaliar Git LFS como comparação, decidir `git2`/libgit2 vs binário do sistema, esboçar
integração TruthID/Arweave, e estruturar o workspace Rust (`mcgit-core`/`mcgit-cli`).

---

### Revisão de escopo: de ferramenta de versionamento pra launcher completo (mesma sessão)

**Contexto**: usuário trouxe um segundo documento de ideação (não salvo como arquivo — colado
diretamente na conversa), bem mais ambicioso: um **launcher de Minecraft multiplataforma**
inspirado em Prism Launcher/Modrinth App/ATLauncher, onde o versionamento de mundos via Git
(tudo que o mcgit era até aqui) vira só um módulo entre vários — autenticação Microsoft, Java,
instâncias, mods, modpacks, resource packs, shaders, skins, backup inteligente,
compartilhamento de mundos com reprodutibilidade de ambiente, e (mais adiante) colaboração e
marketplace. Pedido explícito: **atualizar o `project/` pra essa visão futura, sem implementar
nada agora** ("não é pra hoje").

**Inconsistências identificadas e resolvidas com o usuário** (perguntas feitas antes de mexer
nos docs, conforme pedido: "qualquer inconsistência pode me perguntar"):

1. **Identidade do projeto** — "mcgit" continua sendo o nome do produto inteiro (o launcher),
   não vira só o nome de um módulo interno. Decisão do usuário: **manter "mcgit" como nome do
   launcher completo**.
2. **Reconciliação de fases** — o `PHASE.md` antigo (Fase 0-7, só versionamento) e o roadmap do
   prompt novo (Fase 1-5, launcher inteiro) tinham numeração e escopo conflitantes. Decisão:
   **reescrever `PHASE.md` do zero como o roadmap do launcher**, reencaixando tudo que já
   existia (a Fase 0 de pesquisa de mundo/Git, já com resultados reais, e as Fases 1-7 antigas
   do mcgit-ferramenta) dentro da nova estrutura — nada foi descartado.
3. **Stack** — o prompt novo pedia pra não assumir Rust+Tauri sem reavaliar, dado o escopo bem
   maior (gerenciar processos de Java/Minecraft, OAuth, downloads de mods). Decisão do usuário:
   **manter Rust + Tauri + React/TypeScript travados**, sem reabrir a avaliação.

**O que foi atualizado**:
- `CONTEXT.md` reescrito como **PRD v2.0** (v1.0 preservada como base, agora um módulo do PRD
  maior): visão do launcher, conceitos novos (Instance, Java Runtime, Account, Modloader,
  Mod/Modpack, Resource Pack/Shader, Skin, Environment/Reproducibility Metadata, Backup
  Targets), fluxos novos (criação de instância, login Microsoft, instalação de modpack,
  compartilhamento de mundo), arquitetura de módulos, filosofia de interface (GUI é o produto
  principal — ver item 4 abaixo), requisitos de segurança expandidos, requisitos
  multiplataforma, non-goals revisados, e uma seção nova **Legal & Licensing Considerations**
  (requisitos da Microsoft/Mojang pra launchers de terceiros, ToS CurseForge vs Modrinth,
  redistribuição de mods, API de skins — bloqueia código dessas áreas, não bloqueia pesquisa).
- `ARCHITECTURE.md`: nova seção **Arquitetura de Módulos** (Authentication, Java Manager,
  Instance Manager, Mod/Modpack Manager, Skin Manager, World Manager, Git Engine, Backup
  Engine, Arweave Storage, TruthID Integration, Game Runner) com as abstrações
  `StorageProvider` (Local/Cloud/Arweave) e `AuthenticationProvider` (Microsoft/TruthID), e uma
  proposta inicial de workspace Rust (crates por módulo + apps `cli`/`desktop`). Tabela de
  decisões expandida com itens novos (banco local SQLite tentativo, gerenciamento de Java,
  integração de modpacks, fluxo OAuth Microsoft, armazenamento de credenciais). O benchmark de
  Git da Fase 0 original foi mantido intacto — continua válido, só reencaixado.
- **`ARCHITECTURE.md` §Interface — decisão revisada**: "CLI primeiro" (v1.0) virou **"GUI
  primeiro"** — um launcher é fundamentalmente gráfico; CLI passa a ser opcional/paralela, não
  um requisito do MVP. Refletido também em `GUIDELINES.md` (lista de princípios) e `OVERVIEW.md`.
- `PHASE.md` reescrito do zero: **Fase 0 a Fase 10**. Fase 0 ganhou os 25 itens de pesquisa
  arquitetural do prompt novo (viabilidade, APIs oficiais, licenciamento, schema de banco,
  fluxo de auth, etc.) somados ao que já tinha sido feito/planejado pro versionamento de mundo.
  Fase 1 virou o MVP do launcher completo (absorvendo o antigo "MVP local" do mcgit-ferramenta
  como um pedaço dela). Fases 2, 4, 6, 9 mapeiam diretamente pras antigas Fases 2-6 do
  mcgit-ferramenta (qualidade, minecraft-aware diffing, branching, servidores). Fases 3, 5, 7,
  8, 10 são novas (modloaders/mods/modpacks; skins/backup/sync; Arweave+TruthID; compartilhamento
  e reprodutibilidade; colaboração/marketplace/social).
- `OVERVIEW.md`, `README.md`, `ROADMAP.md`, `INDEX.md`, `GUIDELINES.md`: atualizados pra
  refletir a nova identidade, a lista de fases 0-10, e remover referências à numeração antiga.
  `ROADMAP.md` perdeu os itens que viraram fases concretas (GUI, multiplayer) e ganhou uma nova
  leva de perguntas técnicas em aberto específicas do launcher (Java multiplataforma, isolamento
  de instância, CurseForge vs Modrinth, etc.).

**Importante**: nenhum código de produto foi escrito nesta revisão — só documentação, a pedido
explícito do usuário ("não é pra hoje"). O trabalho de código da Fase 0 (benchmark de Git,
`mca-bench`) continua de pé e válido, sem qualquer alteração.

**Estado da sessão nesse ponto**: `project/` reflete a visão de launcher completo (PRD v2.0,
roadmap Fase 0-10). Repositório publicado em https://github.com/masterlxz/mcgit (público),
confirmado que o mundo de teste (`benchmarks/worlds/`) nunca foi versionado nem enviado.

---

### Fechamento da Fase 0 por análise (mesma sessão, orçamento de tokens curto)

Usuário pediu pra fechar a Fase 0 de uma vez, sinalizando pouco orçamento de tokens restante.
Em vez de rodar os experimentos empíricos que faltavam (Git LFS, edição de bloco real, mundo
maior, mundo salvo pelo servidor oficial, e toda a pesquisa nova do launcher — Java, auth
Microsoft, schema de banco, etc.), a maior parte foi **decidida por raciocínio/conhecimento já
disponível**, sem novos testes, e registrada como decisão provisória (reabrir se a Fase 1
mostrar erro):

- Git chamado via binário do sistema (subprocess), não `git2`/libgit2.
- Sem Git LFS no MVP — o benchmark já feito é sinal forte o suficiente.
- `git gc` automático disparado pelo próprio mcgit, não só o auto-gc padrão.
- Banco local: SQLite, schema proposto (`instances`, `accounts`, `worlds`, `mods`, `modpacks`,
  `java_installations`, `backups`, `git_repositories`, `arweave_uploads`, `skins`, `settings`).
- Gerenciamento de Java: baixar builds Eclipse Temurin/Adoptium por major version.
- Gerenciamento de instâncias: pastas isoladas + cache global de libraries/assets compartilhado
  (mesmo padrão de Prism Launcher/MultiMC).
- Fluxo de autenticação Microsoft documentado em detalhe (MS OAuth → Xbox Live → XSTS →
  Minecraft Services) — registro do app no Azure AD fica como ação prática pendente, não
  decisão técnica.
- Integração de modpacks: Modrinth primeiro (API mais aberta), CurseForge condicionado à
  revisão de ToS.
- Arquitetura multiplataforma (filesystem, Java, credenciais, empacotamento) e interfaces
  internas (`StorageProvider`/`AuthenticationProvider` como traits Rust) esboçadas.

Tudo isso foi registrado em `ARCHITECTURE.md` (novas seções: Fluxo de Autenticação Microsoft,
Gerenciamento de Java, Gerenciamento de Instâncias, Schema do Banco Local, Interfaces Internas
Principais, Arquitetura Multiplataforma) e no checklist da Fase 0 em `PHASE.md`.

**3 itens ficaram deliberadamente adiados** (validação empírica real, não decidível só
raciocinando, mas não-bloqueantes pra Fase 1): testar edição de bloco real (não só metadado),
repetir o benchmark com mundo maior/mais sessões, e revalidar contra um mundo salvo pelo
servidor Java oficial (o benchmark atual usou o writer do `fastanvil`, não o servidor real). A
revisão legal/licenciamento (`CONTEXT.md` §Legal & Licensing) também continua aberta — essa
exige pesquisa formal de ToS, não dá pra decidir só raciocinando, e continua bloqueando código
de autenticação Microsoft, CurseForge e API de skins.

**Estado ao final da sessão**: Fase 0 formalmente encerrada (`OVERVIEW.md`/`PHASE.md`
atualizados). Próximo passo natural: começar a Fase 1 (MVP do launcher), ou, se surgir mais
orçamento, fechar os 3 itens empíricos adiados e a revisão legal antes de codar.

---

## Sessão 2 — 2026-08-15

**Contexto**: retomada da Fase 0. Perguntado ao usuário por onde seguir (Fase 1, os 3 itens
empíricos adiados, ou a revisão legal/licenciamento); escolhida a **revisão legal/licenciamento**.

**O que foi feito**: pesquisa real via web (não só raciocínio, já que essa era justamente a
ressalva que mantinha o item aberto) nas 4 áreas sinalizadas em `CONTEXT.md` §Legal &
Licensing, apresentadas ao usuário uma de cada vez (modo ensino) com confirmação entre elas:

1. **Auth Microsoft/Xbox**: o fluxo OAuth já documentado está correto, mas o escopo
   `XboxLive.signin` é restrito — exige inscrição no Xbox Developer Program via **ID@Xbox**
   (mesmo caminho que Prism Launcher/MultiMC percorreram), não é liberado automaticamente pra
   qualquer app registration. Virou **`PENDING.md` #1**: ação prática de duas etapas (criar o
   app registration a qualquer momento; solicitar o escopo via ID@Xbox cedo, já que tem revisão
   humana e prazo desconhecido).
2. **CurseForge**: pior do que "aguardando aprovação de ToS" sugeria. Os termos reais proíbem
   **cachear qualquer dado obtido via API** (tensão direta com os princípios local-first/
   offline-first do projeto) e proíbem competir direta/indiretamente com a plataforma. Chave de
   API exige formulário + revisão humana da Overwolf (critérios: impacto na receita de autores,
   carga de infra, consentimento de autores). Precedente: em mai/2022 o CurseForge tirou a
   capacidade de baixar modpacks de launchers existentes (MultiMC, PCL2) ao lançar a API
   oficial. Decisão de escopo (CurseForge secundário/opcional vs. modo compatível sem cache)
   ficou em aberto, não resolvida nesta sessão.
3. **Modrinth**: confirmado sem bloqueio de ToS — API aberta, sem cláusula de cache/competição,
   rate limit 300 req/min, exige `User-Agent` identificável. Reforça a decisão já tomada de
   "Modrinth primeiro".
4. **API de skins**: sem bloqueio de ToS, mas o endpoint (`api.minecraftservices.com/.../skins`)
   não é documentado oficialmente (só engenharia reversa da comunidade), rate limit apertado
   (~20 req/min), e excesso de 429 pode gerar **suspensão temporária da conta** do jogador —
   vira requisito de engenharia (backoff agressivo), não bloqueio legal.
5. **Bônus (naming/branding)**: diretrizes de marca da Mojang proíbem "Minecraft" como palavra
   dominante do nome de um produto de terceiros — **"mcgit" já está em conformidade**, sem
   necessidade de renomear.

Todos os achados registrados em `CONTEXT.md` §Legal & Licensing Considerations (cada bullet
expandido com "researched (Sessão 2, 2026-08-15)" + fontes), `PENDING.md` (item #1 criado), e
`PHASE.md` (checklist da Fase 0 — item de limitações legais marcado `[x]` com resumo).

**Estado ao final da sessão**: revisão legal/licenciamento formalmente concluída. Únicos itens
de acompanhamento remanescentes: `PENDING.md` #1 (aprovação externa Microsoft/ID@Xbox — ação
externa, não decisão técnica) e a decisão de escopo do CurseForge (ainda em aberto). Os 3 itens
empíricos adiados da Fase 0 (edição de bloco real, benchmark com mundo maior, revalidação contra
mundo do servidor oficial) continuam intocados. Próximo passo natural: começar a Fase 1 (MVP do
launcher).

---

## Sessão 2 (continuação) — 2026-08-16 — Fase 1: Java Manager (primeiro código do launcher)

**Contexto**: usuário pediu pra começar a Fase 1 pelo Java Manager — "tela pra baixar Java de
forma simples, selecionar qual é o padrão e tudo mais". Recorte de escopo discutido antes de
codar: só o Java Manager isolado (sem "instância", que ainda não existe como conceito em
código). Entrei em modo plano (`/plan`), lancei um agente de design com contexto completo da
arquitetura já travada, e confirmei ao vivo (fetch real) o contrato da API do Adoptium antes de
escrever o plano — evitando planejar em cima de suposição. Plano aprovado com 10 incrementos
pequenos, cada um com comando de verificação e checkpoint de ensino (modo ensino continua ativo
— ver `GUIDELINES.md`).

**O que foi feito** (primeira vez que o repositório tem código de produto de verdade):

- **Workspace Rust criado**: `Cargo.toml` raiz + `crates/mcgit-java` (biblioteca pura, sem
  Tauri/SQLite) + `crates/mcgit-db` (acesso a SQLite via `rusqlite`, feature `bundled`) +
  `apps/desktop` (Tauri 2 + React/TS, scaffolded via `npm create tauri-app@latest`, renomeado
  de `appsdesktop` pro `mcgit-desktop`/`mcgit`).
- **`mcgit-java`**: detecção de Java no sistema (`platform::linux` — Windows/macOS ficam atrás
  de `#[cfg(target_os)]` sem corpo ainda, próximo passo natural quando alguém rodar nessas
  plataformas), parser de `java -version` (cobre Temurin/Oracle/Corretto/OpenJDK genérico, dois
  esquemas de versão), cliente da API do Adoptium (`available_releases`/`latest_asset`,
  confirmado contra a API real), e instalação completa (download em streaming, verificação de
  checksum sha256 antes de extrair, extração por SO via `#[cfg]` — `tar`+`flate2` em
  Linux/macOS, `zip` no Windows — localização do binário na árvore extraída).
- **`mcgit-db`**: schema `java_installations` estendido (`source`, `is_default` com índice único
  parcial garantindo "só um padrão" no nível do banco, não da aplicação) + CRUD completo, 100%
  testado com banco em memória.
- **`apps/desktop`**: `AppState` (`Mutex<Db>` + diretório de Java gerenciado, resolvido via
  `directories::ProjectDirs`, nunca caminho fixo), 6 comandos Tauri conectando `mcgit-java` +
  `mcgit-db` (único ponto de acoplamento, como a arquitetura exige), tela React funcional
  (`JavaManagerScreen` + 4 componentes) consumindo os comandos via `invoke()`/`listen()`.
- **Testado de ponta a ponta pela GUI real**: `npm install` + `cargo tauri dev` abriram uma
  janela de verdade neste ambiente (sandbox tinha os requisitos: `webkit2gtk-4.1`,
  `DISPLAY`/`WAYLAND_DISPLAY`) — usuário confirmou visualmente cada etapa. Um JDK 25 real foi
  baixado, verificado, extraído e instalado clicando em "Install" na tela; marcado como padrão
  clicando em "Set as default"; fechado o app e reaberto, a lista carregou já com o Java 25
  marcado como padrão — prova de que veio do SQLite (`~/.local/share/mcgit/mcgit.sqlite3`), não
  de estado em memória do React. Binário conferido rodando `java -version` de verdade fora do
  app também.
- Dois imprevistos de dependências resolvidos durante a implementação (cache do índice do
  crates.io desatualizado pra `futures-util`/sub-dependências recém-publicadas; mudança de API
  do `sha2` 0.11 exigindo laço manual de leitura em vez de `std::io::copy`/`format!("{:x}", ...)`)
  — nenhum dos dois é decisão de arquitetura, só ajuste de implementação.
- `ARCHITECTURE.md` atualizado: seção "Java Manager — implementado" nova, schema do banco
  estendido documentado, workspace confirmado em parte, decisão `rusqlite` vs `sqlx` registrada,
  e uma seção nova "Débitos Técnicos de Arquitetura" com 4 itens conscientes (plataformas
  faltando, extração bloqueante dentro de `async fn`, eventos de progresso sem throttle, Mutex
  sem `spawn_blocking`) — nenhum bloqueou o funcionamento real, mas ficam registrados pra não
  parecerem descuido depois. `PHASE.md` (Fase 1): item "Gerenciamento de Java" marcado `[x]`,
  com nota de que falta a parte de detectar a versão *necessária* por instância (bloqueado por
  "instância" não existir ainda em código, não por decisão técnica).

**Estado ao final da sessão**: Java Manager funcionando de ponta a ponta, testado na GUI real
por download de um JDK de verdade. Próximo passo natural: continuar a Fase 1 — instância,
Minecraft Vanilla, ou autenticação Microsoft (essa última ainda esperando a aprovação externa do
`PENDING.md` #1).

---

## Sessão 2 (continuação) — 2026-08-16 — Migração de `mcgit-db` pra SeaORM

**Contexto**: ao planejar a próxima peça da Fase 1 (Instância + instalação do Vanilla), o modo
plano foi usado de novo — um agente de design pesquisou e confirmou ao vivo o contrato da API
`piston-meta` da Mojang (incluindo `javaVersion.majorVersion`, que fecha o item que estava
bloqueado por falta de instância) e devolveu um plano detalhado pra `crates/mcgit-minecraft` +
`crates/mcgit-instance` + comandos Tauri + tela React. Antes de aprovar esse plano, ao revisar a
tabela `instances` nova, o gatilho de migração já registrado em `ARCHITECTURE.md` ("na próxima
tabela nova, trocar `schema.sql` único por `rusqlite_migration`") foi puxado — e o usuário, em
vez do meio-termo, perguntou diretamente "não acha que faz mais sentido já usar o ORM?"
(referindo-se ao SeaORM, que ele já usa em outro projeto do mesmo ecossistema). Raciocínio
aceito: com ~10 tabelas planejadas no PRD completo, trocar de fundação agora (2 tabelas) é mais
barato que trocar depois (10 tabelas de SQL cru). Decisão confirmada explicitamente.

**Escopo desta sessão, replanejado**: só a migração da fundação (`mcgit-db` inteiro pra SeaORM,
reaplicando `java_installations` como migration 1) — a feature de Instância + Vanilla install já
desenhada fica pra uma sessão seguinte, construída em cima dessa fundação nova. Plano anterior
(Java Manager) sobrescrito no arquivo de plano, já que era tarefa diferente.

**O que foi feito**:

- `crates/mcgit-db` reescrito por completo: `rusqlite` saiu, `sea-orm`+`sea-orm-migration` 2.0
  entraram (dependências verificadas via `cargo add --dry-run` antes de travar no plano — mesma
  disciplina de "confirmar ao vivo antes de assumir" usada pra APIs externas). Entidade
  `java_installations` via `#[derive(DeriveEntityModel)]`; `JavaSource` virou um enum tipado via
  `DeriveActiveEnum` (string desconhecida no banco agora é erro real, não mais um fallback
  silencioso pra "detected" como no código à mão). `models.rs` (structs escritas manualmente)
  foi removido — a entidade gerada já é o tipo de retorno direto do CRUD.
- Primeira migration real do projeto (`m20260816_000001_create_java_installations.rs`),
  reaplicando exatamente o schema que já estava testado — escrita como SQL cru dentro da
  migration (`execute_unprepared`) de propósito, porque o construtor de schema do SeaORM não
  cobre bem valor-padrão-por-função (`datetime('now')`) nem índice único parcial
  (`WHERE is_default = 1`) — usar o escape hatch documentado em vez de forçar esses dois casos
  numa API pensada pro caso comum.
- `Db::open`/`open_in_memory` viraram `async` (efeito cascata: testes de `mcgit-db` viraram
  `#[tokio::test]`, mesmo padrão já usado nos testes de rede do `mcgit-java`). `java.rs`
  reescrito com `ActiveModel`/`Set(...)`/transação via closure — os mesmos 4 testes de antes
  passaram sem ajuste na lógica, só na sintaxe.
- `apps/desktop`: `AppState.db` deixou de ser `Mutex<Db>` — `DatabaseConnection` do SeaORM já é
  um pool interno seguro pra concorrência, o mutex virou trabalho redundante. Isso fecha um dos 4
  débitos técnicos registrados na sessão anterior. `.setup()` do Tauri usa
  `tauri::async_runtime::block_on` pra rodar a abertura/migração assíncrona do banco uma vez na
  inicialização.
- **Imprevisto real durante a verificação**: o disco ficou 100% cheio (177G de 187G, só 66M
  livres) por causa da árvore de dependências grande que o SeaORM trouxe (`sqlx`, `sea-query`,
  etc.) somada a lixo de builds antigas do `rusqlite`. Resolvido com `cargo clean` (11.5GB
  liberados) — build recompilado depois disso sem problema. Vale de olho: o workspace inteiro
  ocupa bastante espaço em builds de debug; considerar `cargo clean` periódico ou builds em
  modo release pra verificação final, se o disco continuar apertado.
- **Gap real descoberto testando de verdade**: depois de resetar o banco de teste (necessário —
  o arquivo antigo não tinha a tabela `seaql_migrations` do SeaORM), "Scan for Java" não achou o
  JDK 25 que já estava instalado em disco, porque `scan_system_java` só varre locais de
  *sistema* (`/usr/lib/jvm`, `JAVA_HOME`, `PATH`), nunca a pasta gerenciada do próprio mcgit.
  Contornado na hora com "Add manual Java" apontando pro binário direto (funciona, mas registra
  como `source='manual'`, semanticamente errado). Registrado como gap real em
  `ARCHITECTURE.md` §Débitos Técnicos — não corrigido nesta sessão (fora do escopo, que era só a
  fundação de banco).
- Verificação completa pela GUI real: banco resetado → app reaberto → "Add manual Java" pro JDK
  já em disco → "Set as default" → confirmado no arquivo SQLite direto (`is_default=1`, e a
  tabela `seaql_migrations` mostrando a migration aplicada de verdade) → app fechado e reaberto →
  Java 25 continuou aparecendo como padrão. Mesma régua de prova da sessão anterior, agora contra
  a fundação nova.
- `ARCHITECTURE.md` atualizado: linha de decisão SQLite revisada (SeaORM final, não mais
  "adiado"), linha do gatilho de migração marcada como resolvida (SeaORM trouxe migrations
  embutidas), débito técnico do `Mutex<Db>` marcado como fechado, e o gap do "Scan" registrado
  como novo débito técnico.

**Estado ao final da sessão**: `mcgit-db` numa fundação SeaORM funcionando e verificada pela GUI
real. A feature completa de Instância + instalação do Vanilla (já desenhada em detalhe, contrato
da API do piston-meta confirmado ao vivo) fica pronta pra virar o próximo plano. Pendências que
seguem de sessões anteriores: `PENDING.md` #1 (aprovação Microsoft/ID@Xbox), decisão de escopo
do CurseForge, e agora também o gap do "Scan for Java" não redescobrir instalações gerenciadas.

---

## Sessão 3 — 2026-08-16 — ID@Xbox/Azure investigados e pausados; Fase 1: Instância + Vanilla Install

**Parte 1 — tentativa real de destravar `PENDING.md` #1 (login Microsoft)**: usuário tentou
preencher o formulário de cadastro de Xbox Partner (pré-requisito do ID@Xbox), com prints reais
da tela. Achados que mudam o que se sabia antes:

- O campo **DUNS Number** é obrigatório na prática (trava o avanço), apesar do texto de ajuda do
  próprio formulário sugerir que seria opcional.
- Tirar um DUNS number no Brasil **exige CNPJ** — não emite pra CPF de pessoa física sem empresa
  registrada.
- Cadeia de dependência completa: abrir CNPJ (ex.: MEI) → tirar DUNS number → cadastro de Xbox
  Partner → aplicação ID@Xbox → aprovação humana do `XboxLive.signin`. Bem mais pesada que o
  "só preencher formulário" assumido antes.
- Em paralelo, o **app registration no Azure** — que se achava autosserviço, sem pré-requisito —
  também esbarrou: a Microsoft descontinuou criar apps fora de um "directory" (tenant). O M365
  Developer Program (a saída sem cartão de crédito) **não qualificou** a conta do usuário
  (política atual reserva o sandbox gratuito principalmente pra assinantes do Visual Studio). A
  única saída restante é conta Azure gratuita, que exige cartão pra verificação de identidade
  (sem cobrança, mas exige).
- **Decisão do usuário**: pausar as duas frentes (ID@Xbox e Azure) por ora. Detalhe completo em
  `PENDING.md` #1.

**Parte 2 — Instância + instalação do Vanilla (Fase 1)**: com o login MS pausado, seguiu pro
próximo item não-bloqueado da Fase 1. Escopo combinado explicitamente com o usuário via
`AskUserQuestion`: entrega "instância criada + Vanilla baixado/verificado em disco", **sem**
lançar o jogo (Game Runner fica pra depois, quando/se o login destravar). Navegação: instalado
`react-router` (com `HashRouter`, necessário porque um app Tauri não tem servidor resolvendo
paths arbitrários) em vez do `useState` de aba simples que era o default proposto — decisão
também confirmada com o usuário.

- Planejamento em `/plan` real: 3 agentes de exploração em paralelo (docs do projeto, padrões de
  código de `mcgit-db`/`mcgit-java`, padrões da UI React) + 1 agente de design, que **reverificou
  ao vivo o contrato do `piston-meta`** — a versão "confirmada" numa sessão anterior tinha se
  perdido junto com o arquivo de plano sobrescrito, então não podia ser reaproveitada de memória.
- Construído incremento por incremento (13 no total, cada um com build/teste verificando antes
  do próximo), seguindo `modo ensino`: crates novos `mcgit-minecraft` (cliente do piston-meta) e
  `mcgit-instance` (scaffolding de pastas), tabela `instances` no `mcgit-db` (primeira relação
  real via SeaORM), 3 comandos Tauri novos, telas React novas. Detalhes técnicos completos em
  `ARCHITECTURE.md` §Instância + Vanilla Install (implementado) e `PHASE.md` Fase 1.
- **Bug de compilação real encontrado e corrigido**: o macro de comandos do Tauri rejeitou a
  primeira versão do código de download em paralelo (`buffer_unordered` sobre um iterador de
  referências emprestadas) com erro de lifetime (`FnOnce is not general enough`) — um limite
  conhecido de inferência do rustc. Corrigido clonando os itens antes de iterar.
- **Disco ficou apertado de novo** (a mesma classe de problema da Sessão 2 com o SeaORM): só
  3.5GB livres antes da verificação manual. Resolvido com `cargo clean` (9.2GB liberados) antes
  de rodar o `tauri dev` — decisão confirmada com o usuário via `AskUserQuestion` em vez de
  assumida.
- **Verificação de ponta a ponta pela GUI real**, feita pelo próprio usuário (sem ferramenta de
  automação de GUI disponível nesta sessão): criou uma instância pra Minecraft 26.2 de verdade.
  Confirmado depois via disco/banco: `client.jar` com exatamente 39.193.383 bytes (bate com o
  valor real capturado direto da API da Mojang), ~468MB de assets e ~77MB de libraries no cache
  compartilhado, `instance.json` com `java_installation_path` resolvido, linha no banco com
  `status='ready'`. A resolução de Java reaproveitou o JDK 25 já instalado (sem baixar de novo),
  confirmando que a detecção de "instalação existente ainda válida" funciona antes de partir pro
  download.

**Estado ao final da sessão**: Fase 1 tem Java Manager + Instância/Vanilla Install completos e
verificados. Login Microsoft, Game Runner, e resto da Fase 1 (mundos, versionamento Git,
snapshots) seguem não implementados. Pendências que seguem de sessões anteriores: decisão de
escopo do CurseForge; `PENDING.md` #1 agora com a cadeia de dependência CNPJ→DUNS→ID@Xbox
mapeada, mas pausado por decisão do usuário.

---

## Sessão 4 — 2026-08-22 — Fase 1: Ativar/desativar versionamento Git num mundo

Com login Microsoft ainda pausado (`PENDING.md` #1) e Java Manager + Instância/Vanilla Install já
prontos, seguiu pro próximo item não-bloqueado da Fase 1: nasce o **Git Engine**, o módulo que dá
nome ao projeto.

- **`crates/mcgit-core`** (biblioteca pura, mesmo princípio de `mcgit-java`/`mcgit-minecraft`):
  primeira aplicação real da decisão da Fase 0 de chamar o binário `git` do sistema via
  subprocess em vez de `git2`/libgit2. Só duas funções por enquanto: `git::init` (roda
  `git init` via `std::process::Command`, idempotente por garantia do próprio Git — não precisa
  checar `is_repository` antes) e `git::is_repository`.
- **Tabela `worlds`** (`mcgit-db`): primeira FK `NOT NULL`/`ON DELETE CASCADE` do projeto (pra
  `instances`) — diferente do `SET NULL` de `instances.java_installation_id`, porque um `world`
  sem instância não tem significado. Testado de verdade
  (`deleting_instance_cascades_to_worlds`), não só assumido pelo SQL. `set_git_enabled` é
  find-or-create, cobrindo ativar-pela-primeira-vez e reativar-depois-de-desativar sem duplicar
  linha (índice único em `instance_id, folder_name` garante isso no banco também).
- **3 comandos Tauri novos**: `list_worlds` (cruza filesystem — `saves/*` com `level.dat` é a
  lista real de mundos — com o banco, que só complementa `git_enabled`), `enable_world_
  versioning` (roda `git init` dentro de `spawn_blocking`, evitando repetir o débito técnico do
  Java Manager de bloquear o executor async), `disable_world_versioning`.
- **Decisão de produto confirmada**: "desativar" nunca apaga `.git`/histórico, só esconde a ação
  na UI — reversível a qualquer momento reativando o flag.
- Botão por mundo na tela de detalhe da instância (`InstanceDetailScreen.tsx`/`WorldList.tsx`).

Detalhes técnicos completos em `ARCHITECTURE.md` §Git Engine e §Schema do Banco Local; checklist
atualizado em `PHASE.md` Fase 1.

**Estado ao final da sessão**: Fase 1 tem Java Manager, Instância/Vanilla Install e
Ativar/desativar versionamento Git implementados e commitados (`4705add`). Próximo item natural:
criar versão/snapshot (`git commit` por trás de "Salvar versão" na UI). Login Microsoft e Game
Runner seguem pausados/bloqueados por `PENDING.md` #1; decisão de escopo do CurseForge segue em
aberto, não urgente.

---

## Sessão 4 (continuação) — 2026-08-22 — Fase 1: Criar versão/snapshot

Sessão iniciada com um fechamento retroativo: o commit anterior (`4705add`, ativar/desativar
versionamento) tinha ido pro repositório sem a atualização correspondente de `PHASE.md`/
`OVERVIEW.md`/`SESSIONS.md`. Fechado num commit próprio (`dcbe9c4`) antes de seguir pra feature
nova, e registrado como aprendizado: checar sincronia doc/código no início de uma sessão quando
o fim da anterior não está claro, em vez de assumir que os docs estão em dia.

Planejamento em `/plan` real: 1 agente de exploração (leu o Git Engine e a UI de mundo byte a
byte) + 1 agente de design (validou o desenho tentativo, corrigiu 3 pontos — `CommitOutcome`
pertence a `git.rs` não a `types.rs`; a mensagem padrão nunca deveria vir de um timestamp gerado
em Rust, e sim do frontend, pra não introduzir `chrono` como primeira dependência nova do
`mcgit-core`; e faltava um canal de feedback separado do `error` pra "nada mudou" não parecer uma
falha). Duas decisões de produto confirmadas com o usuário antes de implementar: botão + campo de
mensagem opcional (não só um botão), e identidade Git fixa `mcgit <mcgit@localhost>` pros
commits automáticos.

Implementado em 7 incrementos de modo ensino, cada um com seu próprio checkpoint de build/teste:
extrair `run()` (refactor puro) → `CommitOutcome` → `ensure_identity` (`--local`, sempre ganha de
`--global`/`--system`) → `commit()` completo + 3 testes novos (6/6 verdes) → ponte Tauri
(`create_world_snapshot`) → API TS → UI (`SaveSnapshotForm.tsx`, espelhando
`AddManualJavaForm.tsx`). Sem dependência nova em `mcgit-core` — a mensagem-padrão-por-timestamp
é gerada em JS, não em Rust, então `message` nunca chega vazia em `commit()` (`git commit -m ""`
falharia de propósito).

**Verificação ao vivo pela GUI real, pela primeira vez conduzida pelo próprio Claude** (sem
ferramenta de automação de GUI dedicada disponível — Tauri não é Electron, `_electron` do
Playwright não se aplica): app rodado com `GDK_BACKEND=x11` sob a sessão Wayland/KDE real da
máquina, o que faz a janela nativa (WebKitGTK) renderizar via XWayland; `xdotool` então
manipula cliques/teclado como eventos de hardware simulados de verdade (`XTEST`, não eventos
sintéticos por janela, que a maioria dos toolkits ignora); `spectacle -b -a -e -S` (KDE, modo
background, janela ativa, sem decoração/sombra) tira screenshots que batem pixel a pixel com a
área de conteúdo de 800x600 declarada em `tauri.conf.json`, permitindo mapear coordenadas de
clique direto da imagem. Mundo fake criado manualmente (`saves/Snapshot4Test/level.dat`) dentro
da instância real já existente de sessões anteriores (Game Runner segue bloqueado, mesma
limitação de sempre). Fluxo completo confirmado: habilitar versionamento → `.git` criado de
verdade → "Save snapshot" com mensagem customizada → commit real (`git log` confirmou autor
`mcgit <mcgit@localhost>` e mensagem certa) → "Save snapshot" de novo sem mudança → "Nothing
changed since the last snapshot." em texto normal, não vermelho, nenhum commit vazio criado →
campo em branco + mudança real → mensagem por timestamp usada de fato. Um checkpoint extra, só
de verificação: simulado um ambiente sem `HOME`/config Git global nenhuma
(`env -i HOME=/tmp/inexistente GIT_CONFIG_NOSYSTEM=1`) — confirmado que o commit falha sem
identidade e funciona depois de aplicar o mesmo `--local` que `ensure_identity` roda, provando
que a decisão de design realmente resolve o problema que motivou ela. Dados de teste (pasta fake
e as linhas órfãs de `worlds` no banco real, incluindo uma de uma sessão anterior) foram
limpos do ambiente real do usuário ao final.

Detalhes técnicos completos em `ARCHITECTURE.md` §Git Engine (subseção "Criar versão/snapshot");
checklist atualizado em `PHASE.md` Fase 1.

**Estado ao final da sessão**: Fase 1 tem Java Manager, Instância/Vanilla Install, e
Ativar/desativar + Criar snapshot do Git Engine implementados e verificados ao vivo. Próximos
itens não-bloqueados do checklist: "Ver histórico de versões" e "Restaurar uma versão" (ambos
podem ler `git log`/`git checkout` diretamente, sem duplicar dados no SQLite — decisão já
deixada preparada nesta sessão). Login Microsoft e Game Runner seguem pausados por `PENDING.md`
#1; decisão de escopo do CurseForge segue em aberto, não urgente.

---

## Sessão 5 — 2026-08-22 — Fase 1: Ver histórico de versões

Sessão iniciada com "bora continuar" — conferido `git log` contra `PHASE.md` antes de escolher o
próximo passo (working tree limpo, tudo sincronizado desde o fechamento da Sessão 4). Próximo
item não-bloqueado do checklist: "Ver histórico de versões", terceiro slice do Git Engine.

Duas decisões de UX confirmadas com o usuário via `AskUserQuestion` antes de planejar: botão
"Ver histórico" sob demanda por mundo (não sempre visível) e mostrar todos os snapshots de uma
vez (sem paginação por ora).

- **`crates/mcgit-core`**: `log()` reaproveita o chokepoint `run()` já existente. Formato pedido
  ao Git: `git log --pretty=format:%H\x1f%aI\x1f%s` (campos separados por `\x1f`, um commit por
  linha, `%s` sempre uma linha só). Dois casos viram lista vazia, não erro: mundo nunca
  `git init`ado e mundo versionado sem nenhum snapshot ainda (Git recusa `git log` num repo sem
  commits; esse stderr específico é reconhecido e vira `Ok(vec![])`, mesmo espírito do
  `CommitOutcome::NothingToCommit` da sessão anterior). Sem dependência nova — datas ficam como
  string ISO 8601 crua, formatadas no frontend. 4 testes novos, 10/10 verdes no crate.
- **Ponte Tauri**: `list_world_history`, mesmo formato dos comandos existentes. O `SnapshotDto`
  que já existia (resultado de salvar snapshot) foi renomeado pra `SnapshotResultDto` pra abrir
  espaço pro novo `SnapshotDto` do histórico (forma diferente: `hash`/`date`/`message`).
- **UI**: botão "Show history"/"Hide history" por mundo em `WorldList.tsx`, carregado sob
  demanda (só busca no primeiro clique que expande, guarda em `historyByWorld` pra não rebuscar
  ao recolher/expandir de novo). Novo componente `WorldHistory.tsx` (recebe dados via prop,
  mesmo padrão sem-`invoke`-direto de `SaveSnapshotForm`).

**Bug real encontrado e corrigido durante a verificação ao vivo pela GUI**: salvar um snapshot
novo com o painel de histórico já aberto não atualizava a lista — ficava mostrando o estado
antigo até fechar/reabrir o painel manualmente. Corrigido em `handleSaveSnapshot`: se o
histórico daquele mundo já tinha sido carregado antes, ele é recarregado automaticamente depois
de um snapshot criado com sucesso.

Implementado em incrementos de modo ensino (explicando `git log --pretty=format`, o separador
`\x1f`, e o caso "repo sem commits" antes do código de `log()`; depois o padrão sob-demanda do
React antes do código de frontend), cada um com seu checkpoint de build/teste.

**Verificado ao vivo pela GUI real** (mesma técnica de automação da Sessão 4 —
`GDK_BACKEND=x11` + `xdotool` + `spectacle`, mundo fake criado dentro da pasta `saves/` da
instância real já existente, dados de teste limpos do ambiente do usuário ao final, igual à
sessão anterior): botão de histórico ausente num mundo nunca versionado; "No snapshots yet."
num mundo versionado sem snapshot nenhum; 1 snapshot com hash/data/mensagem corretos; 2º e 3º
snapshots na ordem certa (mais recente primeiro); atualização automática do histórico aberto
após salvar um novo snapshot (a correção do bug acima, confirmada funcionando).

Detalhes técnicos completos em `ARCHITECTURE.md` §Git Engine (subseção "Ver histórico de
versões"); checklist atualizado em `PHASE.md` Fase 1.

**Estado ao final da sessão**: Fase 1 tem Java Manager, Instância/Vanilla Install, e
Ativar/desativar + Criar snapshot + Ver histórico do Git Engine implementados e verificados ao
vivo. Próximo item não-bloqueado do checklist: "Restaurar uma versão" (`git checkout`,
checagem de mundo aberto, checkpoint de segurança antes de restaurar). Login Microsoft e Game
Runner seguem pausados por `PENDING.md` #1; decisão de escopo do CurseForge segue em aberto,
não urgente.
