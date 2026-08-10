# Minecraft World Version Control — Ideia / Especificação Inicial

## 1. Visão

Criar uma ferramenta de versionamento específica para mundos de Minecraft, usando **Git como motor de versionamento por baixo**.

A ideia não é reinventar o Git. O software deve esconder a complexidade do Git e oferecer uma experiência parecida com:

> **Git + Time Machine para mundos de Minecraft.**

O usuário trabalha com o mundo normalmente e cria snapshots/versões por meio da ferramenta.

Além do armazenamento local, o projeto terá uma camada futura de **backup remoto descentralizado usando Arweave**, integrada ao **TruthID** já existente. O TruthID será responsável pela identidade/autorização e pelo mecanismo já existente de pagamento das taxas necessárias para o armazenamento em Arweave.

---

## 2. Problema

Git funciona muito bem com código e arquivos de texto, mas mundos de Minecraft têm características diferentes:

- muitos arquivos;
- arquivos binários;
- arquivos `.mca` de regiões;
- dados NBT;
- arquivos de jogadores;
- alterações frequentes;
- mundos potencialmente grandes.

Um simples `git add world/ && git commit` pode funcionar, mas pode ser ineficiente para mundos grandes.

O projeto deve investigar como aproveitar o Git sem ignorar essas características.

---

## 3. Objetivo do MVP

Primeiro construir uma ferramenta local, simples e confiável.

Possíveis comandos:

```bash
mcgit init
mcgit snapshot "Antes de construir a cidade"
mcgit snapshots
mcgit restore <snapshot>
mcgit delete <snapshot>
```

O MVP deve priorizar:

1. segurança dos dados;
2. funcionamento local;
3. integração real com Git;
4. snapshots confiáveis;
5. restauração correta do mundo;
6. simplicidade.

Não começar com cloud ou sistema próprio de contas.

A integração com Arweave/TruthID deve ser planejada desde cedo na arquitetura, mas pode ser implementada depois que o núcleo local estiver sólido.

---

## 4. Arquitetura conceitual

```text
Minecraft World
      |
      v
+----------------------+
|       mcgit          |
|                      |
| Snapshot Manager     |
| World Detection      |
| Git Integration      |
| Restore Manager      |
| Metadata             |
| Remote Storage       |
+----------+-----------+
           |
           v
          Git
           |
           v
        .git/
           |
           | opcional
           v
     TruthID / Arweave
```

O Git continua sendo responsável pelo versionamento local.

O `mcgit` funciona como uma camada especializada para Minecraft.

O Arweave não substitui o Git local. Ele funciona como uma camada de armazenamento remoto e persistente.

---

## 5. Fluxo básico

### Inicialização

```bash
mcgit init
```

A ferramenta deve:

- verificar se o diretório parece ser um mundo válido;
- inicializar um repositório Git;
- criar configurações/metadados necessários;
- evitar adicionar arquivos desnecessários;
- preparar o mundo para snapshots.

### Snapshot

```bash
mcgit snapshot "Construí minha casa"
```

Conceitualmente:

```text
Minecraft World
      ↓
Detectar alterações
      ↓
Preparar arquivos
      ↓
Git add
      ↓
Git commit
      ↓
Snapshot criado
```

### Histórico

```bash
mcgit snapshots
```

Mostrar algo mais amigável que o Git puro:

```text
SNAPSHOTS

● 2026-08-09 18:30
  Construí minha casa

○ 2026-08-09 16:12
  Antes da construção

○ 2026-08-08 22:41
  Comecei o mundo
```

### Restauração

```bash
mcgit restore <snapshot>
```

A ferramenta deve:

- verificar se o Minecraft está usando o mundo;
- alertar sobre alterações não salvas;
- permitir backup/segurança antes da restauração;
- restaurar o estado selecionado;
- minimizar risco de perda de dados.

---

## 6. Git por baixo

A primeira implementação deve realmente utilizar Git.

Exemplo conceitual:

```text
mcgit snapshot
       |
       +--> git add
       |
       +--> git commit
```

O usuário não precisa executar Git manualmente.

