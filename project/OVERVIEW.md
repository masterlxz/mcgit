# O que é o mcgit

Ferramenta de versionamento para mundos de Minecraft, usando **Git como motor por baixo**.
Experiência de uso pretendida: "Git + Time Machine para mundos de Minecraft" — o jogador nunca
precisa digitar um comando Git diretamente.

Camada futura (não-MVP): backup remoto descentralizado via **Arweave**, com identidade e
pagamento de taxas intermediados pelo **TruthID** (`../truthid`, projeto já existente e em produção).

Stack principal:
- **Linguagem**: Rust ✓ (decidido) — ver `ARCHITECTURE.md`
- **Core**: crate `mcgit-core` (lógica de versionamento), consumida pelo binário CLI e, depois, pelo app Tauri
- **Motor de versionamento**: Git (via binário do sistema ou `git2`/libgit2 — detalhe a decidir na Fase 0)
- **Interface**: CLI primeiro (Fase 1); GUI Tauri + React/TypeScript ✓ (decidido, mesma stack do `truthid/desktop`) na Fase 7
- **Backup remoto (futuro)**: TruthID (identidade + pagamento) + Arweave (armazenamento)

---

# Status Geral

```
Fase 0 — Pesquisa                       [~] Em andamento
Fase 1 — MVP local                      [ ] Não iniciada
Fase 2 — Qualidade                      [ ] Não iniciada
Fase 3 — Minecraft-aware                [ ] Não iniciada
Fase 4 — Branching                      [ ] Não iniciada
Fase 5 — Arweave + TruthID              [ ] Não iniciada
Fase 6 — Servidores                     [ ] Não iniciada
Fase 7 — Interface (TUI/GUI)            [ ] Não iniciada
```

Ver `PHASE.md` para o detalhamento de cada fase.

---

# Modelo do projeto

Open source puro (MIT, mesma licença do TruthID). Sem plano de fechar o core ou mudar a
mentalidade open source. Um negócio pode ser construído em cima da ferramenta no futuro
(ex.: hosting gerenciado, backup remoto pago, suporte para servidores), mas o core e o
protocolo de versionamento permanecem sempre livres e abertos — ver `ROADMAP.md` §Monetização.

---

# Checklist antes de qualquer release público

Ainda não aplicável — projeto em fase de ideação/pesquisa (Fase 0). Este checklist será
preenchido quando o MVP local (Fase 1) estiver com código real para revisar, seguindo o mesmo
protocolo do TruthID: `/code-review` por área crítica antes de cortar uma versão.
