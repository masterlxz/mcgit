import { invoke } from "@tauri-apps/api/core";

export type CommitIdentity = {
  name: string;
  email: string;
};

export function getCommitIdentity(): Promise<CommitIdentity> {
  return invoke("get_commit_identity");
}

export function setCommitIdentity(name: string, email: string): Promise<void> {
  return invoke("set_commit_identity", { name, email });
}
