## Fases Detalhadas

Convenção: `[ ]` não iniciado, `[~]` em andamento, `[x]` concluído. Atualizar aqui e no
checklist resumido de `OVERVIEW.md` ao final de cada sessão.

> **Reconciliação (Sessão 1, revisão 2)**: este arquivo cobria só o mcgit-ferramenta-de-versionamento
> (Fase 0-7). O escopo virou um launcher completo (ver `CONTEXT.md` v2.0). Nada do que já foi
> pesquisado/decidido se perdeu — só foi reencaixado nas fases novas abaixo. Onde uma fase
> corresponde diretamente a uma fase antiga, isso está anotado.

---

## Fase 0 — Pesquisa & Arquitetura

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

### Falta (versionamento de mundo — continuação)

- [ ] Avaliar Git LFS para `.mca` e comparar tamanho/performance contra Git puro
- [ ] Testar edição de bloco real (não só metadado como `InhabitedTime`) — mutar block-states de verdade
- [ ] Repetir o benchmark com mundo maior e mais sessões (10s-100s), múltiplas regiões por sessão
- [ ] Revalidar a suposição de "bytes ficam estáveis entre saves" contra um mundo salvo pelo servidor Java oficial (não só pelo `fastanvil`)
- [ ] Decidir como o Git Engine vai chamar o Git (binário do sistema vs `git2`/libgit2) — registrar em `ARCHITECTURE.md`
- [ ] Decidir se/como o mcgit dispara `git gc`/repack periodicamente (indício forte a favor, ver benchmark)

### Novo (launcher completo — do escopo expandido)

- [ ] Analisar viabilidade técnica do launcher como um todo
- [ ] Identificar APIs oficiais necessárias (Microsoft/Mojang auth, Minecraft version manifest, Modrinth, CurseForge, skins)
- [ ] Identificar limitações legais/licenciamento (ver `CONTEXT.md` §Legal & Licensing) — **bloqueia código de auth/mods/skins, não bloqueia pesquisa**
- [ ] Analisar como launchers existentes (Prism Launcher, Modrinth App, ATLauncher) implementam instalação e execução do Minecraft
- [ ] Propor/validar a arquitetura de módulos (`ARCHITECTURE.md` §Arquitetura de Módulos) — refinar conforme aprendizado
- [ ] Confirmar/documentar a justificativa da stack (Rust + Tauri + React/TS) — já decidida, documentar o porquê formalmente
- [ ] Definir estrutura de diretórios do workspace Rust (proposta inicial já em `ARCHITECTURE.md`)
- [ ] Definir os módulos e suas responsabilidades em detalhe (interfaces entre crates)
- [ ] Definir o schema do banco local (SQLite tentativo — tabelas: instances, accounts, worlds, mods, modpacks, java_installations, backups, git_repositories, arweave_uploads, skins, settings)
- [ ] Definir APIs/interfaces internas entre módulos (traits Rust, ex.: `StorageProvider`, `AuthenticationProvider`)
- [ ] Definir o fluxo de autenticação Microsoft (OAuth) em detalhe
- [ ] Definir a estratégia de gerenciamento de Java (detecção, download, múltiplas versões simultâneas)
- [ ] Definir a estratégia de gerenciamento de instâncias (isolamento de arquivos, configs, JVM args)
- [ ] Definir a estratégia de gerenciamento de modpacks (Modrinth primeiro? CurseForge depois? resolução de dependências)
- [ ] Definir estratégia de snapshots e backups (local vs cloud vs Arweave, como isso se relaciona com o Git Engine)
- [ ] Esboçar a integração futura com Arweave (sem implementar)
- [ ] Definir como o TruthID poderia se integrar sem criar acoplamento cedo (abstração `AuthenticationProvider`)
- [ ] Definir o modelo de segurança completo (armazenamento de credenciais por SO, criptografia antes de upload)
- [ ] Definir a arquitetura multiplataforma (abstrações de OS/filesystem/Java/processo/credenciais)
- [ ] Confirmar/refinar a definição do MVP (Fase 1, abaixo)
- [ ] Dividir o desenvolvimento em milestones dentro de cada fase abaixo

**Critério de saída da Fase 0**: arquitetura de módulos, schema do banco, fluxo de auth
Microsoft, estratégia de armazenamento de `.mca`, e revisão legal/licenciamento documentados em
`ARCHITECTURE.md`/`CONTEXT.md` com base em pesquisa real, não só preferência.

---

## Fase 1 — MVP do Launcher

**Objetivo**: um launcher mínimo mas real — login, instalar e jogar Vanilla, e versionar o
mundo. GUI é o produto principal desde esta fase (não uma fase separada e tardia); CLI existe
em paralelo, opcional. Absorve o que era "Fase 1 — MVP local" do mcgit-ferramenta (v1.0).

- [ ] Login com conta Microsoft (OAuth, sem armazenar senha)
- [ ] Instalação de Minecraft Vanilla (download de version manifest, libraries, natives)
- [ ] Gerenciamento de Java (detectar versão necessária, instalar automaticamente se ausente)
- [ ] Criar instância (isolada: Minecraft, config, saves, logs próprios)
- [ ] Iniciar o Minecraft a partir de uma instância (Game Runner multiplataforma)
- [ ] Gerenciar mundos dentro de uma instância (listar, abrir pasta, etc.)
- [ ] Ativar versionamento Git num mundo (opt-in, não forçado) — comandos internos equivalentes a `mcgit init`
- [ ] Criar versão/snapshot (equivalente a `mcgit snapshot`) — UI diz "Salvar versão", não "commit"
- [ ] Ver histórico de versões (equivalente a `mcgit snapshots`) — timeline amigável, não `git log` cru
- [ ] Restaurar uma versão (equivalente a `mcgit restore`) — checagem de mundo aberto, checkpoint de segurança antes de restaurar
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
