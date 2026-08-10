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

**Estado ao final da sessão**: Fase 0 em andamento. Feito: estrutura de mundo, formato `.mca`,
NBT, benchmark de Git puro com mundo real (crescimento + restauração). Falta: repetir o
benchmark com mundo maior e mais sessões, testar edição de bloco real (não só metadado),
avaliar Git LFS como comparação, decidir `git2`/libgit2 vs binário do sistema, esboçar
integração TruthID/Arweave, e estruturar o workspace Rust (`mcgit-core`/`mcgit-cli`).
