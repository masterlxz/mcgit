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
- [ ] Deletar uma versão (nunca silencioso, sempre com confirmação)
- [ ] GUI básica: tela inicial com lista de instâncias + botão "Jogar" (mockup em `CONTEXT.md`)
- [ ] Modo Básico/Avançado: avançado expõe Git (commits/branches/remotes/diff) — básico não
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

- [ ] Parser NBT completo (reaproveitar aprendizado da Fase 0)
- [ ] Diff específico de regiões/blocos/entidades/estruturas entre dois snapshots
- [ ] Estatísticas de mundo por snapshot (blocos, entidades, estruturas)
- [ ] Visualização de alterações entre snapshots (na GUI)

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

- [ ] Criar/trocar de branch (na GUI: "Criar branch experimental" + "Voltar para main")
- [ ] Comparação entre branches/versões
- [ ] Investigar se/como merge faz sentido tecnicamente para arquivos de mundo (não assumir que é seguro — ver `ARCHITECTURE.md`)

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
