import { useCallback, useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { listInstances, type Instance } from "../../api/instance";
import {
  abortWorldMerge,
  createWorldBranch,
  createWorldSnapshot,
  deleteWorldSnapshot,
  diffWorldBranches,
  diffWorldRegionChunks,
  disableWorldVersioning,
  enableWorldVersioning,
  finishWorldMerge,
  listWorldBranches,
  listWorldHistory,
  listWorlds,
  mergeWorldBranch,
  previewWorldMerge,
  resolveWorldMergeConflict,
  restoreWorldVersion,
  switchWorldBranch,
  type Branch,
  type ChunkDiff,
  type FileChange,
  type Snapshot,
  type World,
} from "../../api/world";
import { WorldList } from "../world/WorldList";
import type { MergeState } from "../world/WorldBranches";

export function InstanceDetailScreen() {
  const { id } = useParams<{ id: string }>();
  const instanceId = Number(id);

  const [instance, setInstance] = useState<Instance | null>(null);
  const [worlds, setWorlds] = useState<World[]>([]);
  const [historyByWorld, setHistoryByWorld] = useState<Record<string, Snapshot[] | undefined>>({});
  const [branchesByWorld, setBranchesByWorld] = useState<Record<string, Branch[] | undefined>>({});
  const [diffsByWorld, setDiffsByWorld] = useState<
    Record<string, { otherBranch: string; changes: FileChange[] } | undefined>
  >({});
  const [mergeStateByWorld, setMergeStateByWorld] = useState<Record<string, MergeState | undefined>>({});
  const [regionChunkDiffByWorld, setRegionChunkDiffByWorld] = useState<
    Record<string, { otherBranch: string; path: string; chunks: ChunkDiff[] } | undefined>
  >({});
  const [error, setError] = useState<string | null>(null);
  const [status, setStatus] = useState<string | null>(null);

  useEffect(() => {
    listInstances()
      .then((instances) => setInstance(instances.find((i) => i.id === instanceId) ?? null))
      .catch((err) => setError(String(err)));
  }, [instanceId]);

  const refreshWorlds = useCallback(async () => {
    try {
      setWorlds(await listWorlds(instanceId));
    } catch (err) {
      setError(String(err));
    }
  }, [instanceId]);

  useEffect(() => {
    refreshWorlds();
  }, [refreshWorlds]);

  async function handleToggleVersioning(folderName: string, enable: boolean) {
    setError(null);
    setStatus(null);
    try {
      if (enable) {
        await enableWorldVersioning(instanceId, folderName);
      } else {
        await disableWorldVersioning(instanceId, folderName);
      }
      await refreshWorlds();
    } catch (err) {
      setError(String(err));
    }
  }

  async function handleSaveSnapshot(folderName: string, message: string) {
    setError(null);
    setStatus(null);
    try {
      const result = await createWorldSnapshot(instanceId, folderName, message);
      setStatus(result.created ? "Snapshot saved." : "Nothing changed since the last snapshot.");
      if (result.created && historyByWorld[folderName] !== undefined) {
        await handleShowHistory(folderName);
      }
      // A snapshot changes the current branch's tip (and, on the very first
      // snapshot, is what makes the branch exist as a real ref at all) —
      // keep the branches panel and any open comparison in sync if open.
      if (result.created && branchesByWorld[folderName] !== undefined) {
        await handleShowBranches(folderName);
      }
      if (result.created && diffsByWorld[folderName] !== undefined) {
        await handleCompareBranch(folderName, diffsByWorld[folderName]!.otherBranch);
      }
    } catch (err) {
      setError(String(err));
    }
  }

  async function handleShowHistory(folderName: string) {
    setError(null);
    try {
      const history = await listWorldHistory(instanceId, folderName);
      setHistoryByWorld((prev) => ({ ...prev, [folderName]: history }));
    } catch (err) {
      setError(String(err));
    }
  }

  async function handleRestore(folderName: string, hash: string) {
    setError(null);
    setStatus(null);
    try {
      const result = await restoreWorldVersion(instanceId, folderName, hash);
      const short = hash.slice(0, 7);
      if (!result.restored) {
        setStatus(`Already at this version (${short}).`);
      } else if (result.backup_created) {
        setStatus(`Created a backup and restored to ${short}.`);
      } else {
        setStatus(`Restored to ${short}.`);
      }
      await handleShowHistory(folderName);
    } catch (err) {
      setError(String(err));
    }
  }

  async function handleDelete(folderName: string, hash: string) {
    setError(null);
    setStatus(null);
    try {
      await deleteWorldSnapshot(instanceId, folderName, hash);
      setStatus(`Deleted snapshot ${hash.slice(0, 7)}.`);
      await handleShowHistory(folderName);
    } catch (err) {
      setError(String(err));
    }
  }

  async function handleShowBranches(folderName: string) {
    setError(null);
    try {
      const branches = await listWorldBranches(instanceId, folderName);
      setBranchesByWorld((prev) => ({ ...prev, [folderName]: branches }));
    } catch (err) {
      setError(String(err));
    }
  }

  async function handleCreateBranch(folderName: string, name: string) {
    setError(null);
    setStatus(null);
    try {
      const branches = await createWorldBranch(instanceId, folderName, name);
      setBranchesByWorld((prev) => ({ ...prev, [folderName]: branches }));
      setStatus(`Created and switched to branch "${name}".`);
      // The current branch just changed, so any open comparison or merge
      // preview (computed against the old current branch) is no longer
      // meaningful.
      setDiffsByWorld((prev) => ({ ...prev, [folderName]: undefined }));
      setRegionChunkDiffByWorld((prev) => ({ ...prev, [folderName]: undefined }));
      setMergeStateByWorld((prev) => ({ ...prev, [folderName]: undefined }));
    } catch (err) {
      setError(String(err));
    }
  }

  async function handleSwitchBranch(folderName: string, name: string) {
    setError(null);
    setStatus(null);
    try {
      const result = await switchWorldBranch(instanceId, folderName, name);
      setStatus(
        result.checkpoint_created
          ? `Checkpointed pending changes and switched to "${result.branch}".`
          : `Switched to "${result.branch}".`,
      );
      await handleShowBranches(folderName);
      // git log follows HEAD, so switching branches changes what the
      // history panel would show — keep it in sync if it was already open.
      if (historyByWorld[folderName] !== undefined) {
        await handleShowHistory(folderName);
      }
      // The current branch just changed, so any open comparison or merge
      // preview (computed against the old current branch) is no longer
      // meaningful.
      setDiffsByWorld((prev) => ({ ...prev, [folderName]: undefined }));
      setRegionChunkDiffByWorld((prev) => ({ ...prev, [folderName]: undefined }));
      setMergeStateByWorld((prev) => ({ ...prev, [folderName]: undefined }));
    } catch (err) {
      setError(String(err));
    }
  }

  async function handleCompareBranch(folderName: string, otherBranch: string) {
    setError(null);
    try {
      const changes = await diffWorldBranches(instanceId, folderName, otherBranch);
      setDiffsByWorld((prev) => ({ ...prev, [folderName]: { otherBranch, changes } }));
    } catch (err) {
      setError(String(err));
    }
  }

  async function handleShowRegionChunks(folderName: string, otherBranch: string, path: string) {
    setError(null);
    try {
      const chunks = await diffWorldRegionChunks(instanceId, folderName, otherBranch, path);
      setRegionChunkDiffByWorld((prev) => ({ ...prev, [folderName]: { otherBranch, path, chunks } }));
    } catch (err) {
      setError(String(err));
    }
  }

  async function handlePreviewMerge(folderName: string, otherBranch: string) {
    setError(null);
    try {
      const conflictingFiles = await previewWorldMerge(instanceId, folderName, otherBranch);
      setMergeStateByWorld((prev) => ({
        ...prev,
        [folderName]: { phase: "preview", otherBranch, conflictingFiles },
      }));
    } catch (err) {
      setError(String(err));
    }
  }

  function handleCancelMergePreview(folderName: string) {
    setMergeStateByWorld((prev) => ({ ...prev, [folderName]: undefined }));
  }

  async function handleMerge(folderName: string, otherBranch: string) {
    setError(null);
    setStatus(null);
    try {
      const outcome = await mergeWorldBranch(instanceId, folderName, otherBranch);
      if (outcome.kind === "Merged") {
        setStatus(`Merged "${otherBranch}".`);
        setMergeStateByWorld((prev) => ({ ...prev, [folderName]: undefined }));
        await handleShowBranches(folderName);
        if (historyByWorld[folderName] !== undefined) {
          await handleShowHistory(folderName);
        }
      } else {
        setStatus(
          `${outcome.conflicts.length} file${outcome.conflicts.length === 1 ? "" : "s"} need to be resolved before the merge can finish.`,
        );
        setMergeStateByWorld((prev) => ({
          ...prev,
          [folderName]: { phase: "resolving", otherBranch, conflicts: outcome.conflicts },
        }));
      }
    } catch (err) {
      setError(String(err));
    }
  }

  async function handleResolveConflict(folderName: string, path: string, keep: "ours" | "theirs") {
    setError(null);
    try {
      await resolveWorldMergeConflict(instanceId, folderName, path, keep);
      setMergeStateByWorld((prev) => {
        const state = prev[folderName];
        if (!state || state.phase !== "resolving") return prev;
        return {
          ...prev,
          [folderName]: {
            ...state,
            conflicts: state.conflicts.filter((c) => c.path !== path),
          },
        };
      });
    } catch (err) {
      setError(String(err));
    }
  }

  async function handleFinishMerge(folderName: string) {
    setError(null);
    setStatus(null);
    const state = mergeStateByWorld[folderName];
    if (!state) return;
    try {
      await finishWorldMerge(instanceId, folderName, `Merge branch '${state.otherBranch}'`);
      setStatus(`Merged "${state.otherBranch}".`);
      setMergeStateByWorld((prev) => ({ ...prev, [folderName]: undefined }));
      await handleShowBranches(folderName);
      if (historyByWorld[folderName] !== undefined) {
        await handleShowHistory(folderName);
      }
    } catch (err) {
      setError(String(err));
    }
  }

  async function handleAbortMerge(folderName: string) {
    setError(null);
    setStatus(null);
    try {
      await abortWorldMerge(instanceId, folderName);
      setStatus("Merge aborted — the world is back to how it was before.");
      setMergeStateByWorld((prev) => ({ ...prev, [folderName]: undefined }));
    } catch (err) {
      setError(String(err));
    }
  }

  return (
    <section>
      <p>
        <Link to="/">← Instances</Link>
      </p>
      <h1>{instance ? instance.name : `Instance ${instanceId}`}</h1>
      {error && <p style={{ color: "crimson" }}>{error}</p>}
      {status && <p>{status}</p>}

      <h2>Worlds</h2>
      <WorldList
        worlds={worlds}
        historyByWorld={historyByWorld}
        branchesByWorld={branchesByWorld}
        diffsByWorld={diffsByWorld}
        regionChunkDiffByWorld={regionChunkDiffByWorld}
        mergeStateByWorld={mergeStateByWorld}
        onToggleVersioning={handleToggleVersioning}
        onSaveSnapshot={handleSaveSnapshot}
        onShowHistory={handleShowHistory}
        onRestore={handleRestore}
        onDelete={handleDelete}
        onShowBranches={handleShowBranches}
        onCreateBranch={handleCreateBranch}
        onSwitchBranch={handleSwitchBranch}
        onCompareBranch={handleCompareBranch}
        onShowRegionChunks={handleShowRegionChunks}
        onPreviewMerge={handlePreviewMerge}
        onCancelMergePreview={handleCancelMergePreview}
        onMerge={handleMerge}
        onResolveConflict={handleResolveConflict}
        onFinishMerge={handleFinishMerge}
        onAbortMerge={handleAbortMerge}
      />
    </section>
  );
}
