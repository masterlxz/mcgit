## Fases Detalhadas

Convenção: `[ ]` não iniciado, `[~]` em andamento, `[x]` concluído. Atualizar aqui e no
checklist resumido de `OVERVIEW.md` ao final de cada sessão.

---

## Fase 0 — Pesquisa

**Objetivo**: entender o domínio (mundos Minecraft) e validar/descartar premissas técnicas
antes de escrever código de produto. Nada aqui deve virar código do MVP diretamente — é
investigação e benchmark.

- [ ] Entender a estrutura de diretórios de um mundo Minecraft (single-player e servidor)
- [ ] Entender o formato `.mca` (Anvil region format)
- [ ] Entender NBT (estrutura, bibliotecas existentes na(s) linguagem(ns) candidata(s))
- [ ] Testar Git puro (`git init && git add && git commit`) com mundos reais de tamanhos variados
- [ ] Medir o tamanho do repositório `.git` resultante após múltiplos snapshots
- [ ] Testar restauração (`git checkout`/`git restore` equivalente) e validar integridade do mundo restaurado
- [ ] Avaliar Git LFS para `.mca` e comparar tamanho/performance contra Git puro
- [ ] Decidir linguagem principal (Rust vs Python) com base nos resultados acima — registrar em `ARCHITECTURE.md`
- [ ] Decidir como o mcgit vai chamar o Git (binário do sistema vs biblioteca) — registrar em `ARCHITECTURE.md`
- [ ] Esboçar como o TruthID será integrado ao mcgit (sem implementar ainda)
- [ ] Esboçar uma estratégia preliminar para Arweave (sem implementar ainda)

**Critério de saída da Fase 0**: decisões de linguagem e de estratégia de armazenamento
registradas em `ARCHITECTURE.md` com dados de benchmark que as sustentem, não só preferência.

---

## Fase 1 — MVP local

**Objetivo**: ferramenta local simples e confiável. Sem cloud, sem contas.

Comandos:

```bash
mcgit init
mcgit snapshot "Antes de construir a cidade"
mcgit snapshots
mcgit restore <snapshot>
mcgit delete <snapshot>
```

- [ ] `mcgit init` — detecta mundo válido, inicializa repo Git, cria config/metadados, evita arquivos desnecessários
- [ ] `mcgit snapshot <mensagem>` — detecta alterações, prepara arquivos, `git add` + `git commit`
- [ ] `mcgit snapshots` — histórico em formato amigável (não `git log` cru)
- [ ] `mcgit restore <snapshot>` — checagem de mundo aberto, aviso de alterações não salvas, backup de segurança antes de restaurar, restauração
- [ ] `mcgit delete <snapshot>` — nunca silencioso, sempre com confirmação
- [ ] `mcgit status` — estado atual do mundo em relação ao último snapshot
- [ ] Validações de segurança básicas (ver `CONTEXT.md` §Security Requirements)
- [ ] Testes de restauração repetidos, incluindo cenários de interrupção/corrupção

**Critério de saída da Fase 1**: um usuário consegue instalar o mcgit, versionar um mundo real
seu, e confiar que consegue voltar a qualquer snapshot sem perder dados.

---

## Fase 2 — Qualidade

- [ ] Snapshots automáticos (gatilho a definir — ex.: ao fechar o Minecraft)
- [ ] Melhor tratamento de mundos grandes (dezenas/centenas de GB)
- [ ] `mcgit status` mais completo
- [ ] Sistema de configuração (arquivo de config do mcgit por mundo)
- [ ] Logs
- [ ] Testes de corrupção/interrupção mais extensivos

---

## Fase 3 — Minecraft-aware

- [ ] Parser NBT
- [ ] Diff específico de regiões/blocos/entidades/estruturas (ver exemplo em `CONTEXT.md`)
- [ ] Estatísticas de mundo (blocos, entidades, estruturas por snapshot)
- [ ] Visualização de alterações entre snapshots

---

## Fase 4 — Branching

```bash
mcgit branch experiment
mcgit checkout experiment
mcgit checkout main
```

- [ ] `mcgit branch <nome>`
- [ ] `mcgit checkout <branch>`
- [ ] Comparação entre branches/versões
- [ ] Investigar se/como merge faz sentido tecnicamente para arquivos de mundo (ver `ARCHITECTURE.md` — não assumir que é seguro)

---

## Fase 5 — Arweave + TruthID

- [ ] Integração com o TruthID existente (autenticação/autorização)
- [ ] Preparação de snapshots/objetos para armazenamento remoto
- [ ] Upload para Arweave
- [ ] Associação entre snapshot Git e transação Arweave (metadados — ver `CONTEXT.md`)
- [ ] Verificação de integridade dos dados remotos
- [ ] `mcgit push arweave` / `mcgit pull arweave` (ou `mcgit backup` / `mcgit restore-remote`)
- [ ] Tratamento de falhas e uploads interrompidos
- [ ] Controle/estimativa de custos de armazenamento
- [ ] Proteção contra publicação acidental de dados privados do mundo

---

## Fase 6 — Servidores

- [ ] Integração com servidores Minecraft (dedicated server)
- [ ] Snapshots automáticos (ex.: antes de restart)
- [ ] Rollback rápido
- [ ] Hooks (ex.: pre-restart, post-crash)
- [ ] Backup remoto automático

---

## Fase 7 — Interface

- [ ] Decidir TUI vs GUI (ou ambos)
- [ ] Histórico visual
- [ ] Comparação visual entre versões
- [ ] Restauração pela interface
- [ ] Gerenciamento de backups remotos pela interface
