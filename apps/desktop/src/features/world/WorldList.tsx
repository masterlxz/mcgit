import { useState } from "react";
import { useAdvancedMode } from "../../context/AdvancedModeContext";
import type { Branch, ChunkDiff, FileChange, Snapshot, World } from "../../api/world";
import { SaveSnapshotForm } from "./SaveSnapshotForm";
import { WorldHistory } from "./WorldHistory";
import { WorldBranches, type MergeState } from "./WorldBranches";

type Props = {
  worlds: World[];
  historyByWorld: Record<string, Snapshot[] | undefined>;
  branchesByWorld: Record<string, Branch[] | undefined>;
  diffsByWorld: Record<string, { otherBranch: string; changes: FileChange[] } | undefined>;
  regionChunkDiffByWorld: Record<
    string,
    { otherBranch: string; path: string; chunks: ChunkDiff[] } | undefined
  >;
  mergeStateByWorld: Record<string, MergeState | undefined>;
  onToggleVersioning: (folderName: string, enable: boolean) => void;
  onSaveSnapshot: (folderName: string, message: string) => void;
  onShowHistory: (folderName: string) => void;
  onRestore: (folderName: string, hash: string) => void;
  onDelete: (folderName: string, hash: string) => void;
  onShowBranches: (folderName: string) => void;
  onCreateBranch: (folderName: string, name: string) => void;
  onSwitchBranch: (folderName: string, name: string) => void;
  onCompareBranch: (folderName: string, otherBranch: string) => void;
  onShowRegionChunks: (folderName: string, otherBranch: string, path: string) => void;
  onPreviewMerge: (folderName: string, otherBranch: string) => void;
  onCancelMergePreview: (folderName: string) => void;
  onMerge: (folderName: string, otherBranch: string) => void;
  onResolveConflict: (folderName: string, path: string, keep: "ours" | "theirs") => void;
  onFinishMerge: (folderName: string) => void;
  onAbortMerge: (folderName: string) => void;
};

export function WorldList({
  worlds,
  historyByWorld,
  branchesByWorld,
  diffsByWorld,
  regionChunkDiffByWorld,
  mergeStateByWorld,
  onToggleVersioning,
  onSaveSnapshot,
  onShowHistory,
  onRestore,
  onDelete,
  onShowBranches,
  onCreateBranch,
  onSwitchBranch,
  onCompareBranch,
  onShowRegionChunks,
  onPreviewMerge,
  onCancelMergePreview,
  onMerge,
  onResolveConflict,
  onFinishMerge,
  onAbortMerge,
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
                      diff={diffsByWorld[world.folder_name]}
                      regionChunkDiff={regionChunkDiffByWorld[world.folder_name]}
                      mergeState={mergeStateByWorld[world.folder_name]}
                      onCreate={(name) => onCreateBranch(world.folder_name, name)}
                      onSwitch={(name) => onSwitchBranch(world.folder_name, name)}
                      onCompare={(name) => onCompareBranch(world.folder_name, name)}
                      onShowRegionChunks={(otherBranch, path) =>
                        onShowRegionChunks(world.folder_name, otherBranch, path)
                      }
                      onPreviewMerge={(name) => onPreviewMerge(world.folder_name, name)}
                      onCancelMergePreview={() => onCancelMergePreview(world.folder_name)}
                      onMerge={(name) => onMerge(world.folder_name, name)}
                      onResolveConflict={(path, keep) =>
                        onResolveConflict(world.folder_name, path, keep)
                      }
                      onFinishMerge={() => onFinishMerge(world.folder_name)}
                      onAbortMerge={() => onAbortMerge(world.folder_name)}
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
