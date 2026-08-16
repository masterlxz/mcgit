import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type Instance = {
  id: number;
  name: string;
  mc_version: string;
  loader: "vanilla";
  loader_version: string | null;
  java_installation_id: number | null;
  status: "installing" | "ready" | "failed";
  created_at: string;
};

export type McVersion = {
  id: string;
  kind: "release" | "snapshot";
  release_time: string;
};

export type InstanceInstallProgress = {
  instance_id: number;
  phase:
    | "resolving_java"
    | "downloading_client"
    | "downloading_libraries"
    | "downloading_assets"
    | "verifying"
    | "done";
  files_done: number | null;
  files_total: number | null;
  bytes_done: number | null;
  bytes_total: number | null;
};

export function listInstances(): Promise<Instance[]> {
  return invoke("list_instances");
}

export function listMcVersions(includeSnapshots: boolean): Promise<McVersion[]> {
  return invoke("list_mc_versions", { includeSnapshots });
}

export function createVanillaInstance(name: string, mcVersion: string): Promise<Instance> {
  return invoke("create_vanilla_instance", { name, mcVersion });
}

export function onInstanceInstallProgress(
  callback: (progress: InstanceInstallProgress) => void,
): Promise<UnlistenFn> {
  return listen<InstanceInstallProgress>("instance://install-progress", (event) => {
    callback(event.payload);
  });
}
