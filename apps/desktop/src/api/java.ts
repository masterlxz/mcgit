import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type JavaInstallation = {
  id: number;
  major_version: number;
  vendor: string;
  path: string;
  source: "managed" | "detected" | "manual";
  is_default: boolean;
};

export type InstallProgress = {
  major_version: number;
  stage: "downloading" | "verifying" | "extracting" | "done";
  bytes_done: number | null;
  bytes_total: number | null;
};

export function listJavaInstallations(): Promise<JavaInstallation[]> {
  return invoke("list_java_installations");
}

export function scanSystemJava(): Promise<JavaInstallation[]> {
  return invoke("scan_system_java");
}

export function listInstallableJavaVersions(): Promise<number[]> {
  return invoke("list_installable_java_versions");
}

export function installJava(majorVersion: number): Promise<JavaInstallation> {
  return invoke("install_java", { majorVersion });
}

export function addManualJava(path: string): Promise<JavaInstallation> {
  return invoke("add_manual_java", { path });
}

export function setDefaultJava(id: number): Promise<void> {
  return invoke("set_default_java", { id });
}

export function onInstallProgress(
  callback: (progress: InstallProgress) => void,
): Promise<UnlistenFn> {
  return listen<InstallProgress>("java://install-progress", (event) => {
    callback(event.payload);
  });
}
