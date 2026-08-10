# mcgit

A cross-platform Minecraft Launcher — inspired by Prism Launcher, Modrinth App and ATLauncher —
built around one core differentiator: **every world can be versioned with Git**, through a UI
that never requires you to know Git exists. "Git + Time Machine for Minecraft worlds," inside a
launcher that also handles accounts, Java, instances, mods, modpacks, skins, and backups.

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

Fase 0 (research & architecture) in progress — first real-world Git benchmark done, no product
code yet. See [`project/INDEX.md`](project/INDEX.md) for the full project state,
[`project/CONTEXT.md`](project/CONTEXT.md) for the PRD, and [`project/PHASE.md`](project/PHASE.md)
for the full phased roadmap (Fase 0 through Fase 10). `project/` is the single source of truth
for this project going forward.

## Principles

- Git is the versioning engine for worlds — not reinvented without a reason.
- Local-first, offline-first for all core functionality.
- The GUI is the primary product surface (it's a launcher); the CLI is optional, never required.
- Every instance is isolated; every world's versioning is opt-in.
- Arweave/TruthID backup is an optional add-on, never a requirement.
- Open source (MIT) — the core stays free and open even if a business is built on top later.

## License

MIT — see [`LICENSE`](LICENSE).
