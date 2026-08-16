# mcgit - PRD v2.0

> v2.0 supersedes v1.0 (Sessão 1, "just a world-versioning CLI"). Scope expanded in Sessão 1
> (same day) to a full cross-platform Minecraft Launcher, with Git-based world versioning as
> its core differentiator rather than its entire purpose. Nothing from v1.0 was discarded — it
> now lives inside the "World Manager" / "Git Engine" module described below.

## Vision

mcgit is a modern, cross-platform Minecraft Launcher — inspired by tools like Prism Launcher,
Modrinth App and ATLauncher — built around one core differentiator that none of them have:
**every world can be versioned with Git, with a UI that never requires the player to know Git
exists.**

The goal is not to be "just another launcher." The goal is an ecosystem where installing,
configuring, playing, versioning, backing up, and sharing Minecraft instances and worlds is
extremely easy — combining the best ideas from existing tools (Prism's instance isolation,
Modrinth App's simplicity, ATLauncher's modpack handling, Git's versioning model, Steam's
install experience, Docker's isolated-environment concept) into one product, not by copying
any of them.

Philosophy:

> "The user doesn't need to know how Minecraft works internally. The launcher handles that."

Long-term framing: not "another Minecraft launcher," but **an operating system for your
Minecraft worlds** — Minecraft + Java + instances + mods + modpacks + skins + worlds + Git +
backups + cloud + Arweave, unified under one question the player actually cares about: *"What
do I want to play?"*

---

## Core Problem

Two problems, previously treated as separate projects, are now treated as one product:

**1. Fragmented setup experience.** Installing Minecraft the "right way" today means juggling
Java versions, modloaders (Fabric/Forge/NeoForge), mod compatibility, resource packs, shaders,
and manual account/skin management — much of it manual, error-prone, and hostile to non-technical
players. Existing launchers solve pieces of this, but each optimizes for a different audience
(power users, modpack curators, or pure simplicity) and none of them treat the world itself as
something worth versioning.

**2. Fragile, unversioned world data.** (Carried over from v1.0 — still fully valid.) Git works
very well for code and text, but Minecraft worlds are many binary `.mca` region files, NBT data,
frequent changes, and potentially large size. A world with no history is one bad build, one
corrupted save, or one bad mod interaction away from being unrecoverable.

mcgit treats both problems as the same problem: **the player should never have to think about
the machinery (Java, Git, modloaders, blockchain) to get a reliable, easy-to-manage Minecraft
experience.**

---

## Core Concepts

### Instance

An isolated, self-contained Minecraft installation — its own Minecraft version, modloader,
mods, configs, resource packs, shaders, saves, screenshots, logs, Java configuration, and JVM
arguments. Instances never conflict with each other. Flow: **create instance → pick version →
pick modloader → install → play.**

### Java Runtime

mcgit auto-detects which Java version an instance needs (e.g. Minecraft 1.20.1 → Java 17;
1.21.x → Java 21) and offers to install it automatically if missing — the player is never sent
to a Java download page. Advanced configuration remains available for power users.

### Account (Microsoft)

Login through the official Microsoft/Minecraft OAuth flow. The launcher never stores the user's
password. Multiple accounts are a future capability, not MVP.

### Modloader

Fabric, Forge, NeoForge (and others where feasible), auto-detected and auto-installed per
instance based on what the instance/modpack requires.

### Mod / Modpack

A mod is a single addon; a modpack is a curated bundle (version, loader, mods, configs,
resource packs pinned together). mcgit aims to integrate with existing ecosystems (Modrinth,
CurseForge — see Legal & Licensing below) rather than build its own mod repository. Installing
a modpack should never require the player to manually download a ZIP and extract files.

### Resource Pack / Shader

Per-instance, toggleable, managed the same way mods are.

### Skin

View current skin, change skin, import a skin, manage multiple saved skins — always through
official Mojang/Microsoft APIs and rules, never scraping or storing skins outside those rules.

### World (carried over from v1.0)

A Minecraft world directory (single-player save or server world folder) that mcgit tracks.
Enabling versioning on a world initializes a Git repository underneath it — opt-in per world,
not forced.

### Snapshot / History / Restore (carried over from v1.0)

A snapshot is a named, timestamped version of a world (conceptually a Git commit, exposed
through Minecraft-friendly language, never Git terminology, by default). History is the
snapshot timeline. Restore brings the world back to a snapshot's exact state, with safety
checks: world not currently open, unsaved-changes warning, mandatory safety checkpoint before
restoring (see Fase 0 benchmark results in `ARCHITECTURE.md` for why this is proven feasible
with plain Git).

### Branch (carried over from v1.0, future — see `PHASE.md`)

An alternate timeline for a world, for risk-free experimentation (testing TNT, testing mods,
destructive builds) without touching the main world. Whether/how merge should work between
world branches remains an open question — traditional Git merge is not assumed to be safe for
binary world files. Not MVP.

### Environment / Reproducibility Metadata (new)

Every instance (and, by extension, every world snapshot) can carry a small metadata record
describing the environment it ran in:

```json
{
  "minecraft": "1.21.x",
  "loader": "fabric",
  "loader_version": "...",
  "java": "21",
  "mods": [{ "name": "Sodium", "version": "..." }],
  "resource_packs": [],
  "shaders": []
}
```

Goal: "this world worked in this environment" — and mcgit can attempt to reconstruct that exact
environment later, on the same machine or a different one. This is what makes world *sharing*
(see below) practical for modded worlds, not just vanilla ones.

### Backup Targets (new)

mcgit treats these as explicitly different things, not synonyms:

```text
Local Save          — the world directory itself, unversioned
Git Repository       — local version history (default, if versioning is enabled)
Backup               — a local copy/archive outside the working directory
Cloud Backup          — a traditional remote (self-hosted or third-party Git remote)
Arweave Archive        — permanent, decentralized storage (optional, future — Fase 7)
```

The player can opt into any combination (e.g. Local + Cloud, without Arweave).

---

## User Flows

### Instance Creation

1. Player picks a Minecraft version and (optionally) a modloader.
2. mcgit checks for the required Java version; offers to auto-install if missing.
3. mcgit downloads Minecraft version files, libraries, and natives — no manual file hunting.
4. Instance is created, isolated from all other instances.

### Microsoft Login

```text
Open Launcher → Sign in with Microsoft → OAuth authentication
  → Verify Minecraft ownership → Launcher recognizes the player
```

No password ever stored. Official OAuth flow only.

### Modpack Install

```text
Modpacks → Search → Pick modpack → Install → Play
```

Where feasible: install, update, downgrade, dependency resolution, automatic modloader
detection, conflict handling — all without the player touching a ZIP file.

### World Versioning (carried over from v1.0)

```text
Minecraft World → detect changes → prepare files → git add → git commit → Snapshot created
```

Exposed to the player as "Save version," not "git commit" (see Interface Philosophy). Mockup
of the per-world timeline UI:

```text
My World

Timeline

● Today — 8:30 PM
  Killed the Ender Dragon

● Today — 6:20 PM
  Built my XP farm

● Yesterday — 11:10 PM
  Built my house

● Yesterday — 8:00 PM
  First day

[ Save version ]  [ Restore version ]  [ Compare ]  [ Create Branch ]  [ Backup ]
```

### Rollback (carried over from v1.0, expanded UI copy)

```text
Restore version?

"Before exploring the Nether"

This will replace the world's current state.

[ Cancel ]   [ Create Backup and Restore ]
```

A safety checkpoint of the current state is always created before restoring an older version —
non-negotiable, matches the Security Requirements below.

### World Sharing (new, future — see `PHASE.md`)

```text
My World → Share → generates a World ID
             ↓
Another player → Import World → Download → Verify integrity → Install
```

Using the Environment Metadata above, mcgit attempts to auto-resolve the Minecraft version,
modloader, mods (and their versions), resource packs, and shaders needed to reproduce the
world faithfully — turning "here's my modded world, good luck setting it up" into an import
flow.

---

## Interface Philosophy

The GUI is the primary product surface — a launcher is fundamentally a graphical experience
("what do I want to play?" is not a question players ask a terminal). This **reverses** the
"CLI first" decision from v1.0's mcgit-as-standalone-tool scope; see `ARCHITECTURE.md` for the
explicit decision log.

* Default UI language is Minecraft-domain, not Git-domain: "Versions of the world," not "Git
  commits." "Save version," not "git commit."
* A "Básico / Avançado" (Basic / Advanced) toggle exists for players who do want to see the
  underlying Git repository, branches, remotes, and diffs.
* A CLI exists in parallel (`mcgit init/commit/log/restore` for world versioning, plus
  launcher-level commands like `mcgit create`/`mcgit launch`/`mcgit world list`) but is
  strictly optional — it must never be a requirement for a casual player, only a convenience
  for automation and power users.

Mockup of the home screen (Fase 1 MVP — deliberately sparse, no wall of settings for beginners):

```text
┌────────────────────────────────────────────┐
│ mcgit                                       │
│                                              │
│  My Minecraft                               │
│                                              │
│  ┌──────────────────────────────────────┐  │
│  │ Survival                              │  │
│  │ Minecraft 1.21.x                      │  │
│  │                                        │  │
│  │              [ PLAY ]                 │  │
│  └──────────────────────────────────────┘  │
│                                              │
│  [+ Create Instance]                        │
│                                              │
│  Instances   Modpacks   Worlds   Skins      │
│                                              │
└──────────────────────────────────────────────┘
```

---

## Module Architecture (high-level — detailed interfaces are a Fase 0 deliverable)

```text
mcgit
│
├── Authentication          (Microsoft OAuth now; TruthID abstraction later)
├── Minecraft Version Manager
├── Java Manager
├── Instance Manager
├── Mod Manager
├── Modpack Manager          (Modrinth / CurseForge — see Legal & Licensing)
├── Skin Manager
├── World Manager
├── Git Engine                (the original "mcgit" from v1.0 — snapshot/restore/branch)
├── Backup Engine
├── Arweave Storage            (StorageProvider abstraction — Fase 7)
├── TruthID Integration         (AuthenticationProvider abstraction — Fase 7, no early coupling)
└── Game Runner                  (process launch, JVM args, cross-platform process management)
```

Two abstraction seams matter architecturally from day one, even though only one branch of each
is implemented early:

```text
StorageProvider
├── LocalStorage      (MVP)
├── CloudStorage       (Fase 5)
└── ArweaveStorage      (Fase 7)

AuthenticationProvider
├── Microsoft          (MVP)
└── TruthID              (Fase 7 — must not require early coupling to ship the MVP)
```

Local metadata (instances, accounts, worlds, mods, modpacks, Java installations, backups, Git
repos, Arweave uploads, skins, settings) lives in a local database (tentatively SQLite — see
`ARCHITECTURE.md`, this is not yet a locked decision). The database never stores world file
content itself — the filesystem (and Git, for versioned worlds) stays the source of truth for
actual Minecraft data.

---

## Security Requirements

Carried over from v1.0, expanded for the launcher's larger attack surface:

* Never silently delete a snapshot; confirm destructive operations.
* Detect an open world when possible; create a safety backup before any restore.
* Never overwrite data without a recovery strategy; verify integrity before restoring remote data.
* Never store the Microsoft account password — OAuth only.
* Never store private keys or tokens in plaintext; use the OS-native secure storage (Android
  Keystore / iOS Secure Enclave / Windows credential manager / Linux Keyring — same pattern
  TruthID already uses for its own device keys).
* Anything uploaded to a cloud or Arweave backup target must support local encryption before
  upload — **the server/cloud side should never need to know the content of the user's private
  data.**
* Provide a clear way to avoid accidentally publishing private world data to Arweave's
  permanent storage.
* Never expose credentials/private keys in logs.

---

## Cross-Platform Requirements

Linux, Windows, and macOS from day one — Linux is a first-class target, not an afterthought.
The architecture must abstract: OS differences, filesystem path conventions, Java discovery,
process management, and credential storage, so no code path silently assumes a Windows-only
layout.

---

## Non Goals

* Being a Minecraft server host (running servers on mcgit's own infrastructure) — mcgit can
  *manage* a locally/self-hosted server instance (Fase 9), but does not host one for the user.
* Building a proprietary mod repository — integrate with existing ecosystems (Modrinth,
  CurseForge) rather than compete with them.
* Implementing a custom blockchain, token, or wallet (TruthID already owns that layer).
* Cloud-first design — mcgit must be local-first and fully usable offline; Cloud/Arweave/TruthID
  are opt-in layers, never requirements to install, play, or version a world.
* Governance token, NFT marketplace, DAO functionality, cryptocurrency exchange (inherited
  Non-Goals from TruthID's own PRD — mcgit's Arweave/TruthID integration must not reintroduce
  these through the back door).

---

## Legal & Licensing Considerations (new — flag before building, do not assume)

This is a Fase 0 research item, not a decision — listed here so it isn't silently skipped:

* **Microsoft/Mojang requirements for third-party launchers — researched (Sessão 2,
  2026-08-15)**: the OAuth flow itself (MS OAuth → Xbox Live → XSTS → Minecraft Services) is
  correct as already documented in `ARCHITECTURE.md`, but the `XboxLive.signin` OAuth scope
  required to authenticate against Xbox Live is **restricted** — a normal Azure app registration
  does not get it automatically (attempting to use it returns a 403 "Invalid app registration").
  Getting access requires formal enrollment in the **Xbox Developer Program via ID@Xbox** (the
  independent/small-developer track) — the same path open-source launchers like Prism Launcher
  and MultiMC had to go through; no alternative payment path for hobbyist projects was found.
  This is a two-step, not one-step, action: (1) create the Azure app registration itself —
  doable any time, no approval needed — and (2) separately request `XboxLive.signin` access via
  ID@Xbox, which involves human review and unknown turnaround time. Tracked as **`PENDING.md`
  #1** since it's an external dependency outside our control that can block real login testing
  before Fase 1 code is otherwise ready for it. Sources: [Microsoft Q&A — XboxLive.signin
  permission](https://learn.microsoft.com/en-gb/answers/questions/5768276/how-to-get-xboxlive-signin-permission-for-azure-ap),
  [HeliosLauncher MicrosoftAuth.md](https://github.com/dscalzi/HeliosLauncher/blob/master/docs/MicrosoftAuth.md).
* **CurseForge API terms of service — researched (Sessão 2, 2026-08-15)**: worse than "pending
  ToS approval" implied. Two hard restrictions in the actual 3rd Party API Terms and Conditions:
  (1) **forbids saving or caching any data obtained through the API** — in direct tension with
  mcgit's local-first/offline-first principles (`GUIDELINES.md` #6/#7), since a launcher
  normally caches mod/modpack metadata locally to work offline and avoid hammering the API; (2)
  **forbids competing, directly or indirectly, with CurseForge/the Platform** — vague enough to
  create real classification risk for a generic launcher. API key approval is not automatic: a
  form + human review by Overwolf, judged on author-earnings impact, infrastructure load, and
  author consent for outside-CurseForge distribution. Historical precedent: when CurseForge
  launched its official API in May 2022, it *removed* modpack-download capability that launchers
  like MultiMC and PCL2 previously had — the rules have already broken existing launchers once.
  **Open decision, not yet made**: whether CurseForge support becomes fully secondary/optional to
  Modrinth (see below), or whether a no-cache-compliant integration mode is worth building.
  Sources: [CurseForge 3rd Party API Terms and
  Conditions](https://support.curseforge.com/support/solutions/articles/9000207405-curse-forge-3rd-party-api-terms-and-conditions),
  [About the CurseForge API and How to Apply for a
  Key](https://support.curseforge.com/support/solutions/articles/9000208346-about-the-curseforge-api-and-how-to-apply-for-a-key).
  Modrinth's API/ToS is comparatively open — see next item. This may affect *whether* and *how*
  CurseForge integration ships, not just *when*.
* **Modrinth API — researched (Sessão 2, 2026-08-15)**: open API, no caching/competition
  restrictions (contrast with CurseForge above). Rate limit 300 req/min regardless of token.
  Only real requirement is a uniquely-identifying `User-Agent` header (contact info recommended,
  not required — helps Modrinth warn before blocking instead of just blocking). Other launchers
  (ATLauncher, GDLauncher) have hit 429s exporting large instances — design the mcgit HTTP
  client with pagination/backoff from the start, not as a later optimization. No ToS blocker for
  Modrinth-first, confirming the existing decision. Source: [Modrinth API
  docs](https://docs.modrinth.com/api/).
* **Mod/modpack redistribution**: mods and modpacks are third-party copyrighted content;
  installing them via API (pointing at the source, not rehosting) is the safe default pattern
  used by other launchers — must not assume rehosting is fine without checking each platform's
  terms.
* **Skins API usage — researched (Sessão 2, 2026-08-15)**: must go through official
  Mojang/Microsoft APIs and respect their rules — no scraping, no unauthorized storage. Note:
  this API (`POST https://api.minecraftservices.com/minecraft/profile/skins`) is **not
  officially documented** by Mojang/Microsoft — it's the same endpoint the official launcher
  uses internally, but public documentation is community reverse-engineering (wiki.vg and
  similar), not a published contract. Accepts `multipart/form-data` (file upload) or
  `application/json` (skin URL), bearer-token auth (same token as the rest of the Minecraft
  Services flow). Rate limit is tight — **~20 req/min** — and repeated 429s (e.g. from retrying
  without backoff) can lead to **temporary account suspension**, not just a UI error. Doesn't
  block code the way CurseForge's explicit ToS does, but demands client-side rate limiting and
  backoff-on-retry from the first commit that touches it. Sources: [Mojang API —
  wiki.vg](https://wiki.vg/Mojang_API), [Mojang API — Minecraft
  Wiki](https://minecraft.wiki/w/Mojang_API).

* **Naming/branding — researched (Sessão 2, 2026-08-15)**: Mojang's Usage Guidelines say
  "Minecraft" cannot be the first word or the dominant part of a third-party product's name.
  **"mcgit" already complies** — no rename needed. Domain names may include a Minecraft brand
  term as long as they don't look official. Source: [Minecraft Usage
  Guidelines](https://www.minecraft.net/en-us/usage-guidelines).

None of this blocks Fase 0 research or documentation work, but it does block writing any code
that touches Microsoft auth or CurseForge until reviewed (see `PENDING.md` #1 for the Microsoft
auth action item). Skins and Modrinth code are not ToS-blocked, but must follow the rate-limiting
notes above from the first implementation.

---

## Monetization (unchanged from v1.0)

The core tool (local versioning, launcher, CLI) is open source (MIT) and will remain so,
matching TruthID's protocol-is-free stance. A business may be built on top in the future — e.g.
managed hosting, a managed remote-backup/sync service, or a marketplace/social layer (see
`PHASE.md` Fase 10) — without changing the open-source nature of the core tool itself. See
`ROADMAP.md` for current thinking; no concrete plan exists yet.
