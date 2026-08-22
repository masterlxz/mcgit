import { invoke } from "@tauri-apps/api/core";

export type World = {
  folder_name: string;
};

export function listWorlds(instanceId: number): Promise<World[]> {
  return invoke("list_worlds", { instanceId });
}
