import { useState } from "react";
import type { BlockDiff, Branch, ChunkDiff, ConflictedFile, FileChange } from "../../api/world";
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
  mergeState: MergeState | undefined;
  onCreate: (name: string) => void;
  onSwitch: (name: string) => void;
  onCompare: (name: string) => void;
  onShowRegionChunks: (otherBranch: string, path: string) => void;
  onShowChunkBlocks: (otherBranch: string, path: string, chunkX: number, chunkZ: number) => void;
  onPreviewMerge: (name: string) => void;
  onCancelMergePreview: () => void;
  onMerge: (name: string) => void;
  onResolveConflict: (path: string, keep: "ours" | "theirs") => void;
  onFinishMerge: () => void;
  onAbortMerge: () => void;
};

function isRegionFile(path: string): boolean {
  return path.startsWith("region/") && path.endsWith(".mca");
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
  mergeState,
  onCreate,
  onSwitch,
  onCompare,
  onShowRegionChunks,
  onShowChunkBlocks,
  onPreviewMerge,
  onCancelMergePreview,
  onMerge,
  onResolveConflict,
  onFinishMerge,
  onAbortMerge,
}: Props) {
  const [newBranchName, setNewBranchName] = useState("");
  const [confirmingSwitchFor, setConfirmingSwitchFor] = useState<string | null>(null);
  const [openDiffFor, setOpenDiffFor] = useState<string | null>(null);
  const [openChunksFor, setOpenChunksFor] = useState<string | null>(null);
  const [openBlocksFor, setOpenBlocksFor] = useState<string | null>(null);

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

  function toggleBlocks(otherBranch: string, path: string, chunkX: number, chunkZ: number) {
    const key = `${chunkX},${chunkZ}`;
    if (openBlocksFor === key) {
      setOpenBlocksFor(null);
    } else {
      setOpenBlocksFor(key);
      onShowChunkBlocks(otherBranch, path, chunkX, chunkZ);
    }
  }

  return (
    <div>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          if (!newBranchName.trim()) return;
          onCreate(newBranchName.trim());
          setNewBranchName("");
        }}
      >
        <input
          value={newBranchName}
          onChange={(e) => setNewBranchName(e.target.value)}
          placeholder="New branch name"
        />
        <button type="submit">Create branch</button>
      </form>

      <ul>
        {branches.map((branch) => (
          <li key={branch.name}>
            {branch.is_current ? <strong>{branch.name} (current)</strong> : branch.name}{" "}
            {!branch.is_current && confirmingSwitchFor === branch.name && (
              <>
                <em>
                  Switching branches changes the world's files. Any pending change on the
                  current branch is checkpointed automatically first.
                </em>{" "}
                <button onClick={() => setConfirmingSwitchFor(null)}>Cancel</button>
                <button
                  onClick={() => {
                    onSwitch(branch.name);
                    setConfirmingSwitchFor(null);
                  }}
                >
                  Checkpoint and Switch
                </button>
              </>
            )}
            {!branch.is_current && confirmingSwitchFor !== branch.name && (
              <>
                <button onClick={() => setConfirmingSwitchFor(branch.name)}>Switch</button>
                <button onClick={() => toggleDiff(branch.name)}>
                  {openDiffFor === branch.name ? "Hide compare" : "Compare"}
                </button>
                {!mergeState && (
                  <button onClick={() => onPreviewMerge(branch.name)}>Merge</button>
                )}
              </>
            )}
            {!branch.is_current && openDiffFor === branch.name && (
              <ul>
                {diff && diff.otherBranch === branch.name && diff.changes.length === 0 && (
                  <li>
                    <em>No differences from the current branch.</em>
                  </li>
                )}
                {diff &&
                  diff.otherBranch === branch.name &&
                  diff.changes.map((change) => {
                    const regionCoords = isRegionFile(change.path) ? parseRegionCoords(change.path) : null;
                    const thisRegionChunkDiff =
                      regionChunkDiff &&
                      regionChunkDiff.otherBranch === branch.name &&
                      regionChunkDiff.path === change.path
                        ? regionChunkDiff
                        : undefined;
                    const openChunkX = openBlocksFor ? Number(openBlocksFor.split(",")[0]) : null;
                    const openChunkZ = openBlocksFor ? Number(openBlocksFor.split(",")[1]) : null;
                    const thisChunkBlockDiff =
                      chunkBlockDiff &&
                      chunkBlockDiff.otherBranch === branch.name &&
                      chunkBlockDiff.path === change.path &&
                      chunkBlockDiff.chunkX === openChunkX &&
                      chunkBlockDiff.chunkZ === openChunkZ
                        ? chunkBlockDiff
                        : undefined;

                    return (
                      <li key={change.path}>
                        {change.status} — {change.path} — {describeChange(change)}
                        {change.status === "modified" && regionCoords && (
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
                                    openChunkKey={openBlocksFor}
                                    onToggleChunk={(chunkX, chunkZ) =>
                                      toggleBlocks(branch.name, change.path, chunkX, chunkZ)
                                    }
                                  />
                                )}
                                {openBlocksFor && openChunkX !== null && openChunkZ !== null && (
                                  <div>
                                    <p>
                                      Blocks changed in chunk ({openChunkX}, {openChunkZ}):
                                    </p>
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
                <div>
                  {mergeState.conflictingFiles.length === 0 ? (
                    <em>No files would conflict — safe to merge.</em>
                  ) : (
                    <em>
                      {mergeState.conflictingFiles.length} file
                      {mergeState.conflictingFiles.length === 1 ? "" : "s"} would conflict:{" "}
                      {mergeState.conflictingFiles.join(", ")}. You'll pick one branch's full
                      version of each — the losing side's changes to that whole file are
                      discarded.
                    </em>
                  )}{" "}
                  <button onClick={onCancelMergePreview}>Cancel</button>
                  <button onClick={() => onMerge(branch.name)}>
                    {mergeState.conflictingFiles.length === 0 ? "Merge" : "Merge anyway"}
                  </button>
                </div>
              )}
            {mergeState &&
              mergeState.otherBranch === branch.name &&
              mergeState.phase === "resolving" && (
                <div>
                  <p>
                    <em>Resolving merge with "{branch.name}":</em>
                  </p>
                  <ul>
                    {mergeState.conflicts.map((conflict) => (
                      <li key={conflict.path}>
                        {conflict.path} — <em>{describeConflictKind(conflict.kind)}</em>{" "}
                        <button onClick={() => onResolveConflict(conflict.path, "ours")}>
                          Keep this branch's version
                        </button>
                        <button onClick={() => onResolveConflict(conflict.path, "theirs")}>
                          Keep the other branch's version
                        </button>
                      </li>
                    ))}
                  </ul>
                  <button onClick={onAbortMerge}>Abort merge</button>
                  {mergeState.conflicts.length === 0 && (
                    <button onClick={onFinishMerge}>Finish merge</button>
                  )}
                </div>
              )}
          </li>
        ))}
      </ul>
    </div>
  );
}
