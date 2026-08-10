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

**Estado ao final da sessão**: `project/` reflete a visão de launcher completo (PRD v2.0,
roadmap Fase 0-10). Próximo passo natural, quando o usuário quiser retomar código: fechar os
itens restantes da Fase 0 (tanto os de versionamento quanto os novos de arquitetura/launcher)
antes de começar a Fase 1.
