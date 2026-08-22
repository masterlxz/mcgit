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
