import { useState } from "react";
import { useAdvancedMode } from "../../context/AdvancedModeContext";
import type { BlockCount, Snapshot } from "../../api/world";

const MAX_BLOCK_STATS_SHOWN = 20;

type Props = {
  snapshots: Snapshot[];
  blockStats: { hash: string; stats: BlockCount[] } | undefined;
  onRestore: (hash: string) => void;
  onDelete: (hash: string) => void;
  onShowStats: (hash: string) => void;
};

type Confirming = { hash: string; action: "restore" | "delete" } | null;

export function WorldHistory({ snapshots, blockStats, onRestore, onDelete, onShowStats }: Props) {
  const [confirming, setConfirming] = useState<Confirming>(null);
  const [openStatsFor, setOpenStatsFor] = useState<string | null>(null);
  const { advancedMode } = useAdvancedMode();

  function toggleStats(hash: string) {
    if (openStatsFor === hash) {
      setOpenStatsFor(null);
    } else {
      setOpenStatsFor(hash);
      onShowStats(hash);
    }
  }

  if (snapshots.length === 0) {
    return <p>No snapshots yet.</p>;
  }

  return (
    <ul>
      {snapshots.map((snapshot, index) => {
        const isTip = index === 0;
        const isConfirming = confirming?.hash === snapshot.hash ? confirming.action : null;

        return (
          <li key={snapshot.hash}>
            <code>{advancedMode ? snapshot.hash : snapshot.hash.slice(0, 7)}</code> —{" "}
            {new Date(snapshot.date).toLocaleString()} — {snapshot.message}
            {advancedMode && (
              <>
                {" "}
                — <em>mcgit &lt;mcgit@localhost&gt;</em>
              </>
            )}{" "}
            {isConfirming === "restore" && (
              <>
                <em>This will replace the world's current state.</em>{" "}
                <button onClick={() => setConfirming(null)}>Cancel</button>
                <button
                  onClick={() => {
                    onRestore(snapshot.hash);
                    setConfirming(null);
                  }}
                >
                  Create Backup and Restore
                </button>
              </>
            )}
            {isConfirming === "delete" && (
              <>
                <em>
                  {isTip && snapshots.length > 1
                    ? "This is your most recent snapshot. Deleting it will also reset the world's files to the previous snapshot's state."
                    : isTip
                      ? "This is your only snapshot. Deleting it removes all version history for this world (your current files won't be touched)."
                      : "This will permanently remove this snapshot. This cannot be undone."}
                </em>{" "}
                <button onClick={() => setConfirming(null)}>Cancel</button>
                <button
                  onClick={() => {
                    onDelete(snapshot.hash);
                    setConfirming(null);
                  }}
                >
                  Delete snapshot
                </button>
              </>
            )}
            {!isConfirming && (
              <>
                <button onClick={() => setConfirming({ hash: snapshot.hash, action: "restore" })}>
                  Restore
                </button>
                <button onClick={() => setConfirming({ hash: snapshot.hash, action: "delete" })}>
                  Delete
                </button>
                <button onClick={() => toggleStats(snapshot.hash)}>
                  {openStatsFor === snapshot.hash ? "Hide stats" : "Show stats"}
                </button>
                {openStatsFor === snapshot.hash && (
                  <ul>
                    {blockStats && blockStats.hash === snapshot.hash && blockStats.stats.length === 0 && (
                      <li>
                        <em>No blocks found (empty world, or no region files yet).</em>
                      </li>
                    )}
                    {blockStats &&
                      blockStats.hash === snapshot.hash &&
                      blockStats.stats.slice(0, MAX_BLOCK_STATS_SHOWN).map((block) => (
                        <li key={block.name}>
                          {block.count.toLocaleString()} × {block.name}
                        </li>
                      ))}
                    {blockStats &&
                      blockStats.hash === snapshot.hash &&
                      blockStats.stats.length > MAX_BLOCK_STATS_SHOWN && (
                        <li>
                          <em>...and {blockStats.stats.length - MAX_BLOCK_STATS_SHOWN} more block types.</em>
                        </li>
                      )}
                  </ul>
                )}
              </>
            )}
          </li>
        );
      })}
    </ul>
  );
}
