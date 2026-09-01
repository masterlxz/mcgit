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
