## Roadmap de Evoluções Planejadas

Este arquivo cobre o que vem depois/além do que já está detalhado por fase em `PHASE.md`:
ideias de produto, perguntas técnicas em aberto e a visão de monetização.

---

## Ideias de produto (além do MVP)

- **Multiplayer/servidores** (Fase 6): pode ser mais valioso comercialmente do que apenas
  versionamento single-player — snapshots automáticos, rollback rápido, backup remoto
  automático para operadores de servidor.
- **Remote Git tradicional**: manter `mcgit push` para um remote Git convencional como opção,
  em paralelo ao armazenamento descentralizado via TruthID + Arweave (que deve ser
  tratado como opção de primeira classe da arquitetura, não um extra de segunda categoria).
- **Storage self-hosted**: terceira opção de remote além de Git tradicional e Arweave.
- **GUI/TUI** (Fase 7): interface visual de histórico, comparação e restauração. Mockup inicial
  (do brainstorm original, Sessão 1):

  ```text
  Minecraft World
  ────────────────────────────────

  Snapshots

  ● Hoje 18:30
    Construí a cidade

  ○ Hoje 15:10
    Antes da cidade

  ○ Ontem 21:43
    Comecei a farm

  [ Restore ] [ Compare ] [ Branch ]
  ```

---

## Monetização

Decisão do usuário (2026-08-10): **open source puro** no mesmo espírito do protocolo TruthID
— o core do mcgit (versionamento local, CLI) é e continua sendo livre e aberto.

Ao mesmo tempo, a intenção declarada é eventualmente construir um negócio em cima da
ferramenta, **sem que isso mude a mentalidade open source do projeto**. Ou seja: o negócio
deve viver ao redor do core aberto (hosting gerenciado, serviço de backup remoto operado pelo
projeto, suporte a operadores de servidor, etc.), não substituí-lo por uma versão fechada ou
freemium do próprio versionamento. Nenhum plano concreto definido ainda — revisar esta seção
quando a Fase 5/6 estiver mais madura.

---

## Perguntas técnicas em aberto (herdadas do documento de ideação)

Perguntas que não têm resposta ainda e devem ser decididas com dados, não por preferência:

- Git puro é suficiente ou precisamos de Git LFS / camada própria?
- Devemos versionar `.mca` diretamente ou dividir/interpretar regiões antes?
- Rust ou Python? (ver `ARCHITECTURE.md`)
- CLI pura ou TUI desde cedo?
- Como detectar de forma confiável se o mundo está aberto no Minecraft?
- Como garantir que a restauração seja atômica (sem estado parcial em caso de falha)?
- Como lidar com mundos de dezenas/centenas de GB?
- Snapshots automáticos devem ser o padrão, ou opt-in?
- Branches realmente precisam de merge, ou branch descartável já basta?
- Como lidar com servidores em execução durante uma operação do mcgit?
- Qual é a unidade ideal de armazenamento no Arweave (snapshot completo vs deltas vs regiões)?
- Como aproveitar deduplicação entre snapshots?
- Como mapear um commit Git para uma transação Arweave de forma auditável?
- Quais metadados mínimos são necessários para recuperar um mundo em outro computador?
- Como o TruthID deve expor a integração ao mcgit (SDK existente? novo módulo?)?
- Como estimar e comunicar custos de armazenamento ao usuário antes de um upload?
- Como lidar com uploads interrompidos para o Arweave?
- Como verificar a integridade de um snapshot remoto antes/depois de restaurar?
- Como evitar que dados privados do mundo sejam publicados acidentalmente no Arweave (que é permanente)?
