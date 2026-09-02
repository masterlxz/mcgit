## Fases Detalhadas

Convenção: `[ ]` não iniciado, `[~]` em andamento, `[x]` concluído. Atualizar aqui e no
checklist resumido de `OVERVIEW.md` ao final de cada sessão.

> **Reconciliação (Sessão 1, revisão 2)**: este arquivo cobria só o mcgit-ferramenta-de-versionamento
> (Fase 0-7). O escopo virou um launcher completo (ver `CONTEXT.md` v2.0). Nada do que já foi
> pesquisado/decidido se perdeu — só foi reencaixado nas fases novas abaixo. Onde uma fase
> corresponde diretamente a uma fase antiga, isso está anotado.

---

## Fase 0 — Pesquisa & Arquitetura ✅ (encerrada na Sessão 1 — ver ressalvas no fim da fase)

**Objetivo**: entender o domínio (mundos Minecraft *e* o ecossistema de launchers/mods/Java/contas)
e validar/descartar premissas técnicas antes de escrever código de produto. Nada aqui deve virar
código do MVP diretamente — é investigação, benchmark e desenho de arquitetura.

### Já feito (versionamento de mundo, Sessão 1)

- [x] Entender a estrutura de diretórios de um mundo Minecraft (single-player e servidor)
- [x] Entender o formato `.mca` (Anvil region format)
- [x] Entender NBT — usando `fastnbt`/`fastanvil`, testado contra mundo real
- [x] Testar Git puro (`git init && git add && git commit`) com mundo real
- [x] Medir o tamanho do repositório `.git` após múltiplos snapshots
- [x] Testar restauração (`git checkout`) e validar integridade (hash idêntico)

Detalhes completos do benchmark: `ARCHITECTURE.md` §Benchmark.

### Decidido por análise (Sessão 1, fechamento da Fase 0 — sem novo experimento)

Orçamento de tokens curto nesta sessão: as decisões abaixo foram tomadas raciocinando a partir
do benchmark já feito + conhecimento de como Git e outros launchers resolvem isso, não com
novos testes. São decisões provisórias — reabrir se a Fase 1 mostrar que alguma está errada.

- [x] Git LFS: **não usar no MVP.** O benchmark já mostrou Git puro + `git gc` periódico
  resolvendo o caso comum (7 snapshots ≈ tamanho de 1). LFS troca esse ganho por uma
  dependência extra (servidor LFS do lado do remote); só reavaliar se um mundo real em
  produção mostrar o contrário.
- [x] Como chamar o Git: **binário `git` do sistema via subprocess**, não `git2`/libgit2 — mais
  simples, sem custo de build/linking cross-platform de uma lib C. `git2` fica de reserva se
  algum dia a CLI do git não der conta de algo fino o suficiente.
- [x] Compactação: **mcgit dispara `git gc` automaticamente** (após N snapshots ou em
  background), não depende só do auto-gc padrão do Git — o benchmark mostrou que sem isso o
  repositório incha rápido demais.

### Adiado, não-bloqueante pra Fase 1 (validação empírica futura)

- [ ] Testar edição de bloco real (block-states packed), não só metadado
- [ ] Repetir o benchmark com mundo maior e mais sessões (10s-100s)
- [ ] Revalidar contra um mundo salvo pelo servidor Java oficial (não só `fastanvil`)

### Launcher — decidido por análise (Sessão 1)

- [x] Viabilidade técnica: **viável** — todos os componentes (OAuth, download de assets, Java,
  execução de processo, Git) já são resolvidos por launchers existentes; o diferencial do
  mcgit é a composição (Git + UX simples), não uma técnica inédita.
- [x] APIs oficiais identificadas: Microsoft Identity Platform (OAuth) → Xbox Live → XSTS →
  Minecraft Services API (posse do jogo, perfil, skins); Mojang piston-meta (manifesto de
  versões/libraries/natives); Modrinth API (aberta); CurseForge API (API key aprovada — ver
  Legal & Licensing). Detalhe do fluxo: `ARCHITECTURE.md` §Fluxo de Autenticação Microsoft.
- [x] Como launchers existentes resolvem isso: manifesto de versão → baixar client jar +
  libraries + natives → resolver Java → montar argumentos JVM → executar. Prism/MultiMC usam
  pastas de instância isoladas + bibliotecas compartilhadas num cache global — replicamos o
  conceito de isolamento, não o formato de arquivo deles.
- [x] Arquitetura de módulos, stack, estrutura de diretórios, módulos/responsabilidades, schema
  de banco, interfaces internas, fluxo de auth Microsoft, gerenciamento de Java, gerenciamento
  de instâncias, arquitetura multiplataforma: todos detalhados em `ARCHITECTURE.md` (seções
  novas desta sessão).
- [x] Gerenciamento de modpacks: Modrinth primeiro (API aberta), CurseForge condicionado à
  revisão de ToS — já refletido em `PHASE.md` Fase 3.
- [x] Estratégia de snapshots/backups: `CONTEXT.md` §Backup Targets + `PHASE.md` Fase 2/5.
- [x] Integração Arweave/TruthID sem acoplamento cedo: abstrações `StorageProvider`/
  `AuthenticationProvider`, `ARCHITECTURE.md`.
