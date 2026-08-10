## Roadmap de Evoluções Planejadas

Este arquivo cobre o que vem depois/além do que já está detalhado por fase em `PHASE.md`:
ideias de produto ainda sem fase própria, perguntas técnicas em aberto e a visão de
monetização. (Sessão 1, revisão 2: os itens de GUI e de multiplayer/servidores que estavam
aqui viraram fases concretas — Fase 1 e Fase 9 em `PHASE.md` — e saíram deste arquivo.)

---

## Ideias de produto sem fase própria ainda

- **Remote Git tradicional**: manter `mcgit push` para um remote Git convencional como opção,
  em paralelo ao armazenamento descentralizado via TruthID + Arweave (que deve ser
  tratado como opção de primeira classe da arquitetura, não um extra de segunda categoria).
- **Storage self-hosted**: terceira opção de remote além de Git tradicional e Arweave.

---

## Monetização

Decisão do usuário (2026-08-10): **open source puro** no mesmo espírito do protocolo TruthID
— o core do mcgit (launcher, versionamento local, CLI) é e continua sendo livre e aberto.

Ao mesmo tempo, a intenção declarada é eventualmente construir um negócio em cima da
ferramenta, **sem que isso mude a mentalidade open source do projeto**. Ou seja: o negócio
deve viver ao redor do core aberto (hosting gerenciado, serviço de backup remoto operado pelo
projeto, suporte a operadores de servidor, eventual marketplace da Fase 10, etc.), não
substituí-lo por uma versão fechada ou freemium do próprio launcher/versionamento. Nenhum plano
concreto definido ainda — revisar esta seção quando as fases de Arweave/TruthID (Fase 7) e
Colaboração/Marketplace (Fase 10) estiverem mais maduras.

---

## Perguntas técnicas em aberto

Perguntas que não têm resposta ainda e devem ser decididas com dados, não por preferência.

### Versionamento de mundo (herdadas do documento de ideação original, v1.0)

- Git puro é suficiente ou precisamos de Git LFS / camada própria?
- Devemos versionar `.mca` diretamente ou dividir/interpretar regiões antes?
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

### Launcher (novas, do escopo expandido — Sessão 1, revisão 2)

- CurseForge vale o atrito de licenciamento cedo, ou começar só com Modrinth e avaliar depois?
- Qual estratégia de gerenciamento de Java generaliza melhor entre Linux/Windows/macOS?
- Instâncias devem compartilhar bibliotecas/assets comuns (economia de espaço) ou ficar 100% isoladas (simplicidade/segurança)? Prism e afins já resolveram isso de algum jeito — vale estudar antes de decidir.
- Sincronização entre dispositivos (Fase 5) usa o quê como transporte — Git remoto, um serviço próprio, ou delega pro StorageProvider de cloud?
- Reprodutibilidade de ambiente (Fase 8) é sempre possível, ou só numa boa parte dos casos (ex.: mod removido da plataforma de origem)? Como comunicar isso ao usuário quando falhar?
- Marketplace (Fase 10) — existe uma forma de fazer isso sem contradizer o princípio de open source puro? (ver `GUIDELINES.md` princípio 16)
