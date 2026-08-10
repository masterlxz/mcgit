# mcgit

Version control for Minecraft worlds, built on top of Git. The goal: "Git + Time Machine for
Minecraft worlds" — snapshot, browse history, and restore your world without ever touching Git
directly.

```bash
mcgit init
mcgit snapshot "Before building the city"
mcgit snapshots
mcgit restore <snapshot>
```

A future, fully optional layer adds decentralized remote backup via **Arweave**, with identity
and fee payment handled through **TruthID** (`../truthid`) — the tool stays 100% usable and
local-first without it.

## Status

Fase 0 (research) in progress — first real-world Git benchmark done, no product code yet. See
[`project/INDEX.md`](project/INDEX.md) for the full project state, [`project/CONTEXT.md`](project/CONTEXT.md)
for the PRD, and [`project/PHASE.md`](project/PHASE.md) for the phased plan. `project/` is the
single source of truth for this project going forward.

## Principles

- Git is the initial versioning engine — not reinvented without a reason.
- Local-first, offline-first for all core functionality.
- CLI first.
- Arweave/TruthID backup is an optional add-on, never a requirement.
- Open source (MIT) — the core stays free and open even if a business is built on top later.

## License

MIT — see [`LICENSE`](LICENSE).