- [x] Modelo de segurança: `CONTEXT.md` §Security Requirements.
- [x] MVP definido: `PHASE.md` Fase 1. Milestones: a própria lista de checkboxes da Fase 1 já
  funciona como milestones sequenciais (auth → Java → instância → jogar → versionamento).
- [x] Limitações legais/licenciamento: revisão formal feita (Sessão 2, 2026-08-15) — resultado
  completo em `CONTEXT.md` §Legal & Licensing Considerations. Resumo: (1) auth Microsoft precisa
  de aprovação externa do escopo `XboxLive.signin` via ID@Xbox antes de testes reais de login —
  registrado como `PENDING.md` #1, não bloqueia mais código de auth em si (o app registration
  Azure pode ser criado a qualquer momento), só bloqueia login funcionando de ponta a ponta; (2)
  CurseForge tem restrição de ToS mais séria do que se imaginava (proíbe cache de dados da API —
  tensão real com local-first/offline-first) — decisão de escopo ainda em aberto, não apenas
  aprovação de chave; (3) Modrinth confirmado sem bloqueio de ToS, só rate limit (300 req/min) e
  `User-Agent` obrigatório; (4) skins não tem bloqueio de ToS mas o endpoint não é documentado
  oficialmente e tem rate limit apertado (~20 req/min) com risco de suspensão de conta —
  requisito de engenharia (backoff), não legal; (5) nome "mcgit" já está em conformidade com as
  diretrizes de marca da Mojang, sem necessidade de renomear.

**Fase 0 encerrada** neste nível de profundidade. Os 3 itens empíricos "adiados" continuam
abertos, mas não travam o início da Fase 1. A revisão legal está formalmente feita; só
`PENDING.md` #1 (aprovação externa da Microsoft) e a decisão de escopo do CurseForge continuam
pendentes como itens de acompanhamento, não como bloqueio de pesquisa.

**Critério de saída da Fase 0**: arquitetura de módulos, schema do banco, fluxo de auth
Microsoft, estratégia de armazenamento de `.mca`, e revisão legal/licenciamento documentados em
`ARCHITECTURE.md`/`CONTEXT.md` com base em pesquisa real, não só preferência.

---

