# Pendências do Projeto

Nenhuma pendência registrada ainda — projeto em Fase 0 (pesquisa), sem código escrito.

Quando uma pendência (bug, débito técnico, decisão adiada) surgir, registrar aqui com um ID
sequencial (`#1`, `#2`, ...), seguindo o mesmo formato usado em `../truthid/project/PENDING.md`.

---

## Não Resolvidas

### #1 — Solicitar acesso ao escopo `XboxLive.signin` via ID@Xbox — **PAUSADO**

Descoberto na revisão legal/licenciamento da Sessão 2 (2026-08-15): o escopo OAuth
`XboxLive.signin`, necessário pro fluxo de login Microsoft (MS OAuth → Xbox Live → XSTS →
Minecraft Services), é restrito. Um app registration comum no Azure não recebe esse escopo
automaticamente — é preciso se inscrever no Xbox Developer Program via **ID@Xbox** (via pra
devs independentes/pequenos), o mesmo caminho que Prism Launcher e MultiMC percorreram.

- **Bloqueia**: testes reais de login/autenticação Microsoft (não bloqueia o resto da Fase 0/1).

**Investigação da Sessão 3 (2026-08-16)** — tentativa real de preencher o formulário de
cadastro como Xbox Partner (pré-requisito do ID@Xbox), com prints da tela real:

- Existem (pelo menos) dois caminhos possivelmente confundidos entre si nas fontes públicas:
  um formulário específico "Permission to use the Minecraft API" (`aka.ms/mce-reviewappid`,
  citado pela Minecraft Wiki) que pareceria mais leve, vs. o cadastro completo de **Xbox
  Partner** (Partner Center) — o usuário abriu o segundo. Não confirmado se o primeiro ainda é
  um caminho válido/ativo ou se convergiu pro segundo.
- O formulário de cadastro de Partner aceita pessoa física sem empresa formal: o campo
  "Legally registered business name" aceita nome completo se for "sole proprietor", e
  "Business title" sugere "Sole Proprietor" como opção válida.
- **Bloqueio real encontrado**: o campo **DUNS Number** aparece como obrigatório na prática
  (não deixa avançar sem preencher), apesar do texto de ajuda do próprio formulário sugerir que
  seria opcional ("you may be asked for additional identification... if not provided"). Texto
  de ajuda parece não bater com a validação real da tela.
- E pra tirar um DUNS number no Brasil, a **Dun & Bradstreet exige CNPJ** — não emite pra CPF
  de pessoa física sem empresa registrada. Prazo: gratuito em até 30 dias úteis, ou pago
  (~24-48h acelerado) via CIAL D&B Brasil (`pt.cialdnb.com/duns-number`).
- **Cadeia de dependência completa descoberta**: abrir CNPJ (ex.: MEI) → tirar DUNS number →
  formulário de cadastro Xbox Partner → aplicação ID@Xbox → aprovação humana do escopo
  `XboxLive.signin`. Bem mais pesada do que o "só preencher um formulário" assumido antes.
- **Decisão do usuário (2026-08-16)**: pausar essa frente por ora. Não abrir CNPJ só por causa
  disso no momento. Retomar quando/se fizer sentido (ex.: se abrir MEI por outro motivo, ou se
  achar um caminho alternativo sem CNPJ que ainda não foi identificado).
- **Correção**: o app registration no Azure NÃO é mais autosserviço independente como
  assumido acima. A Microsoft descontinuou criar app registrations fora de um "directory"
  (tenant) — a conta pessoal do usuário (`fabio.anjos.junior@gmail.com`) não tem um tenant por
  trás, e a tela deu o erro "The ability to create applications outside of a directory has been
  deprecated." As duas saídas oferecidas pela própria tela: (a) **M365 Developer Program** —
  tentado, **bloqueado**: mensagem "You don't currently qualify for a Microsoft 365 Developer
  Program sandbox subscription" — política restringiu esse sandbox gratuito, hoje fica
  reservado principalmente pra quem já tem assinatura ativa do Visual Studio
  Professional/Enterprise; (b) **conta Azure gratuita** — não tentada, mas confirmado por
  pesquisa que mesmo o tier gratuito ("Entra ID Free") exige cartão de crédito só pra
  verificação de identidade (não cobra, mas exige o cartão).
- **Decisão do usuário (2026-08-16)**: pausar essa frente também, junto com o ID@Xbox. Não
  seguir pro cadastro Azure (que pediria cartão) por ora.
- **Ação prática (retomada futura)**: (1) se decidir seguir, o único caminho restante é o
  cadastro de conta Azure gratuita (pede cartão de crédito + telefone, sem cobrança automática,
  cria um "Default Directory" que destrava a tela de App registrations); (2) antes de reabrir
  esta investigação, vale checar se `aka.ms/mce-reviewappid` ainda existe como caminho separado
  e mais leve pro ID@Xbox propriamente (não confirmado nesta sessão); (3) se for adiante com o
  ID@Xbox, MEI é a via mais rápida/barata de CNPJ no Brasil (registro online, geralmente no
  mesmo dia, sem custo de abertura), necessário pra tirar um DUNS number.
- Detalhe completo: `CONTEXT.md` §Legal & Licensing Considerations.

---

## Resolvidas

Nenhuma.
