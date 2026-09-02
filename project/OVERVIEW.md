# O que é o mcgit

Launcher de Minecraft multiplataforma — inspirado em Prism Launcher, Modrinth App e ATLauncher,
mas com um diferencial central que nenhum deles tem: **todo mundo pode ser versionado com Git**,
numa interface que nunca exige o jogador saber que o Git existe. "Um sistema operacional pros
seus mundos de Minecraft": Minecraft + Java + instâncias + mods + modpacks + skins + mundos +
Git + backups + cloud + Arweave, resolvidos por baixo enquanto o jogador só escolhe "o que eu
quero jogar?".

O versionamento de mundos (a ideia original do projeto, v1.0 do PRD) continua sendo o
diferencial central — só deixou de ser a *única* coisa que o mcgit faz. Ver `CONTEXT.md` v2.0
para o PRD completo.

Camada futura (não-MVP): backup remoto descentralizado via **Arweave**, com identidade e
pagamento de taxas intermediados pelo **TruthID** (`../truthid`, projeto já existente e em produção).

Stack principal:
- **Linguagem**: Rust ✓ (decidido, confirmado válido pro launcher inteiro) — ver `ARCHITECTURE.md`
- **Arquitetura**: workspace de crates por módulo (auth, Java, instâncias, mods, storage, etc.) — ver `ARCHITECTURE.md` §Arquitetura de Módulos
- **Motor de versionamento**: Git (via binário do sistema ou `git2`/libgit2 — detalhe a decidir na Fase 0), dentro do módulo Git Engine (era "mcgit-core" da v1.0)
- **Interface**: GUI primeiro ✓ (decisão revisada — um launcher é fundamentalmente gráfico) com Tauri + React/TypeScript, mesma stack do `truthid/desktop`; CLI existe em paralelo, opcional
- **Banco local**: SQLite tentativo, para metadados (instâncias, contas, mods, backups...) — nunca o conteúdo dos mundos em si
- **Backup remoto (futuro)**: TruthID (identidade + pagamento) + Arweave (armazenamento)

---

# Status Geral

```
Fase 0  — Pesquisa & Arquitetura                    [x] Encerrada (3 itens adiados, não-bloqueantes — ver PHASE.md)
Fase 1  — MVP do Launcher                           [~] Em andamento (Java Manager, Instância+Vanilla, Git Engine básico completo, GUI inicial, Modo Básico/Avançado — ver PHASE.md)
Fase 2  — Qualidade do Versionamento                [ ] Priorizada (ver nota de reordenação abaixo)
Fase 3  — Modloaders, Mods & Modpacks               [ ] Não iniciada (adiada — ver nota abaixo)
Fase 4  — Minecraft-Aware World Diffing             [~] Em andamento (diff por chunk, parser NBT completo, estatísticas de blocos — ver PHASE.md)
Fase 5  — Skins, Backup Inteligente & Sincronização [ ] Não iniciada (adiada — ver nota abaixo)
Fase 6  — Branching de Mundos                       [x] Completa (criar/trocar branch, comparação e merge — ver PHASE.md)
Fase 7  — Arweave + TruthID                         [ ] Não iniciada (adiada — ver nota abaixo)
Fase 8  — Compartilhamento & Reprodutibilidade      [ ] Não iniciada
Fase 9  — Servidores                                [ ] Não iniciada
Fase 10 — Colaboração, Marketplace & Social         [ ] Não iniciada
```

**Nota de reordenação (Sessão 8, 2026-09-01)**: a numeração das fases é histórica, não é mais a
ordem de execução. A partir desta sessão, o usuário pediu para focar em aprofundar o
versionamento de mundo (incluindo branches) e a GUI antes de expandir pro resto do launcher.
Ordem de execução real agora: terminar o que resta da Fase 1 relacionado a Git (validações de
segurança, testes de restauração) → **Fase 6 (Branching)**, começando por "criar/trocar branch"
→ Fase 2 (qualidade do versionamento: auto-snapshot, auto-gc) → Fase 4 (diff entre snapshots) →
só depois retomar Fase 3/5/7 (mods, backup/sync, Arweave) na ordem antiga. Ver `PHASE.md` para o
detalhe completo dessa decisão.

Ver `PHASE.md` para o detalhamento de cada fase (reconciliação completa entre o roadmap
original do mcgit-ferramenta e o roadmap do launcher documentada lá).

---

# Modelo do projeto

Open source puro (MIT, mesma licença do TruthID). Sem plano de fechar o core ou mudar a
mentalidade open source. Um negócio pode ser construído em cima da ferramenta no futuro
(ex.: hosting gerenciado, backup remoto pago, suporte para servidores, marketplace/social da
Fase 10), mas o core e o protocolo de versionamento permanecem sempre livres e abertos — ver
`ROADMAP.md` §Monetização.

---

# Checklist antes de qualquer release público

Ainda não aplicável — projeto em fase de pesquisa/arquitetura (Fase 0). Este checklist será
preenchido quando o MVP do launcher (Fase 1) estiver com código real para revisar, seguindo o
mesmo protocolo do TruthID: `/code-review` por área crítica antes de cortar uma versão. Dado o
escopo maior (auth, mods de terceiros, dinheiro real via Arweave mais adiante), esse checklist
provavelmente vai precisar cobrir mais áreas do que cobriria uma ferramenta CLI isolada —
revisar quando chegar lá.