> **Reordenação (Sessão 8, 2026-09-01)**: a partir desta sessão, o usuário pediu para priorizar
> versionamento de mundo (incluindo branches) e a GUI, à frente do resto do escopo do launcher.
> A numeração das fases abaixo continua sendo a organização temática original — não foi
> renumerada, pra não invalidar todas as referências cruzadas já escritas (`SESSIONS.md`,
> `ARCHITECTURE.md`, commits) — mas a **ordem de execução real** passa a ser: fechar os itens de
> Git ainda pendentes na Fase 1 (validações de segurança, testes de restauração) → **Fase 6
> (Branching)**, começando por "criar/trocar branch" → Fase 2 (qualidade do versionamento:
> auto-snapshot, auto-gc) → Fase 4 (diff entre snapshots) → só então retomar Fase 3 (mods), Fase
> 5 (backup/sync) e Fase 7 (Arweave) na ordem antiga. Login Microsoft/Game Runner continuam
> pausados por bloqueio externo (`PENDING.md` #1), independente desta reordenação.

## Fase 1 — MVP do Launcher

**Objetivo**: um launcher mínimo mas real — login, instalar e jogar Vanilla, e versionar o
mundo. GUI é o produto principal desde esta fase (não uma fase separada e tardia); CLI existe
em paralelo, opcional. Absorve o que era "Fase 1 — MVP local" do mcgit-ferramenta (v1.0).

- [ ] Login com conta Microsoft (OAuth, sem armazenar senha) — pausado nesta sessão (Sessão 3,
  2026-08-16): tentativa real de ID@Xbox e de app registration no Azure esbarrou numa cadeia de
  pré-requisitos (CNPJ → DUNS number → cadastro de Xbox Partner; e conta Azure exige cartão de
  crédito pra criar um "directory"). Detalhes completos em `PENDING.md` #1. Não bloqueia o resto
  da Fase 1 além do login/lançamento autenticado em si.
- [x] Instalação de Minecraft Vanilla (download de version manifest, libraries, natives) —
  implementado (Sessão 3, 2026-08-16) junto com "Criar instância" abaixo, ver detalhes lá.
- [x] Gerenciamento de Java — Java Manager implementado (Sessão 2, 2026-08-16): detecção de
  instalações no sistema, listagem de versões LTS via API do Adoptium, download+verificação de
  checksum+extração+instalação, seleção de padrão, e adição manual (usuário avançado) — tudo
  testado de ponta a ponta pela GUI real (Tauri+React), com persistência confirmada via SQLite
  (sobrevive a fechar/reabrir o app). Detalhes: `ARCHITECTURE.md` §Java Manager (implementado).
  **Update (Sessão 3, 2026-08-16)**: o item que faltava — detectar automaticamente a versão de
  Java exigida a partir do manifesto de uma instância — está fechado. `create_vanilla_instance`
  lê `javaVersion.majorVersion` do manifesto da Mojang e resolve/instala o Java certo sozinho,
  reaproveitando uma instalação já existente quando possível (validado ao vivo: reaproveitou o
  JDK 25 já instalado na sessão anterior, sem baixar de novo).
- [x] Criar instância (isolada: Minecraft, config, saves, logs próprios) — implementado e
  testado de ponta a ponta pela GUI real (Sessão 3, 2026-08-16). Novos crates `mcgit-minecraft`
  (cliente do `piston-meta`: manifesto, libraries filtradas por SO, assets, download com
  progresso e verificação sha1) e `mcgit-instance` (scaffolding de pastas + `instance.json`),
  mais a tabela `instances` no `mcgit-db` (primeira relação real do projeto via SeaORM,
  `belongs_to` pra `java_installations`). Cache global compartilhado de libraries/assets fora da
  pasta de cada instância, keyed por hash — implementa o design já documentado em
  `ARCHITECTURE.md` §Gerenciamento de Instâncias. Escopo combinado com o usuário: para em
  "instância pronta com o Vanilla baixado e verificado", não inclui abrir o jogo. Detalhes
  completos em `ARCHITECTURE.md` §Instância + Vanilla Install (implementado).
- [ ] Iniciar o Minecraft a partir de uma instância (Game Runner multiplataforma) — próximo item
  natural da Fase 1, mas depende de uma sessão autenticada da Minecraft Services API pra testar
  de ponta a ponta (login MS pausado, ver acima).
- [~] Gerenciar mundos dentro de uma instância (listar, abrir pasta, etc.) — listagem
  implementada como efeito colateral do item de versionamento acima (`list_worlds` lê `saves/*`
  no filesystem, Sessão 4, 2026-08-22); "abrir pasta" e outras ações de gerenciamento ainda não
- [x] Ativar versionamento Git num mundo (opt-in, não forçado) — comandos internos equivalentes
  a `mcgit init`, implementado (Sessão 4, 2026-08-22). Nasce a crate `mcgit-core` (Git Engine),
  chamando o binário `git` via subprocess (decisão da Fase 0, primeira vez exercitada com código
  real). Nova tabela `worlds` em `mcgit-db` (`git_enabled` por mundo, primeira FK obrigatória do
  projeto — `ON DELETE CASCADE` pra `instances`, testado de verdade). Dois comandos Tauri novos
  (`enable_world_versioning`/`disable_world_versioning`) e botão por mundo na tela de detalhe da
  instância. "Desativar" nunca apaga o `.git` nem histórico — só esconde a ação, reversível a
  qualquer momento. Detalhes completos em `ARCHITECTURE.md` §Git Engine.
- [x] Criar versão/snapshot (equivalente a `mcgit snapshot`) — UI diz "Salvar versão", não
  "commit", implementado (Sessão 4, 2026-08-22, continuação). `mcgit-core` ganha `commit()`
  (`git add -A` + `git commit` + `git rev-parse HEAD`), com identidade Git fixa `mcgit
  <mcgit@localhost>` configurada `--local` por mundo (funciona mesmo numa máquina que nunca usou
  Git). "Nada mudou desde o último snapshot" é tratado como resultado normal
  (`CommitOutcome::NothingToCommit`), não erro. Comando Tauri `create_world_snapshot` + botão
  "Save snapshot" com campo de mensagem opcional (mensagem padrão por timestamp se vazio) na
  tela de detalhe da instância. Verificado ao vivo pela GUI real, incluindo o cenário sem
  identidade Git global nenhuma. Detalhes completos em `ARCHITECTURE.md` §Git Engine.
- [x] Ver histórico de versões (equivalente a `mcgit snapshots`) — timeline amigável, não
  `git log` cru, implementado (Sessão 5, 2026-08-22). `mcgit-core` ganha `log()`: roda
  `git log --pretty=format:%H\x1f%aI\x1f%s` (campos separados por `\x1f`, um commit por
  linha) e parseia em `Vec<Snapshot { hash, date, message }>`, mais recente primeiro. Dois
  casos tratados como histórico vazio, não erro: mundo nunca versionado (`!is_repository`)
  e mundo versionado sem nenhum snapshot ainda (git recusa `git log` num repo sem commits;
  esse stderr específico vira `Ok(vec![])`). Comando Tauri `list_world_history` (mesmo
  padrão dos outros: `spawn_blocking`, path resolvido, erro como `String`); `SnapshotDto`
  antigo (resultado de salvar snapshot) renomeado pra `SnapshotResultDto` pra não colidir
  com o novo `SnapshotDto` do histórico (`hash`/`date`/`message`). Botão "Show history" por
  mundo (só quando `git_enabled`), carregado sob demanda no primeiro clique — decisão de UX
  confirmada com o usuário (botão sob demanda, não sempre visível; sem limite de
  quantidade por ora). Nenhuma dependência nova (`chrono`/`time`): datas ficam como string
  ISO 8601 crua, formatadas no frontend com `toLocaleString()`, mesmo padrão já usado.
  **Bug encontrado e corrigido durante a verificação ao vivo**: salvar um snapshot com o
  painel de histórico já aberto deixava a lista desatualizada até fechar/reabrir —
  `handleSaveSnapshot` agora recarrega o histórico daquele mundo automaticamente quando um
  snapshot novo é criado e o histórico já tinha sido carregado antes. Verificado ao vivo
  pela GUI real (mundo de teste criado/removido na pasta `saves/` da instância existente):
  histórico vazio, 1 snapshot, ordem correta com 3 snapshots, atualização automática após
  salvar, e ausência do botão num mundo nunca versionado. Detalhes completos em
  `ARCHITECTURE.md` §Git Engine.
- [x] Restaurar uma versão (equivalente a `mcgit restore`) — checagem de mundo aberto,
  checkpoint de segurança antes de restaurar, implementado (Sessão 6, 2026-08-22). Fecha os
  dois requisitos de segurança já documentados no `CONTEXT.md`: checagem de mundo aberto
  (implementada de verdade, não adiada — `mcgit-core::git::restore()` tenta adquirir o mesmo
  lock exclusivo de `session.lock` que o próprio Minecraft usa, via `std::fs::File::try_lock`
  nativo do Rust, sem dependência nova) e checkpoint de segurança não-negociável (sempre salva
  o estado atual antes de restaurar, reaproveitando `commit()`). Nunca é destrutivo: restaurar
  é `git checkout <hash> -- .` seguido de um novo commit "Restored to `<hash>`" — nunca um
  `reset --hard`, então nenhum commit é perdido e o próprio restore é sempre desfazível
  restaurando de novo. Comando Tauri `restore_world_version`; confirmação inline por
  snapshot ("Restore" → aviso + "Cancel"/"Create Backup and Restore", inspirado no mockup do
  `CONTEXT.md`, sem introduzir modal). **Achado e corrigido durante a verificação ao vivo**: o
  próprio `session.lock` (usado pela checagem de mundo aberto) estava sendo versionado junto
  pelo `git add -A` do `commit()` — corrigido fazendo `git init` escrever
  `session.lock` em `.git/info/exclude` (mecanismo nativo do Git pra exclusões só-locais, que
  nunca precisa virar commit — ao contrário de um `.gitignore` rastreado, não cria uma entrada
  falsa na timeline do jogador). Verificado ao vivo pela GUI real. Detalhes completos em
  `ARCHITECTURE.md` §Git Engine.
- [x] Deletar uma versão (nunca silencioso, sempre com confirmação), implementado (Sessão 7,
  2026-08-22). Fecha o ciclo básico do Git Engine (ativar → snapshot → histórico → restaurar →
  **deletar**) e é a primeira operação de verdade destrutiva da Fase 1. Nunca usa
  `git rebase`/`filter-branch` (risco real de conflito em arquivos binários de mundo, o mesmo
  que o `PHASE.md` já registra como não resolvido na Fase 6 — Branching): `delete_snapshot()`
  religa os ponteiros de pai dos commits sobreviventes direto com `git commit-tree`,
  reaproveitando a árvore de arquivos exata que cada um já tinha (um commit já é uma foto
  completa, não um diff) — nunca pede ao Git pra reconciliar nada, então nunca há conflito
  possível, mesmo em binário. Datas originais preservadas via `GIT_AUTHOR_DATE`/
  `GIT_COMMITTER_DATE`, então snapshots sobreviventes não mudam de data na timeline só porque
  um outro foi deletado. Cobre os 4 casos possíveis (commit do meio, topo/mais recente, raiz
  com descendentes, único commit existente) — os 4 validados manualmente num repositório Git
  descartável **antes** de virar código, e depois cobertos por testes reais. Mesma checagem de
  mundo aberto (`is_currently_open`) do `restore()`. Comando Tauri `delete_world_snapshot`;
  confirmação inline por snapshot (nunca modal), com **dois avisos diferentes** confirmados
  com o usuário: deletar o mais recente é permitido mas avisa que também reseta os arquivos do
  mundo (e um terceiro texto, achado durante a verificação, pro caso de ser o único snapshot
  existente, já que aí não há "estado anterior" nenhum pra voltar). Verificado ao vivo pela GUI
  real. Detalhes completos em `ARCHITECTURE.md` §Git Engine.
- [x] GUI básica: tela inicial com lista de instâncias + botão "Jogar" (mockup em `CONTEXT.md`),
  implementado (Sessão 7, 2026-08-22, continuação). `InstanceList.tsx` virou uma lista de
  cards (nome, versão do MC, botão "Jogar") em vez da lista de links crua de antes — mais
  fiel ao espírito do mockup (que mostra uma única instância em destaque), mas funcionando bem
  com várias instâncias reais, já que o card se repete por instância em vez de assumir só uma.
  Botão "Jogar" existe na tela mas fica desabilitado com "Available after Microsoft login" —
  decisão confirmada com o usuário: aparece (bate com o mockup, não fica escondido) em vez de
  sumir, já que o Game Runner depende do login Microsoft ainda pausado (`PENDING.md` #1); texto
  reflete `status` da instância (`installing`/`failed` mostram texto próprio no botão, não o
  aviso de login). Nenhuma dependência nova, nenhuma migration — mudança só de UI.
- [x] Modo Básico/Avançado: avançado expõe Git (commits/branches/remotes/diff) — básico não,
  implementado (Sessão 7, 2026-08-22, continuação) no escopo hoje possível: branches, remotes e
  diff ainda não existem como features (ficam pra quando forem implementadas, Fase 6+); o único
  detalhe de Git já escondido por baixo dos panos hoje é o hash completo do commit (truncado
  pra 7 chars) e a identidade fixa do autor (`mcgit <mcgit@localhost>`), já antecipada numa nota
  da Sessão 4. Toggle global "Advanced mode" na barra de navegação, persistido em
  `localStorage` (decisão deliberada: preferência de UI, não dado de jogo — não abre migration
  nova no banco). Em Modo Avançado, `WorldHistory.tsx` mostra o hash completo (40 chars) e a
  linha de autor; em Modo Básico, continua igual a antes (hash curto, sem autor). Zero mudança
  em Rust/DB — feature 100% frontend. Verificado ao vivo pela GUI real, incluindo persistência
  real entre fechar/reabrir o app.
- [ ] CLI em paralelo (opcional): `mcgit init/snapshot/snapshots/restore/delete`, `mcgit create/launch`
- [ ] Validações de segurança básicas (ver `CONTEXT.md` §Security Requirements)
- [ ] Testes de restauração repetidos, incluindo cenários de interrupção/corrupção

**Critério de saída da Fase 1**: um jogador consegue instalar o launcher, logar com a conta
Microsoft, criar uma instância Vanilla, jogar, versionar o mundo e confiar que consegue voltar
a qualquer snapshot sem perder dados — tudo pela GUI, sem precisar saber que existe Git.

---

## Fase 2 — Qualidade do Versionamento

Antiga "Fase 2 — Qualidade" do mcgit-ferramenta (v1.0), posicionada logo após o MVP pra
garantir que a base de versionamento é sólida antes de expandir pra mods/modpacks.

- [ ] Snapshots automáticos (gatilhos: ao iniciar/encerrar o Minecraft, ou a cada X minutos — configurável, com mensagem amigável: "Minecraft iniciado", "Minecraft encerrado", "Checkpoint automático")
- [ ] Compactação automática do repositório (`git gc`/repack periódico) — decisão da Fase 0 aplicada
- [ ] Melhor tratamento de mundos grandes (dezenas/centenas de GB)
- [ ] Status mais completo (estado atual vs último snapshot)
- [ ] Sistema de configuração (por instância e por mundo)
- [ ] Logs
- [ ] Testes de corrupção/interrupção mais extensivos

---

## Fase 3 — Modloaders, Mods & Modpacks

- [ ] Fabric — detecção e instalação automática
- [ ] Forge — detecção e instalação automática
- [ ] NeoForge — detecção e instalação automática
- [ ] Busca e instalação de mods individuais (Modrinth primeiro — ver revisão legal da Fase 0)
- [ ] Detecção de incompatibilidade (versão do jogo, modloader, dependências ausentes, mods duplicados)
- [ ] Ativar/desativar mods sem apagar arquivos
- [ ] Busca e instalação de modpacks (Modrinth; CurseForge condicionado à revisão de ToS)
- [ ] Atualização/downgrade de modpacks, resolução de dependências
- [ ] Resource packs (ativar/desativar, por instância)
- [ ] Shaders (ativar/desativar, por instância)

---

## Fase 4 — Minecraft-Aware World Diffing

Antiga "Fase 3 — Minecraft-aware" do mcgit-ferramenta (v1.0).

- [~] Diff específico de regiões/blocos/entidades/estruturas entre dois snapshots — primeira
  fatia implementada (Sessão 8, quinta continuação, 2026-09-01): **diff por chunk** entre duas
  branches, escolhida com o usuário por atacar direto o problema de granularidade achado na
  investigação de merge da Fase 6. Novo crate `mcgit-world` (pure lib, zero conhecimento de
  Git), usando `fastanvil` 0.32 (já validado na prática pela Fase 0) — pra um arquivo de região
  `.mca` marcado como "modified" na comparação da Fase 6, mostra quais chunks (16×16 blocos, em
  coordenadas absolutas do mundo) especificamente mudaram, por comparação de bytes do NBT
  descomprimido de cada chunk — sem decodificar block-states ainda (isso é o próximo passo,
  "Parser NBT completo" abaixo). Ainda não cobre entidades/estruturas nem as dimensões
  Nether/End (só a pasta `region/` principal). Verificado ao vivo pela GUI real (tela livre):
  duas branches com um arquivo de região sintético editado num chunk só, comparação mostrando
  corretamente `(0, 0) changed` e nada mais. Detalhes completos em `ARCHITECTURE.md` §Fase 4.
- [x] Parser NBT completo (block-states) — implementado (Sessão 9, 2026-09-02): decodifica o
  `block_states` bit-packed (palette + índices) de cada seção de um chunk, incluindo o caso
  especial de palette de 1 tipo só (sem `data`), e diz exatamente qual bloco (posição absoluta +
  `Name`+`Properties` de/para) mudou entre duas branches — não só "esse chunk mudou". Entidades
  ainda não cobertas (só block-states). Novo módulo `mcgit-world::chunk`, comando Tauri
  `diff_world_chunk_blocks`, botão "Show blocks" na UI (dentro do "Show chunks" já existente).
  Verificado ao vivo pela GUI real, num mundo real com um bloco editado numa branch. Detalhes
  completos em `ARCHITECTURE.md` §Fase 4.
- [x] Estatísticas de mundo por snapshot (blocos, entidades, estruturas) — blocos implementados
  (Sessão 9, 2026-09-02): conta cada tipo de bloco (por `Name`, ignorando `Properties` — decisão
  deliberada, diferente do diff, porque "quantos `stone`" não deveria separar variante nenhuma)
  em todo o `region/` de um snapshot, somando entre todos os arquivos de região, ordenado do mais
  comum pro menos comum. Reaproveita o mesmo decoder do item anterior, mas contando em vez de
  comparando — sem materializar os 4096 blocos por seção, só os índices decodificados são
  somados por posição de palette. Novo comando Tauri `world_block_stats`, botão "Show stats" por
  snapshot na tela de histórico (`WorldHistory`, disponível também no Modo Básico, não só
  Avançado). Verificado ao vivo pela GUI real contra um mundo real (`medieval`).
  **Entidades e estruturas fechados na mesma sessão (continuação, 2026-09-02)**: entidades
  contadas por `id` (mobs, itens no chão, ...) somando todos os arquivos de `entities/` — pasta
  separada de `region/` desde a 1.17, formato Anvil igual mas raiz do chunk diferente (`Entities`
  em vez de `sections`). Estruturas contadas por tipo somando a chave `structures.starts` de
  cada chunk em `region/` — cada estrutura gerada aparece como "start" só no chunk onde começou
  (os outros que ela atravessa só têm uma referência de volta), então somar starts já dá a
  contagem certa sem duplicar. Novos comandos Tauri `world_entity_stats`/`world_structure_stats`,
  painel "Show stats" agora com três seções (Blocks/Structures/Entities). **Bug real achado e
  corrigido na verificação ao vivo**: um arquivo `.mca` de 0 bytes (placeholder real que o
  próprio Minecraft escreve pra uma região sem nada gerado ainda — visto de verdade em
  `entities/`/`poi/` do mundo `medieval`) não tem cabeçalho Anvil válido, e `world_entity_stats`
  quebrava com `UnexpectedEof` ao tentar abri-lo — corrigido tratando bytes vazios como "sem
  entradas" nas três funções de contagem (`count_region_blocks/_structures/_entities`), não como
  erro. Verificado ao vivo pela GUI real contra o mundo `medieval` de novo, agora as três seções
  populadas com dados reais (18 mineshafts, 5 trial chambers, 110 sheep, etc.). Detalhes
  completos em `ARCHITECTURE.md` §Fase 4.
- [ ] Visualização de alterações entre snapshots (na GUI) — já existe uma versão mínima (lista
  textual de chunks alterados na tela de comparação de branches); uma visualização de verdade
  (mapa, destaque visual) ainda não foi feita.

---

## Fase 5 — Skins, Backup Inteligente & Sincronização

- [ ] Gerenciamento de skins (visualizar atual, trocar, importar, gerenciar múltiplas) — via API oficial
- [ ] Backup inteligente: toggle Local / Cloud / Arweave (checkboxes independentes, ver `CONTEXT.md` §Backup Targets)
- [ ] Compressão e deduplicação de backups
- [ ] Snapshots incrementais
- [ ] Criptografia local antes de qualquer upload (cloud ou Arweave)
- [ ] Verificação de integridade de backups
- [ ] Sincronização entre dispositivos (metadados via banco local + Git remoto/cloud)
- [ ] Metadados de reprodutibilidade por instância/mundo (versão do MC, loader, Java, mods, resource packs, shaders — ver `CONTEXT.md` §Environment Metadata)

---

## Fase 6 — Branching de Mundos

Antiga "Fase 4 — Branching" do mcgit-ferramenta (v1.0).

```bash
mcgit branch experiment
mcgit checkout experiment
mcgit checkout main
```

- [x] Criar/trocar de branch, implementado (Sessão 8, 2026-09-01), primeiro item da trilha
  priorizada de versionamento+branches (ver nota de reordenação no topo deste arquivo). `git.rs`
  ganha `current_branch()` (`git branch --show-current`, funciona mesmo sem nenhum commit ainda,
  já que só lê o ref simbólico do HEAD), `list_branches()` (`git branch --format`),
  `create_branch()` (`git checkout -b <nome>` — cria e já troca num único passo atômico; não
  precisa de checkpoint de segurança nem de checagem de mundo aberto, porque a nova branch
  aponta pro mesmo commit atual, então nenhum arquivo muda de conteúdo) e `switch_branch()`
  (troca pra uma branch já existente). Novo erro `BranchError` (mesmo formato de
  `RestoreError`/`DeleteError`). Nenhuma migration nova no `mcgit-db` — a branch atual é sempre
  derivada ao vivo via `git branch --show-current`, mesma filosofia já usada por `log()`.
  **Dois pontos de design confirmados com o usuário via `AskUserQuestion`**: (1) trocar de
  branch faz um checkpoint automático das mudanças pendentes antes (reaproveita `commit()`,
  mesmo padrão do backup do `restore()`) — nunca falha por "local changes would be
  overwritten", nunca perde nada; (2) a seção de branches na GUI fica visível só em Modo
  Avançado (`WorldList.tsx` usa `useAdvancedMode()` pra esconder a seção inteira, diferente do
  histórico de snapshots, que fica sempre visível) — confirma o que o `ARCHITECTURE.md` já
  antecipava desde a Sessão 7. Novos comandos Tauri `list_world_branches`/`create_world_branch`/
  `switch_world_branch`; novo componente `WorldBranches.tsx` (espelha `WorldHistory.tsx`: dumb/
  presentational, confirmação inline sem modal antes de trocar de branch, já que trocar muda
  visivelmente os arquivos do mundo pro jogador, mesma razão do `restore`/`delete`). Trocar de
  branch também re-busca o histórico de snapshots se o painel já estava aberto (`git log` segue
  o HEAD atual, então muda de conteúdo quando a branch muda — mesma classe de bug de painel
  desatualizado já corrigida na Sessão 5, agora prevenida desde o início). 6 testes novos em
  `git.rs` cobrindo criação, troca com/sem mudança pendente, bloqueio por mundo aberto e listagem
  — os 28 testes de `git.rs` passam. **Verificação ao vivo pela GUI não foi feita nesta sessão**:
  a tela de desenvolvimento tinha uma partida de xadrez ativa roubando o foco da janela
  repetidamente, tornando a interação pela GUI real pouco confiável; o usuário confirmou (via
  `AskUserQuestion`) aceitar a cobertura de testes automatizados como verificação desta vez, sem
  a checagem visual ao vivo que as sessões anteriores sempre fizeram. Vale rodar essa checagem
  manualmente numa sessão futura antes de considerar o item 100% fechado no mesmo padrão de
  confiança dos itens anteriores.
- [x] Comparação entre branches, implementado (Sessão 8, continuação, 2026-09-01). Escopo
  confirmado com o usuário via `AskUserQuestion`: só compara a branch atual com outra branch
  (não snapshots do histórico) e mostra uma lista de arquivos alterados com tamanho em bytes
  antes/depois — nenhum diff de conteúdo, já que a maioria dos arquivos de mundo é binária (isso
  é trabalho da Fase 4, que interpreta o formato de verdade). `git.rs` ganha `diff_branches()`
  (`git diff --name-status` pra saber que arquivos mudaram + `A`/`M`/`D`, mais uma chamada de
  `git cat-file -s <ref>:<path>` por arquivo pra pegar o tamanho em bytes de cada lado — evita
  parsear o `Bin X -> Y bytes` de texto humano do `git diff --stat`). Sem detecção de rename
  (não ativada por padrão no Git). Novo comando Tauri `diff_world_branches` (compara a branch
  atual contra uma branch informada). Novo botão "Compare" por branch não-atual em
  `WorldBranches.tsx`, expandindo inline a lista de arquivos (sem modal). 4 testes novos em
  `git.rs` (32/32 verdes no crate); frontend compila limpo. **Dois bugs reais de painel
  desatualizado encontrados e corrigidos durante a verificação ao vivo** (mesma classe do bug já
  corrigido na Sessão 5 pro histórico): o painel de branches e o de comparação não se atualizavam
  sozinhos depois de um novo snapshot enquanto já estavam abertos — corrigido fazendo
  `handleSaveSnapshot` re-buscar os dois se já estavam carregados; e o painel de comparação
  ficava mostrando uma comparação obsoleta depois de trocar de branch (já que a "branch atual" da
  comparação tinha mudado) — corrigido limpando a comparação aberta em `handleCreateBranch`/
  `handleSwitchBranch`. **Verificado ao vivo pela GUI real** (tela livre desta vez): branch
  "experiment" criada e trocada, snapshot com conteúdo diferente salvo nela, comparação com
  "main" mostrando corretamente `modified — level.dat — 35 bytes → 19 bytes` e
  `deleted — r.0.0.mca — deleted, was 31 bytes`; painel de branches e de comparação atualizando
  sozinhos após snapshot novo; comparação limpa corretamente depois de trocar de branch de volta
  pra "main". Detalhes completos em `ARCHITECTURE.md` §Git Engine.
- [x] Investigar se/como merge faz sentido tecnicamente para arquivos de mundo, feito (Sessão 8,
  continuação, 2026-09-01) — 5 experimentos reais num repositório Git descartável (mesmo método
  usado antes do `delete_snapshot`), não só análise teórica, mais um sexto experimento adicional
  feito depois de o usuário questionar diretamente a conclusão inicial. **Resultado: merge
  tradicional do Git nunca corrompe nem perde dado silenciosamente (desde que a UI nunca tente
  resolver conflito de conteúdo sozinha — o próprio Git já se recusa a fazer isso e nunca injeta
  marcadores dentro de um arquivo binário), mas a granularidade do conflito é o arquivo inteiro
  (uma região `.mca` = 512×512 blocos), não o chunk/bloco que realmente mudou** — confirmado com
  duas mudanças sem NENHUMA sobreposição real, em pontas opostas do mesmo arquivo simulando uma
  região: ainda assim conflito no arquivo inteiro, forçando escolher a região inteira de uma
  branch ou da outra e descartando a outra por completo. Isso significa que cenários realistas
  (casa construída na main, mob quebra um bloco em outro canto da mesma região na branch) forçam
  uma escolha tudo-ou-nada mesmo sem sobreposição de intenção nenhuma — resolver isso de verdade
  exige entender o formato Anvil/NBT por dentro, trabalho da Fase 4 (Minecraft-Aware World
  Diffing), não algo que a Fase 6 (Git puro) resolve sozinha.
  Confirmado o contraste com arquivo texto puro (marcadores `<<<<<<<` escritos literalmente
  dentro do arquivo, ao contrário do binário); arquivos diferentes alterados em cada branch, ou
  mesmo arquivo com resultado idêntico nos dois lados, ainda fazem merge automático limpo;
  conflito de conteúdo e de modify/delete são detectados com clareza, o arquivo original fica
  intacto no disco, as duas versões continuam recuperáveis via `git ls-files -u`, e
  `git merge --abort` desfaz tudo de forma limpa. `git merge-tree --write-tree` (Git ≥ 2.38)
  permite pré-visualizar se um merge vai conflitar sem tocar a árvore de trabalho — útil pra um
  futuro "essas N regiões vão conflitar, escolher uma versão inteira de cada?" antes de qualquer
  coisa mudar de verdade. Desenho de resolução (escolher "esta versão" ou "a outra" por arquivo
  inteiro, via `checkout --ours`/`--theirs` + `add`/`rm` + `commit`, com aviso explícito de
  granularidade na UI) virou código de verdade ainda na mesma sessão — ver logo abaixo.
- [x] **Merge entre branches**, implementado (Sessão 8, continuação, 2026-09-01), fechando o
  ciclo completo do Git Engine puro. `git.rs` ganha `preview_merge`/`list_merge_conflicts`/
  `merge_branch`/`resolve_conflict`/`finish_merge`/`abort_merge`, com guardas de mundo aberto e
  de merge já em andamento (`MergeError`). Botão "Merge" por branch na UI, com preview mostrando
  a lista real de arquivos que vão conflitar e o aviso de granularidade por extenso antes de
  qualquer coisa mudar de verdade; resolução por arquivo inteiro ("Keep this branch's version"/
  "Keep the other branch's version"), "Abort merge" sempre disponível. 9 testes novos (41/41 no
  crate). **Verificado ao vivo pela GUI real**: conflito real criado e resolvido com sucesso
  (commit de merge de 2 pais confirmado no `git log`), e um segundo conflito criado e abortado
  (estado restaurado exatamente ao de antes, conferido no disco). Detalhes completos em
  `ARCHITECTURE.md` §Git Engine (subseções "Investigação: merge entre branches" e "Merge entre
  branches — implementado").

**Fase 6 completa**: os 3 itens (criar/trocar branch, comparar, merge) estão implementados e
verificados ao vivo.

---

## Fase 7 — Arweave + TruthID

Antiga "Fase 5 — Arweave + TruthID" do mcgit-ferramenta (v1.0), agora explicitamente usando as
abstrações `StorageProvider`/`AuthenticationProvider` definidas na Fase 0.

- [ ] Integração com o TruthID existente (via `AuthenticationProvider`)
- [ ] Preparação de snapshots/objetos para armazenamento remoto
- [ ] Upload para Arweave (via `StorageProvider::ArweaveStorage`)
- [ ] Associação entre snapshot Git e transação Arweave (metadados)
- [ ] Verificação de integridade dos dados remotos
- [ ] `mcgit push arweave` / `mcgit pull arweave` (ou `mcgit backup` / `mcgit restore-remote`)
- [ ] Tratamento de falhas e uploads interrompidos
- [ ] Controle/estimativa de custos de armazenamento, exibido ao usuário antes do upload
- [ ] Proteção contra publicação acidental de dados privados do mundo (permanência do Arweave é irreversível)

---

## Fase 8 — Compartilhamento de Mundos & Reprodutibilidade

- [ ] Gerar um "World ID" compartilhável para um mundo
- [ ] Importar mundo a partir de um World ID (download, verificação de integridade, instalação)
- [ ] Descoberta automática do ambiente necessário (versão do MC, modloader, mods e versões, resource packs, shaders, configs) usando os metadados de reprodutibilidade da Fase 5
- [ ] Reconstrução automática do ambiente ao importar um mundo modded

---

## Fase 9 — Servidores

Antiga "Fase 6 — Servidores" do mcgit-ferramenta (v1.0).

- [ ] Integração com servidores Minecraft (dedicated server, gerenciado localmente/self-hosted — mcgit não hospeda servidores de terceiros, ver `CONTEXT.md` §Non Goals)
- [ ] Snapshots automáticos (ex.: antes de restart)
- [ ] Rollback rápido
- [ ] Hooks (ex.: pre-restart, post-crash)
- [ ] Backup remoto automático

---

## Fase 10 — Colaboração, Marketplace & Social

Antiga "Fase 5" do prompt do launcher (a mais especulativa/distante de todas).

- [ ] Branches colaborativas entre jogadores
- [ ] Compartilhamento avançado (permissões, acesso a backups de outros jogadores)
- [ ] Marketplace/ecossistema — avaliar se faz sentido e como, sem contradizer o modelo open source (ver `ROADMAP.md` §Monetização)
- [ ] Recursos sociais, se fizer sentido (avaliar caso a caso, não é objetivo em si)