No futuro, avaliar se alguma parte precisa de uma estratégia própria de armazenamento ou integração mais profunda com Git.

**Não assumir de antemão que Git puro será suficiente para todos os casos. Testar.**

---

## 7. Problema de armazenamento

Esse é um dos pontos técnicos mais importantes.

Mundos podem ter vários GB e arquivos `.mca` são binários.

Investigar:

- como o Git armazena versões de arquivos binários;
- quanto espaço snapshots reais consomem;
- comportamento quando uma região `.mca` muda;
- compressão;
- deduplicação;
- Git packfiles;
- Git LFS;
- armazenamento por região/chunk;
- possibilidade de uma camada especializada antes do Git.

Não otimizar prematuramente.

Primeiro criar benchmarks com mundos reais.

---

## 8. Minecraft-aware features — futuro

Depois do MVP, o software poderia entender a estrutura do mundo.

### Diff específico

Em vez de:

```text
binary files differ
```

mostrar algo como:

```text
WORLD DIFF

Regions changed: 3

Blocks:
+ 2,341
- 1,827

Entities:
+ 4
- 2

Structures:
+ 1
```

Isso exigiria interpretar NBT e/ou dados das regiões.

---

## 9. Branches

Uma possível feature muito interessante:

```bash
mcgit branch experiment
mcgit checkout experiment
```

Permitir que o jogador experimente sem comprometer o mundo principal.

Exemplo:

```text
main
 |
 +--- snapshot: cidade
 |
 +--- snapshot: farm
 |
 +--- experiment
       |
       +--- destruir vila
       +--- testar construção
       +--- testar alguma alteração
```

Se o experimento não funcionar:

```bash
mcgit checkout main
```

Se houver interesse, estudar posteriormente como merges poderiam funcionar para mundos Minecraft. Não assumir que merge tradicional do Git será seguro ou desejável para arquivos de mundo.

---

## 10. Time Travel

Uma das principais propostas de valor:

> "Quero voltar meu mundo para como ele estava ontem."

Ou:

> "Quero ver meu mundo antes de construir isso."

A ferramenta deve tornar isso simples.

---

## 11. Interface

Começar por CLI.

Exemplo:

```bash
mcgit status
mcgit snapshot "..."
mcgit history
mcgit restore ...
```

Depois, se fizer sentido, criar GUI.

Possível interface:

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

## 12. Arweave + TruthID

### Objetivo

Permitir que o usuário faça backup de snapshots do mundo em armazenamento descentralizado usando **Arweave**.

A integração será feita através do **TruthID**, que já possui a infraestrutura necessária para identidade e pagamento das taxas de Arweave.

Conceitualmente:

```text
Minecraft
    |
    v
  mcgit
    |
    v
Git snapshot
    |
    v
Preparar snapshot/objetos
    |
    v
TruthID
    |
    +--> identidade/autorização
    |
    +--> pagamento das taxas
    |
    v
Arweave
```

### Experiência do usuário

O usuário não deve precisar entender blockchain, Arweave ou pagamento de taxas.

A experiência deve ser parecida com:

```bash
mcgit push arweave
```

ou futuramente:

```bash
mcgit backup
```

com a ferramenta cuidando da infraestrutura através do TruthID.

### O que exatamente será enviado?

Essa decisão precisa ser investigada.

Não assumir que o mundo inteiro deve ser enviado a cada snapshot.

Possibilidades:

- snapshot completo;
- objetos Git necessários;
- arquivos alterados;
- pacotes compactados;
- regiões alteradas;
- snapshots incrementais;
- um bundle contendo uma sequência de versões.

O objetivo é aproveitar deduplicação e reduzir custo.

### Separação de responsabilidades

```text
mcgit
  |
  +--> Git = versionamento
  |
  +--> TruthID = identidade/autorização + pagamento
  |
  +--> Arweave = armazenamento remoto persistente
```

O `mcgit` deve continuar funcionando completamente offline. Arweave é uma funcionalidade adicional.

---

## 13. Recuperação remota

No futuro:

```bash
mcgit pull arweave
```

ou:

```bash
mcgit restore-remote <snapshot>
```

