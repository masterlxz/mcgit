# Pendências do Projeto

Nenhuma pendência registrada ainda — projeto em Fase 0 (pesquisa), sem código escrito.

Quando uma pendência (bug, débito técnico, decisão adiada) surgir, registrar aqui com um ID
sequencial (`#1`, `#2`, ...), seguindo o mesmo formato usado em `../truthid/project/PENDING.md`.

---

## Não Resolvidas

### #1 — Solicitar acesso ao escopo `XboxLive.signin` via ID@Xbox

Descoberto na revisão legal/licenciamento da Sessão 2 (2026-08-15): o escopo OAuth
`XboxLive.signin`, necessário pro fluxo de login Microsoft (MS OAuth → Xbox Live → XSTS →
Minecraft Services), é restrito. Um app registration comum no Azure não recebe esse escopo
automaticamente — é preciso se inscrever no Xbox Developer Program via **ID@Xbox** (via pra
devs independentes/pequenos), o mesmo caminho que Prism Launcher e MultiMC percorreram.

- **Bloqueia**: testes reais de login/autenticação Microsoft (não bloqueia o resto da Fase 0/1).
- **Ação prática**: (1) criar o app registration no Azure normalmente — pode ser feito a
  qualquer momento; (2) submeter pedido de acesso ao escopo via ID@Xbox — tem revisão humana da
  Microsoft, prazo desconhecido. Fazer isso cedo (mesmo antes do código de auth estar pronto)
  pra não travar o fim da Fase 1 esperando aprovação externa.
- Detalhe completo: `CONTEXT.md` §Legal & Licensing Considerations.

---

## Resolvidas

Nenhuma.
