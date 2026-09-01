import { useState } from "react";
import { useAdvancedMode } from "../../context/AdvancedModeContext";
import type { Branch, Snapshot, World } from "../../api/world";
import { SaveSnapshotForm } from "./SaveSnapshotForm";
import { WorldHistory } from "./WorldHistory";
import { WorldBranches } from "./WorldBranches";

type Props = {
  worlds: World[];
  historyByWorld: Record<string, Snapshot[] | undefined>;
  branchesByWorld: Record<string, Branch[] | undefined>;
  onToggleVersioning: (folderName: string, enable: boolean) => void;
  onSaveSnapshot: (folderName: string, message: string) => void;
  onShowHistory: (folderName: string) => void;
  onRestore: (folderName: string, hash: string) => void;
  onDelete: (folderName: string, hash: string) => void;
  onShowBranches: (folderName: string) => void;
  onCreateBranch: (folderName: string, name: string) => void;
  onSwitchBranch: (folderName: string, name: string) => void;
};

export function WorldList({
  worlds,
  historyByWorld,
  branchesByWorld,
  onToggleVersioning,
  onSaveSnapshot,
  onShowHistory,
  onRestore,
  onDelete,
  onShowBranches,
  onCreateBranch,
  onSwitchBranch,
}: Props) {
  const [openHistoryFor, setOpenHistoryFor] = useState<Set<string>>(new Set());
  const [openBranchesFor, setOpenBranchesFor] = useState<Set<string>>(new Set());
  const { advancedMode } = useAdvancedMode();

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

  function toggleBranches(folderName: string) {
    setOpenBranchesFor((prev) => {
      const next = new Set(prev);
      if (next.has(folderName)) {
        next.delete(folderName);
      } else {
        next.add(folderName);
        onShowBranches(folderName);
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
              {advancedMode && (
                <>
                  <button onClick={() => toggleBranches(world.folder_name)}>
                    {openBranchesFor.has(world.folder_name) ? "Hide branches" : "Show branches"}
                  </button>
                  {openBranchesFor.has(world.folder_name) && (
                    <WorldBranches
                      branches={branchesByWorld[world.folder_name] ?? []}
                      onCreate={(name) => onCreateBranch(world.folder_name, name)}
                      onSwitch={(name) => onSwitchBranch(world.folder_name, name)}
                    />
                  )}
                </>
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