Deve ser possível recuperar um snapshot armazenado remotamente mesmo que o computador original tenha sido perdido.

Fluxo ideal:

```text
PC perdido
    ↓
novo PC
    ↓
instalar mcgit
    ↓
autenticar pelo TruthID
    ↓
encontrar snapshots
    ↓
baixar do Arweave
    ↓
restaurar mundo
```

---

## 14. Integridade e verificabilidade

Estudar mecanismos para garantir:

- hash dos snapshots;
- integridade dos arquivos;
- identificação única de versões;
- associação entre snapshot e commit Git;
- verificação de dados restaurados;
- detecção de corrupção;
- metadados mínimos necessários para recuperação.

Uma possível ideia:

```text
Git Commit
    |
    +--> snapshot ID
    |
    +--> content hash
    |
    +--> Arweave transaction ID
    |
    +--> metadata
```

A estrutura final deve ser definida depois da investigação técnica.

---

## 15. Segurança

Esse projeto mexe diretamente com dados potencialmente valiosos do usuário.

Regras:

- nunca apagar um snapshot silenciosamente;
- confirmar operações destrutivas;
- detectar mundo aberto quando possível;
- criar backup antes de operações arriscadas;
- nunca sobrescrever dados sem uma estratégia de recuperação;
- testar restauração repetidamente;
- considerar corrupção/interrupção durante snapshot;
- verificar integridade antes de restaurar dados remotos;
- não expor credenciais/chaves privadas em logs.

Também deve existir uma forma clara de evitar que o usuário publique acidentalmente dados privados em armazenamento permanente do Arweave.

---

## 16. Multiplayer / servidores

Possível suporte futuro:

```text
Minecraft Server
       |
       v
     mcgit
       |
       +------> Git
       |
       +------> TruthID
                   |
                   v
                Arweave
```

Possibilidades:

- snapshots automáticos;
- snapshot antes de restart;
- rollback;
- histórico;
- backups;
- branches para testes;
- recuperação após erro;
- backup remoto automático.

Isso pode ser mais valioso comercialmente do que apenas versionamento de mundos single-player.

---

## 17. Cloud / remoto tradicional

Não é prioridade.

O projeto deve continuar permitindo um remote Git tradicional caso faça sentido:

```bash
mcgit push
```

Mas o armazenamento descentralizado via TruthID + Arweave deve ser tratado como uma opção de primeira classe da arquitetura.

Possibilidades futuras:

```text
mcgit
 ├── Git remote
 ├── Arweave
 └── storage self-hosted
```

---

## 18. Tecnologias

Ainda não decidir definitivamente.

### Rust

Vantagens:

- ótimo para CLI;
- binário único;
- desempenho;
- baixo consumo;
- bom para manipulação de arquivos;
- interessante para uma ferramenta de infraestrutura.

### Python

Vantagens:

- desenvolvimento rápido;
- excelente para prototipagem;
- fácil integração com processos externos.

A decisão deve ser tomada depois de avaliar o escopo e os requisitos.

---

## 19. Roadmap sugerido

### Fase 0 — Pesquisa

- entender estrutura de mundos Minecraft;
- entender `.mca`;
- entender NBT;
- testar Git com mundos reais;
- medir tamanho dos repositórios;
- testar restauração;
- investigar estratégia de armazenamento remoto;
- definir como o TruthID será integrado ao `mcgit`.

### Fase 1 — MVP local

- `init`;
- `snapshot`;
- `history`;
- `restore`;
- integração Git;
- validações;
- backups de segurança.

### Fase 2 — Qualidade

- snapshots automáticos;
- melhor tratamento de mundos grandes;
- status;
- configuração;
- logs;
- testes de corrupção/interrupção.

### Fase 3 — Minecraft-aware

- parser NBT;
- diff de regiões;
- estatísticas;
- visualização de alterações.

### Fase 4 — Branching

- branches;
- experimentos;
- comparação entre versões;
- merge, se fizer sentido tecnicamente.

### Fase 5 — Arweave + TruthID

