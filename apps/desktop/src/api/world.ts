import { invoke } from "@tauri-apps/api/core";

export type World = {
  folder_name: string;
  git_enabled: boolean;
};

export function listWorlds(instanceId: number): Promise<World[]> {
  return invoke("list_worlds", { instanceId });
}

export function enableWorldVersioning(instanceId: number, folderName: string): Promise<World> {
  return invoke("enable_world_versioning", { instanceId, folderName });
}

export function disableWorldVersioning(instanceId: number, folderName: string): Promise<World> {
  return invoke("disable_world_versioning", { instanceId, folderName });
}

export type SnapshotResult = { created: boolean; commit_hash: string | null };

export function createWorldSnapshot(
  instanceId: number,
  folderName: string,
  message: string,
): Promise<SnapshotResult> {
  return invoke("create_world_snapshot", { instanceId, folderName, message });
}

export type Snapshot = { hash: string; date: string; message: string };

export function listWorldHistory(instanceId: number, folderName: string): Promise<Snapshot[]> {
  return invoke("list_world_history", { instanceId, folderName });
}

export type RestoreResult = { backup_created: boolean; restored: boolean };

export function restoreWorldVersion(
  instanceId: number,
  folderName: string,
  commitHash: string,
): Promise<RestoreResult> {
  return invoke("restore_world_version", { instanceId, folderName, commitHash });
}

export function deleteWorldSnapshot(
  instanceId: number,
  folderName: string,
  commitHash: string,
): Promise<void> {
  return invoke("delete_world_snapshot", { instanceId, folderName, commitHash });
}

export type Branch = { name: string; is_current: boolean };

export function listWorldBranches(instanceId: number, folderName: string): Promise<Branch[]> {
  return invoke("list_world_branches", { instanceId, folderName });
}

export function createWorldBranch(
  instanceId: number,
  folderName: string,
  name: string,
): Promise<Branch[]> {
  return invoke("create_world_branch", { instanceId, folderName, name });
}

export type SwitchResult = { checkpoint_created: boolean; branch: string };

export function switchWorldBranch(
  instanceId: number,
  folderName: string,
  name: string,
): Promise<SwitchResult> {
  return invoke("switch_world_branch", { instanceId, folderName, name });
}

export type FileChange = {
  path: string;
  status: "added" | "modified" | "deleted";
  old_size: number | null;
  new_size: number | null;
};

export function diffWorldBranches(
  instanceId: number,
  folderName: string,
  otherBranch: string,
): Promise<FileChange[]> {
  return invoke("diff_world_branches", { instanceId, folderName, otherBranch });
}

export function previewWorldMerge(
  instanceId: number,
  folderName: string,
  otherBranch: string,
): Promise<string[]> {
  return invoke("preview_world_merge", { instanceId, folderName, otherBranch });
}

export type ConflictedFile = {
  path: string;
  kind: "both_modified" | "deleted_by_us" | "deleted_by_them";
};

export type MergeOutcome =
  | { kind: "Merged"; commit_hash: string }
  | { kind: "ConflictsPending"; conflicts: ConflictedFile[] };

export function mergeWorldBranch(
  instanceId: number,
  folderName: string,
  otherBranch: string,
): Promise<MergeOutcome> {
  return invoke("merge_world_branch", { instanceId, folderName, otherBranch });
}

export function resolveWorldMergeConflict(
  instanceId: number,
  folderName: string,
  path: string,
  keep: "ours" | "theirs",
): Promise<void> {
  return invoke("resolve_world_merge_conflict", { instanceId, folderName, path, keep });
}

export function finishWorldMerge(
  instanceId: number,
  folderName: string,
  message: string,
): Promise<string> {
  return invoke("finish_world_merge", { instanceId, folderName, message });
}

export function abortWorldMerge(instanceId: number, folderName: string): Promise<void> {
  return invoke("abort_world_merge", { instanceId, folderName });
}

export type ChunkDiff = {
  chunk_x: number;
  chunk_z: number;
  status: "added" | "removed" | "changed";
};

export function diffWorldRegionChunks(
  instanceId: number,
  folderName: string,
  otherBranch: string,
  path: string,
): Promise<ChunkDiff[]> {
  return invoke("diff_world_region_chunks", { instanceId, folderName, otherBranch, path });
}

export type BlockDiff = {
  x: number;
  y: number;
  z: number;
  from: string;
  to: string;
};

export type BlockCount = {
  name: string;
  count: number;
};

export function worldBlockStats(
  instanceId: number,
  folderName: string,
  commitHash: string,
): Promise<BlockCount[]> {
  return invoke("world_block_stats", { instanceId, folderName, commitHash });
}

export function diffWorldChunkBlocks(
  instanceId: number,
  folderName: string,
  otherBranch: string,
  path: string,
  chunkX: number,
  chunkZ: number,
): Promise<BlockDiff[]> {
  return invoke("diff_world_chunk_blocks", {
    instanceId,
    folderName,
    otherBranch,
    path,
    chunkX,
    chunkZ,
  });
}
