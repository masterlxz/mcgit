# Diretriz de código (IMPORTANTE — sempre seguir)

**Todo código novo deve ser escrito em inglês — sem exceção.**
- Strings visíveis ao usuário (CLI output, mensagens de erro, help text): inglês
- Nomes de variáveis, funções, structs/classes, arquivos: inglês
- Comentários no código: podem ficar em português (não são visíveis ao usuário e facilitam o aprendizado)
- Esta regra vale para todos os arquivos do projeto, qualquer que seja a linguagem escolhida na Fase 0 (`.rs`, `.py`, etc.)

**I18n (múltiplos idiomas) não é prioridade agora.** Se/quando houver demanda, extrair strings
visíveis para arquivos de tradução, com inglês como idioma base (source of truth).

---

# Diretriz de ensino (IMPORTANTE — ler antes de cada sessão)

O usuário pediu explicitamente **modo ensino (devagar)** para este projeto: explicar conceitos
antes de codar, em vez de simplesmente entregar código pronto.

Isso vale especialmente para áreas novas neste projeto: internals do Git (objects, packfiles,
refs), a linguagem escolhida na Fase 0 caso não seja uma que o usuário já domine, e o formato
de mundos Minecraft (NBT, `.mca`/regiões, anvil format).

Base de comparação conhecida de projetos anteriores (TruthID, mesmo usuário): Python (bom) e
Ruby (básico) como linguagens de referência para analogias; exposição a Rust (desktop Tauri) e
TypeScript/Dart (desktop/mobile) através daquele projeto, mas não necessariamente domínio ativo
dessas linguagens. **Confirmar com o usuário no início da Fase 0 / primeira sessão de código**
se essa base ainda é válida, antes de assumir um nível de conhecimento.

**Regras para o Claude:**
- Explicar o conceito ANTES de escrever o código
- Introduzir um conceito novo de cada vez — nunca vários ao mesmo tempo
- Usar analogias do mundo real antes de termos técnicos
- Comparar a linguagem/ferramenta nova com algo que o usuário já conhece (Python/Ruby) sempre que possível
- Perguntar se o usuário entendeu antes de avançar — esperar confirmação
- Não assumir conhecimento prévio de internals de Git, da linguagem escolhida, ou do formato de arquivos do Minecraft
- Ritmo lento e deliberado é melhor que velocidade
- **Nunca escrever um bloco grande de código sem explicar depois linha por linha**
- Quando escrever código novo, percorrer cada trecho explicando o que faz e por quê
- Quando explicar código já escrito, dividir em partes e pedir confirmação antes de avançar para a próxima parte

---

# Princípios do projeto

Do documento de ideação original (v1.0, versionamento de mundo) mais os ajustados na revisão
de escopo pra launcher completo (Sessão 1, v2.0 — ver `CONTEXT.md`/`ARCHITECTURE.md`):

1. Git é o motor de versionamento de mundos.
2. Minecraft é o domínio que o launcher entende — não só versionamento, o pacote completo (Java, instâncias, mods, contas).
3. TruthID fornece identidade/autorização e a infraestrutura de pagamento para a camada Web3.
4. Arweave fornece armazenamento remoto persistente.
5. **GUI é a interface principal** (revisado — era "CLI primeiro" na v1.0, quando o projeto era só uma ferramenta de linha de comando; um launcher é fundamentalmente gráfico). CLI existe em paralelo, sempre opcional.
6. Local-first.
7. Offline-first para as funções básicas.
8. Não depender de cloud para funcionar.
9. Não reinventar o Git sem necessidade.
10. Testar armazenamento antes de criar otimizações complexas.
11. Dados do usuário são prioridade.
12. Restauração precisa ser extremamente confiável.
13. Arweave é uma camada opcional, não um requisito para usar o software.
14. Não expor a complexidade de blockchain ao usuário final.
15. Features avançadas só depois de um MVP sólido.
16. Open source puro — o core nunca deixa de ser livre, mesmo que um negócio seja construído em cima no futuro.
17. Instâncias são isoladas entre si — sem conflito entre diferentes Minecrafts/modloaders/mods instalados.
18. Nenhum código que toque autenticação Microsoft, CurseForge ou API de skins antes da revisão legal/licenciamento da Fase 0 (ver `CONTEXT.md` §Legal & Licensing).

---

# Identidade Visual

O usuário mantém um ecossistema de projetos onde cada um tem uma cor própria de identidade:
Anchor é verde, TruthID é azul, Warden é roxo. **mcgit é vermelho** (decidido na Sessão 2,
2026-08-16) — mcgit faz parte desse mesmo ecossistema.

**Tom exato decidido (Sessão 10, continuação, 2026-09-02)**: vermelho vivo, tipo "Minecraft
redstone" — `#E11D2E` no modo claro, `#FF4C57` no modo escuro (mais claro pra manter contraste
num fundo escuro), escolhido pelo usuário entre três direções (crimson sóbrio, vivo/redstone,
terroso/tijolo) via `AskUserQuestion`. Aplicado como paleta de tokens CSS completa (fundo,
superfície, texto, bordas, primário, perigo, sucesso, aviso — claro e escuro via
`prefers-color-scheme`, sem toggle explícito de tema ainda) em `apps/desktop/src/App.css`.
Detalhes completos em `ARCHITECTURE.md` §Identidade Visual & Design System.

**Ícone do app e favicon fechados (Sessão 10, terceira continuação, 2026-09-02)**: um glifo de
"git branch" (três nós + duas linhas, uma reta e uma curva) em branco sobre um quadrado
arredondado vermelho de marca — comunica as duas metades do nome "mcgit" numa imagem só, em vez
de uma letra genérica. Fonte editável em `apps/desktop/src-tauri/icons/icon-source.svg`; todo o
resto do conjunto (ico/icns/pngs) gerado dela via `npx tauri icon`. Favicon da build web usa o
mesmo SVG. Detalhes completos em `ARCHITECTURE.md` §Identidade Visual & Design System (subseção
"Terceira continuação — Ícone do app e favicon").