- integração com o TruthID existente;
- autenticação/autorização;
- preparação de snapshots para armazenamento remoto;
- upload para Arweave;
- associação entre snapshot Git e transação Arweave;
- verificação de integridade;
- recuperação;
- `push`/`pull` remoto;
- tratamento de falhas e uploads interrompidos;
- controle/estimativa de custos.

### Fase 6 — Servidores

- integração com servidores;
- snapshots automáticos;
- hooks;
- rollback rápido;
- backup remoto automático.

### Fase 7 — Interface

- TUI ou GUI;
- histórico visual;
- comparação;
- restauração;
- gerenciamento de backups remotos.

---

## 20. Princípios do projeto

1. **Git é o motor de versionamento inicial.**
2. **Minecraft é o domínio que a ferramenta entende.**
3. **TruthID fornece identidade/autorização e a infraestrutura de pagamento para a camada Web3.**
4. **Arweave fornece armazenamento remoto persistente.**
5. **CLI primeiro.**
6. **Local-first.**
7. **Offline-first para as funções básicas.**
8. **Não depender de cloud para funcionar.**
9. **Não reinventar o Git sem necessidade.**
10. **Testar armazenamento antes de criar otimizações complexas.**
11. **Dados do usuário são prioridade.**
12. **Restauração precisa ser extremamente confiável.**
13. **Arweave é uma camada opcional, não um requisito para usar o software.**
14. **Não expor a complexidade de blockchain ao usuário final.**
15. **Features avançadas só depois de um MVP sólido.**

---

## 21. Perguntas para decidir durante o desenvolvimento

- Git puro é suficiente?
- Vale a pena usar Git LFS?
- Devemos versionar `.mca` diretamente?
- Devemos dividir/interpretar regiões?
- Rust ou Python?
- CLI pura ou TUI?
- Como detectar se o mundo está aberto?
- Como garantir restauração atômica?
- Como lidar com mundos de dezenas/centenas de GB?
- Snapshots automáticos devem ser padrão?
- Branches realmente precisam de merge?
- Como lidar com servidores em execução?
- Qual é a unidade ideal de armazenamento no Arweave?
- Devemos armazenar snapshots completos ou objetos/deltas?
- Como aproveitar deduplicação?
- Como mapear um commit Git para uma transação Arweave?
- Quais metadados são necessários para recuperar um mundo em outro computador?
- Como o TruthID deve expor a integração ao `mcgit`?
- Como lidar com custos de armazenamento?
- Como lidar com uploads interrompidos?
- Como verificar a integridade de um snapshot remoto?
- Como evitar que dados privados do mundo sejam publicados acidentalmente no Arweave?

---

## 22. Primeira tarefa para o agente de desenvolvimento

Antes de escrever uma grande quantidade de código:

1. analisar esta especificação;
2. questionar premissas técnicas;
3. pesquisar/analisar como Git lida com arquivos `.mca`;
4. propor uma arquitetura inicial;
5. definir um MVP pequeno;
6. criar benchmarks para medir armazenamento;
7. estudar como integrar o TruthID existente;
8. definir uma estratégia preliminar para Arweave;
9. somente depois implementar.

**Importante:** não implementar todas as features descritas de uma vez.

Este arquivo é uma visão do projeto, não uma especificação rígida. As decisões técnicas podem mudar conforme os testes mostrarem o que funciona melhor.

---

## 23. Visão de longo prazo

```text
                    Minecraft World
                           |
                           v
                         mcgit
                           |
             +-------------+-------------+
             |                           |
             v                           v
            Git                    TruthID + Arweave
       Versionamento                Backup remoto
             |                           |
             +-------------+-------------+
                           |
                           v
                  Versioned Minecraft
```

O usuário deve conseguir:

- criar snapshots;
- voltar no tempo;
- experimentar mudanças;
- criar branches;
- comparar versões;
- manter backups;
- recuperar um mundo perdido;
- levar seu histórico para outro computador;
- armazenar backups remotamente sem depender de um servidor central do projeto.

A complexidade técnica deve ficar por trás da ferramenta. Para o jogador, deve parecer simplesmente que existe um **"Git para Minecraft"**.
