import { useState } from "react";
import type {
  BlockDiff,
  Branch,
  ChunkDiff,
  ConflictedFile,
  EntityDiff,
  FileChange,
  StructureDiff,
} from "../../api/world";
import { Modal } from "../../components/Modal";
import { RegionChunkMap } from "./RegionChunkMap";

const MAX_BLOCK_DIFFS_SHOWN = 50;

export type MergeState =
  | { phase: "preview"; otherBranch: string; conflictingFiles: string[] }
  | { phase: "resolving"; otherBranch: string; conflicts: ConflictedFile[] };

type Props = {
  branches: Branch[];
  diff: { otherBranch: string; changes: FileChange[] } | undefined;
  regionChunkDiff: { otherBranch: string; path: string; chunks: ChunkDiff[] } | undefined;
  chunkBlockDiff:
    | { otherBranch: string; path: string; chunkX: number; chunkZ: number; blocks: BlockDiff[] }
    | undefined;
  chunkStructureDiff:
    | { otherBranch: string; path: string; chunkX: number; chunkZ: number; structures: StructureDiff[] }
    | undefined;
  chunkEntityDiff:
    | { otherBranch: string; path: string; chunkX: number; chunkZ: number; entities: EntityDiff[] }
    | undefined;
  mergeState: MergeState | undefined;
  onCreate: (name: string) => void;
  onSwitch: (name: string) => void;
  onCompare: (name: string) => void;
  onShowRegionChunks: (otherBranch: string, path: string) => void;
  onShowChunkBlocks: (otherBranch: string, path: string, chunkX: number, chunkZ: number) => void;
  onShowChunkStructures: (otherBranch: string, path: string, chunkX: number, chunkZ: number) => void;
  onShowChunkEntities: (otherBranch: string, path: string, chunkX: number, chunkZ: number) => void;
  onPreviewMerge: (name: string) => void;
  onCancelMergePreview: () => void;
  onMerge: (name: string) => void;
  onResolveConflict: (path: string, keep: "ours" | "theirs") => void;
  onFinishMerge: () => void;
  onAbortMerge: () => void;
};

/// Whether `path` is one of the two chunk-diffable shapes, and which: a
/// `region/` file (blocks + structures live in its chunks) or an
/// `entities/` file (mobs, dropped items — a different chunk NBT root).
/// Covers all three dimension folders — Overworld (`region/...`), Nether
/// (`DIM-1/region/...`), End (`DIM1/region/...`) — since the block-diff
/// pipeline itself only cares about the filename, not which folder it's in.
function regionFileKind(path: string): "blocks" | "entities" | null {
  const match = path.match(/^(?:DIM-1\/|DIM1\/)?(region|entities)\/.*\.mca$/);
  if (!match) return null;
  return match[1] === "region" ? "blocks" : "entities";
}

/// Pulls `(region_x, region_z)` out of a region file's path (e.g.
/// `"region/r.-1.0.mca"` -> `[-1, 0]`), the same `r.<x>.<z>.mca` naming
/// Minecraft itself uses (mirrors `mcgit_world::parse_region_coords` on the
/// Rust side) — needed to place each chunk in its 32×32 map cell.
function parseRegionCoords(path: string): [number, number] | null {
  const filename = path.split("/").pop() ?? path;
  const match = filename.match(/^r\.(-?\d+)\.(-?\d+)\.mca$/);
  if (!match) return null;
  return [Number(match[1]), Number(match[2])];
}

function describeBlockDiff(block: BlockDiff): string {
  return `(${block.x}, ${block.y}, ${block.z}): ${block.from} → ${block.to}`;
}

function describeStructureDiff(structure: StructureDiff): string {
  return `${structure.presence} — ${structure.id}`;
}

function describeEntityDiff(entity: EntityDiff): string {
  return `${entity.presence} — ${entity.id}`;
}

function formatSize(bytes: number | null): string {
  if (bytes === null) return "";
  if (bytes < 1024) return `${bytes} bytes`;
  return `${(bytes / 1024).toFixed(1)} KB`;
}

function describeChange(change: FileChange): string {
  switch (change.status) {
    case "added":
      return `new file, ${formatSize(change.new_size)}`;
    case "deleted":
      return `deleted, was ${formatSize(change.old_size)}`;
    case "modified":
      return `${formatSize(change.old_size)} → ${formatSize(change.new_size)}`;
  }
}

