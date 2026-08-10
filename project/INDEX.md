# mcgit — Estado do Projeto

> Última atualização: 2026-08-10 (Sessão 1 — estrutura de projeto criada; Fase 0 em andamento,
> benchmark de Git com mundo real feito; escopo expandido de "ferramenta de versionamento" pra
> "launcher completo" no PRD v2.0 — ver `CONTEXT.md`)

---

## Como Usar Este Arquivo

O estado do projeto foi dividido em arquivos menores dentro desta pasta (`project/`).
Leia o arquivo relevante para o que você precisa:

| Para saber sobre | Leia |
|---|---|
| Diretrizes de código e ensino | `GUIDELINES.md` |
| Visão geral, stack, status das fases | `OVERVIEW.md` |
| PRD (Product Requirements Document) | `CONTEXT.md` |
| **Todas as fases detalhadas (0 a 10)** | **`PHASE.md`** |
| Decisões de arquitetura (em aberto e tomadas) | `ARCHITECTURE.md` |
| **Pendências (resolvidas e não resolvidas)** | **`PENDING.md`** |
| Roadmap, evoluções planejadas, monetização, backlog | `ROADMAP.md` |
| Log completo de sessões de trabalho | `SESSIONS.md` |

Esta pasta (`project/`) é a única base do projeto — não existe mais um documento de ideação
separado na raiz do repositório. O brainstorm original foi incorporado a estes arquivos na
Sessão 1 e removido em seguida; o que estava nele (visão, diagramas, perguntas técnicas em
aberto) está espalhado por `CONTEXT.md`, `ARCHITECTURE.md`, `PHASE.md` e `ROADMAP.md`.

**Ao começar uma sessão**: Diga ao Claude "leia os arquivos em `project/` e me ajude a continuar"
**Ao terminar uma sessão**: Atualize o Log de Sessões em `SESSIONS.md` e marque etapas concluídas. Se resolveu uma pendência, atualize `PENDING.md`.
**Ao tomar uma decisão técnica**: Registre em `ARCHITECTURE.md`
**Ao encontrar um bug/pendência nova**: Adicione em `PENDING.md` com ID sequencial
**Ao mudar de máquina**: Sincronize via git
