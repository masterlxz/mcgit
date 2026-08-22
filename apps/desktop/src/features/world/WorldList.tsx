import { useState } from "react";
import type { Snapshot, World } from "../../api/world";
import { SaveSnapshotForm } from "./SaveSnapshotForm";
import { WorldHistory } from "./WorldHistory";

type Props = {
  worlds: World[];
  historyByWorld: Record<string, Snapshot[] | undefined>;
  onToggleVersioning: (folderName: string, enable: boolean) => void;
  onSaveSnapshot: (folderName: string, message: string) => void;
  onShowHistory: (folderName: string) => void;
  onRestore: (folderName: string, hash: string) => void;
  onDelete: (folderName: string, hash: string) => void;
};

export function WorldList({
  worlds,
  historyByWorld,
  onToggleVersioning,
  onSaveSnapshot,
  onShowHistory,
  onRestore,
  onDelete,
}: Props) {
  const [openHistoryFor, setOpenHistoryFor] = useState<Set<string>>(new Set());

  if (worlds.length === 0) {
    return <p>No worlds yet.</p>;
  }

  function toggleHistory(folderName: string) {
    setOpenHistoryFor((prev) => {
      const next = new Set(prev);
      if (next.has(folderName)) {
        next.delete(folderName);
      } else {
        next.add(folderName);
        onShowHistory(folderName);
      }
      return next;
    });
  }

  return (
    <ul>
      {worlds.map((world) => (
        <li key={world.folder_name}>
          {world.folder_name}
          {world.git_enabled ? (
            <>
              <button onClick={() => onToggleVersioning(world.folder_name, false)}>
                Disable versioning
              </button>
              <SaveSnapshotForm onSave={(message) => onSaveSnapshot(world.folder_name, message)} />
              <button onClick={() => toggleHistory(world.folder_name)}>
                {openHistoryFor.has(world.folder_name) ? "Hide history" : "Show history"}
              </button>
              {openHistoryFor.has(world.folder_name) && (
                <WorldHistory
                  snapshots={historyByWorld[world.folder_name] ?? []}
                  onRestore={(hash) => onRestore(world.folder_name, hash)}
                  onDelete={(hash) => onDelete(world.folder_name, hash)}
                />
              )}
            </>
          ) : (
            <button onClick={() => onToggleVersioning(world.folder_name, true)}>
              Enable versioning
            </button>
          )}
        </li>
      ))}
    </ul>
  );
}
