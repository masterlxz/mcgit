# mcgit - PRD v1.0

## Vision

mcgit is a version control tool built specifically for Minecraft worlds, using Git as the
underlying versioning engine.

The goal is not to reinvent Git. mcgit hides Git's complexity and offers an experience close to:

> **Git + Time Machine for Minecraft worlds.**

The player works with their world normally and creates snapshots/versions through the tool,
without ever needing to type a Git command directly.

Beyond local storage, the project has a future layer for **decentralized remote backup using
Arweave**, integrated with the already-existing **TruthID** platform (`../truthid`). TruthID
is responsible for identity/authorization and for the payment mechanism already in place for
Arweave storage fees.

The project is NOT a Minecraft server host, a mod, or a general-purpose backup tool. The core
focus is version control (snapshot / history / restore / branch) for world data.

---

## Core Problem

Git works very well for code and text files, but Minecraft worlds have different
characteristics:

* many files
* binary files
* `.mca` region files
* NBT data
* player data files
* frequent changes
* potentially large worlds

A plain `git add world/ && git commit` may technically work, but may be inefficient for large
worlds. The project must investigate how to leverage Git without ignoring these
characteristics — see Fase 0 in `PHASE.md`.

---

## Core Concepts

### World

A Minecraft world directory (single-player save or server world folder) that mcgit tracks.
`mcgit init` turns a world directory into an mcgit-tracked world (backed by a Git repository).

### Snapshot

A named, timestamped version of the world at a point in time. Conceptually equivalent to a
Git commit, but exposed to the user through Minecraft-friendly language and UX (no Git
terminology required).

### History

The ordered list of snapshots for a world, shown in a friendly format (see `mcgit snapshots`
in Fase 1 of `PHASE.md`).

### Restore

Bringing the world back to the exact state captured by a given snapshot, with safety checks
(world not currently open, unsaved-changes warning, pre-restore safety backup) to minimize
risk of data loss.

### Branch (future — Fase 4)

An alternate timeline for a world, letting the player experiment (build, destroy, test)
without committing changes to the main world. Whether/how merges between branches should work
is an open question — see `ARCHITECTURE.md`. Traditional Git merge is not assumed to be safe
or desirable for world files.

---

## User Flow

### Init

```bash
mcgit init
```

1. Verify the directory looks like a valid Minecraft world.
2. Initialize a Git repository underneath.
3. Create the metadata/config needed by mcgit.
4. Avoid tracking unnecessary files.
5. Prepare the world for snapshots.

### Snapshot

```bash
mcgit snapshot "Built my house"
```

1. Detect changes in the world directory.
2. Prepare files (respecting any exclusion rules).
3. `git add` the changes.
4. `git commit` with the snapshot message.
5. Snapshot created.

### History

```bash
mcgit snapshots
```

Show a human-friendly view of the snapshot history (not raw `git log`).

### Restore

```bash
mcgit restore <snapshot>
```

1. Check whether Minecraft currently has the world open, when detectable.
2. Warn about unsaved changes.
3. Offer/perform a safety backup before restoring.
4. Restore the selected state.
5. Minimize risk of data loss throughout.

---

## Storage Problem

This is one of the most important technical points of the whole project.

Worlds can be multiple GB, and `.mca` region files are binary and can churn frequently.

To investigate (Fase 0, before any optimization):

* how Git stores versions of binary files;
* how much space real snapshots actually consume;
* behavior when a `.mca` region changes;
* compression;
* deduplication;
* Git packfiles;
* Git LFS;
* per-region/per-chunk storage strategies;
* whether a specialized layer in front of Git is worth it.

Principle: **do not optimize prematurely — build benchmarks against real worlds first.**

---

## Minecraft-Aware Features (Future — Fase 3)

Instead of a generic binary diff:

```text
binary files differ
```

show something like:

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

This requires interpreting NBT and/or region data.

---

## Arweave + TruthID Integration (Future — Fase 5)

### Goal

Let the user back up world snapshots to decentralized storage (Arweave). The integration goes
through TruthID, which already has the infrastructure for identity and for paying Arweave
storage fees.

### User Experience

The user should not need to understand blockchain, Arweave, or fee payment.

```bash
mcgit push arweave
```

or eventually:

```bash
mcgit backup
```

with mcgit handling the infrastructure through TruthID under the hood.

### What exactly gets uploaded

To be investigated — do not assume the whole world gets re-uploaded on every snapshot.
Candidates: full snapshot, only the necessary Git objects, changed files only, compressed
packs, changed regions only, incremental snapshots, a bundle of a version sequence. Goal:
leverage deduplication and minimize cost.

### Separation of Responsibilities

```text
mcgit
  |
  +--> Git = versioning
  |
  +--> TruthID = identity/authorization + fee payment
  |
  +--> Arweave = persistent remote storage
```

mcgit must keep working completely offline. Arweave is an optional add-on, never a requirement.

### Remote Recovery

```bash
mcgit pull arweave
# or
mcgit restore-remote <snapshot>
```

It must be possible to recover a remotely-stored snapshot even if the original computer was
lost:

```text
lost PC → new PC → install mcgit → authenticate via TruthID
  → find snapshots → download from Arweave → restore world
```

### Integrity & Verifiability

Must be studied: snapshot hashes, file integrity, unique version IDs, the mapping between a
snapshot and its Git commit, verification of restored data, corruption detection, and the
minimum metadata needed for recovery. A rough shape:

```text
Git Commit
    |
    +--> snapshot ID
    +--> content hash
    +--> Arweave transaction ID
    +--> metadata
```

The final structure will be defined after technical investigation.

---

## Multiplayer / Servers (Future — Fase 6)

```text
Minecraft Server → mcgit → Git
                       └──→ TruthID → Arweave
```

Possibilities: automatic snapshots, snapshot-before-restart, rollback, history, backups,
branches for testing, recovery after a crash, automatic remote backup. This may end up more
commercially valuable than single-player world versioning alone — see `ROADMAP.md`.

---

## Security Requirements

* Never silently delete a snapshot.
* Confirm destructive operations.
* Detect an open world when possible.
* Create a backup before risky operations.
* Never overwrite data without a recovery strategy.
* Test restoration repeatedly.
* Consider corruption/interruption during a snapshot.
* Verify integrity before restoring remote data.
* Never expose credentials/private keys in logs.
* Provide a clear way to avoid accidentally publishing private world data to Arweave's
  permanent storage.

---

## Interface

CLI first (Fase 1). TUI or GUI are possible later (Fase 7), once the CLI experience is solid.

---

## Non Goals

The following are explicitly out of scope for mcgit:

* Being a Minecraft server host or launcher.
* Being a mod or requiring the user to install one.
* Being a general-purpose file backup tool (scope is world data, not arbitrary files).
* Implementing a custom blockchain, token, or wallet (TruthID already owns that layer).
* Cloud-first design — mcgit must be local-first and fully usable offline; Arweave/TruthID is
  an optional add-on, not a requirement.

---

## Monetization (Optional Future)

The core tool (local versioning, CLI) is open source (MIT) and will remain so, matching
TruthID's protocol-is-free stance. A business may be built on top in the future — e.g. managed
hosting, a managed remote-backup service, or support for server operators — without changing
the open-source nature of the core tool itself. See `ROADMAP.md` for the current thinking; no
concrete plan exists yet.
