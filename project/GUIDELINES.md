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

# Princípios do projeto (do documento de ideação original)

1. Git é o motor de versionamento inicial.
2. Minecraft é o domínio que a ferramenta entende.
3. TruthID fornece identidade/autorização e a infraestrutura de pagamento para a camada Web3.
4. Arweave fornece armazenamento remoto persistente.
5. CLI primeiro.
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