function describeConflictKind(kind: ConflictedFile["kind"]): string {
  switch (kind) {
    case "both_modified":
      return "changed differently on both sides";
    case "deleted_by_us":
      return "deleted here, changed on the other branch";
    case "deleted_by_them":
      return "changed here, deleted on the other branch";
  }
}

export function WorldBranches({
  branches,
  diff,
  regionChunkDiff,
  chunkBlockDiff,
  chunkStructureDiff,
  chunkEntityDiff,
  mergeState,
  onCreate,
  onSwitch,
  onCompare,
  onShowRegionChunks,
  onShowChunkBlocks,
  onShowChunkStructures,
  onShowChunkEntities,
  onPreviewMerge,
  onCancelMergePreview,
  onMerge,
  onResolveConflict,
  onFinishMerge,
  onAbortMerge,
}: Props) {
  const [newBranchName, setNewBranchName] = useState("");
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [confirmingSwitchFor, setConfirmingSwitchFor] = useState<string | null>(null);
  const [openDiffFor, setOpenDiffFor] = useState<string | null>(null);
  const [openChunksFor, setOpenChunksFor] = useState<string | null>(null);
  const [openChunkDetailFor, setOpenChunkDetailFor] = useState<string | null>(null);

  function toggleDiff(name: string) {
    if (openDiffFor === name) {
      setOpenDiffFor(null);
    } else {
      setOpenDiffFor(name);
      onCompare(name);
    }
  }

  function toggleChunks(otherBranch: string, path: string) {
    if (openChunksFor === path) {
      setOpenChunksFor(null);
    } else {
      setOpenChunksFor(path);
      onShowRegionChunks(otherBranch, path);
    }
  }

  /// Opens/closes the per-chunk detail panel. Which data gets fetched
  /// depends on the file's shape: a `region/` chunk holds both blocks and
  /// structures, an `entities/` chunk holds only entities (see
  /// `regionFileKind`).
  function toggleChunkDetail(
    otherBranch: string,
    path: string,
    chunkX: number,
    chunkZ: number,
    kind: "blocks" | "entities",
  ) {
    const key = `${chunkX},${chunkZ}`;
    if (openChunkDetailFor === key) {
      setOpenChunkDetailFor(null);
    } else {
      setOpenChunkDetailFor(key);
      if (kind === "blocks") {
        onShowChunkBlocks(otherBranch, path, chunkX, chunkZ);
        onShowChunkStructures(otherBranch, path, chunkX, chunkZ);
      } else {
        onShowChunkEntities(otherBranch, path, chunkX, chunkZ);
      }
    }
  }

  return (
    <div>
      <div className="page-header">
        <h4>Branches</h4>
        <button className="btn-primary" onClick={() => setShowCreateForm(true)}>
          + New branch
        </button>
      </div>
      {showCreateForm && (
        <Modal title="New branch" onClose={() => setShowCreateForm(false)}>
          <form
            className="stacked-form"
            onSubmit={(e) => {
              e.preventDefault();
              if (!newBranchName.trim()) return;
              onCreate(newBranchName.trim());
              setNewBranchName("");
              setShowCreateForm(false);
            }}
          >
            <input
              value={newBranchName}
              onChange={(e) => setNewBranchName(e.target.value)}
              placeholder="New branch name"
            />
            <button type="submit" className="btn-primary">
              Create branch
            </button>
          </form>
        </Modal>
      )}

      <ul className="branch-list">
        {branches.map((branch) => (
          <li key={branch.name}>
            {branch.is_current ? <strong>{branch.name} (current)</strong> : branch.name}
            {!branch.is_current && confirmingSwitchFor === branch.name && (
              <div className="confirm-box">
                <p>
                  Switching branches changes the world's files. Any pending change on the
                  current branch is checkpointed automatically first.
                </p>
                <div className="toolbar">
                  <button onClick={() => setConfirmingSwitchFor(null)}>Cancel</button>
                  <button
                    className="btn-primary"
                    onClick={() => {
                      onSwitch(branch.name);
                      setConfirmingSwitchFor(null);
                    }}
                  >
                    Checkpoint and Switch
                  </button>
                </div>
              </div>
            )}
            {!branch.is_current && confirmingSwitchFor !== branch.name && (
              <div className="toolbar">
                <button onClick={() => setConfirmingSwitchFor(branch.name)}>Switch</button>
                <button onClick={() => toggleDiff(branch.name)}>
                  {openDiffFor === branch.name ? "Hide compare" : "Compare"}
                </button>
                {!mergeState && (
                  <button onClick={() => onPreviewMerge(branch.name)}>Merge</button>
                )}
              </div>
            )}
            {!branch.is_current && openDiffFor === branch.name && (
              <ul className="subsection">
                {diff && diff.otherBranch === branch.name && diff.changes.length === 0 && (
                  <li>
                    <em>No differences from the current branch.</em>
                  </li>
                )}
                {diff &&
                  diff.otherBranch === branch.name &&
                  diff.changes.map((change) => {
                    const regionKind = change.status === "modified" ? regionFileKind(change.path) : null;
                    const regionCoords = regionKind ? parseRegionCoords(change.path) : null;
                    const thisRegionChunkDiff =
                      regionChunkDiff &&
                      regionChunkDiff.otherBranch === branch.name &&
                      regionChunkDiff.path === change.path
                        ? regionChunkDiff
                        : undefined;
                    const openChunkX = openChunkDetailFor ? Number(openChunkDetailFor.split(",")[0]) : null;
                    const openChunkZ = openChunkDetailFor ? Number(openChunkDetailFor.split(",")[1]) : null;
                    const thisChunkBlockDiff =
                      chunkBlockDiff &&
                      chunkBlockDiff.otherBranch === branch.name &&
                      chunkBlockDiff.path === change.path &&
                      chunkBlockDiff.chunkX === openChunkX &&
                      chunkBlockDiff.chunkZ === openChunkZ
                        ? chunkBlockDiff
                        : undefined;
                    const thisChunkStructureDiff =
                      chunkStructureDiff &&
                      chunkStructureDiff.otherBranch === branch.name &&
                      chunkStructureDiff.path === change.path &&
                      chunkStructureDiff.chunkX === openChunkX &&
                      chunkStructureDiff.chunkZ === openChunkZ
                        ? chunkStructureDiff
                        : undefined;
                    const thisChunkEntityDiff =
                      chunkEntityDiff &&
                      chunkEntityDiff.otherBranch === branch.name &&
                      chunkEntityDiff.path === change.path &&
                      chunkEntityDiff.chunkX === openChunkX &&
                      chunkEntityDiff.chunkZ === openChunkZ
                        ? chunkEntityDiff
                        : undefined;

                    return (
                      <li key={change.path}>
                        {change.status} — {change.path} — {describeChange(change)}
                        {regionKind && regionCoords && (
                          <>
                            {" "}
                            <button onClick={() => toggleChunks(branch.name, change.path)}>
                              {openChunksFor === change.path ? "Hide chunks" : "Show chunks"}
                            </button>
                            {openChunksFor === change.path && thisRegionChunkDiff && (
                              <>
                                {thisRegionChunkDiff.chunks.length === 0 ? (
                                  <p>
                                    <em>No chunks differ (only metadata changed).</em>
                                  </p>
                                ) : (
                                  <RegionChunkMap
                                    chunks={thisRegionChunkDiff.chunks}
                                    regionX={regionCoords[0]}
                                    regionZ={regionCoords[1]}
                                    openChunkKey={openChunkDetailFor}
                                    detailLabel={regionKind === "blocks" ? "blocks and structures" : "entities"}
                                    onToggleChunk={(chunkX, chunkZ) =>
                                      toggleChunkDetail(branch.name, change.path, chunkX, chunkZ, regionKind)
                                    }
                                  />
                                )}
                                {openChunkDetailFor && openChunkX !== null && openChunkZ !== null && (
                                  <div className="subsection">
                                    {regionKind === "blocks" && (
                                      <>
                                        <h4>
                                          Blocks changed in chunk ({openChunkX}, {openChunkZ})
                                        </h4>
                                        <ul>
                                          {thisChunkBlockDiff && thisChunkBlockDiff.blocks.length === 0 && (
                                            <li>
                                              <em>
                                                No blocks differ in shared sections (the change is in
                                                a section only one side has).
                                              </em>
                                            </li>
                                          )}
                                          {thisChunkBlockDiff &&
                                            thisChunkBlockDiff.blocks
                                              .slice(0, MAX_BLOCK_DIFFS_SHOWN)
                                              .map((block) => (
                                                <li key={`${block.x},${block.y},${block.z}`}>
                                                  {describeBlockDiff(block)}
                                                </li>
                                              ))}
                                          {thisChunkBlockDiff &&
                                            thisChunkBlockDiff.blocks.length > MAX_BLOCK_DIFFS_SHOWN && (
                                              <li>
                                                <em>
                                                  ...and{" "}
                                                  {thisChunkBlockDiff.blocks.length - MAX_BLOCK_DIFFS_SHOWN} more.
                                                </em>
                                              </li>
                                            )}
                                        </ul>
                                        <h4>
                                          Structures changed in chunk ({openChunkX}, {openChunkZ})
                                        </h4>
                                        <ul>
                                          {thisChunkStructureDiff && thisChunkStructureDiff.structures.length === 0 && (
                                            <li>
                                              <em>No structures differ.</em>
                                            </li>
                                          )}
                                          {thisChunkStructureDiff &&
                                            thisChunkStructureDiff.structures.map((structure) => (
                                              <li key={structure.id}>{describeStructureDiff(structure)}</li>
                                            ))}
                                        </ul>
                                      </>
                                    )}
                                    {regionKind === "entities" && (
                                      <>
                                        <h4>
                                          Entities changed in chunk ({openChunkX}, {openChunkZ})
                                        </h4>
                                        <ul>
                                          {thisChunkEntityDiff && thisChunkEntityDiff.entities.length === 0 && (
                                            <li>
                                              <em>No entities differ.</em>
                                            </li>
                                          )}
                                          {thisChunkEntityDiff &&
                                            thisChunkEntityDiff.entities.map((entity) => (
                                              <li key={entity.uuid}>{describeEntityDiff(entity)}</li>
                                            ))}
                                        </ul>
                                      </>
                                    )}
                                  </div>
                                )}
                              </>
                            )}
                          </>
                        )}
                      </li>
                    );
                  })}
              </ul>
            )}
            {mergeState &&
              mergeState.otherBranch === branch.name &&
              mergeState.phase === "preview" && (
                <div className="confirm-box">
                  <p>
                    {mergeState.conflictingFiles.length === 0 ? (
                      "No files would conflict — safe to merge."
                    ) : (
                      <>
                        {mergeState.conflictingFiles.length} file
                        {mergeState.conflictingFiles.length === 1 ? "" : "s"} would conflict:{" "}
                        {mergeState.conflictingFiles.join(", ")}. You'll pick one branch's full
                        version of each — the losing side's changes to that whole file are
                        discarded.
                      </>
                    )}
                  </p>
                  <div className="toolbar">
                    <button onClick={onCancelMergePreview}>Cancel</button>
                    <button className="btn-primary" onClick={() => onMerge(branch.name)}>
                      {mergeState.conflictingFiles.length === 0 ? "Merge" : "Merge anyway"}
                    </button>
                  </div>
                </div>
              )}
            {mergeState &&
              mergeState.otherBranch === branch.name &&
              mergeState.phase === "resolving" && (
                <div className="subsection">
                  <h4>Resolving merge with "{branch.name}"</h4>
                  <ul>
                    {mergeState.conflicts.map((conflict) => (
                      <li key={conflict.path}>
                        {conflict.path} — <em>{describeConflictKind(conflict.kind)}</em>
                        <div className="toolbar">
                          <button onClick={() => onResolveConflict(conflict.path, "ours")}>
                            Keep this branch's version
                          </button>
                          <button onClick={() => onResolveConflict(conflict.path, "theirs")}>
                            Keep the other branch's version
                          </button>
                        </div>
                      </li>
                    ))}
                  </ul>
                  <div className="toolbar">
                    <button className="btn-danger" onClick={onAbortMerge}>
                      Abort merge
                    </button>
                    {mergeState.conflicts.length === 0 && (
                      <button className="btn-primary" onClick={onFinishMerge}>
                        Finish merge
                      </button>
                    )}
                  </div>
                </div>
              )}
          </li>
        ))}
      </ul>
    </div>
  );
}
