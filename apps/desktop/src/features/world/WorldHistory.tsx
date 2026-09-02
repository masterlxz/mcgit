import { useState } from "react";
import { useAdvancedMode } from "../../context/AdvancedModeContext";
import type { BlockCount, EntityCount, Snapshot, StructureCount } from "../../api/world";

const MAX_STATS_SHOWN = 20;

type Props = {
  snapshots: Snapshot[];
  blockStats: { hash: string; stats: BlockCount[] } | undefined;
  structureStats: { hash: string; stats: StructureCount[] } | undefined;
  entityStats: { hash: string; stats: EntityCount[] } | undefined;
  onRestore: (hash: string) => void;
  onDelete: (hash: string) => void;
  onShowStats: (hash: string) => void;
};

type Confirming = { hash: string; action: "restore" | "delete" } | null;

/// One stats sub-section (blocks, structures, or entities) under a
/// snapshot's "Show stats" panel — same shape (`name` + `count`), so the
/// list rendering is shared across all three instead of repeated per kind.
function StatsSection({
  title,
  emptyMessage,
  entries,
}: {
  title: string;
  emptyMessage: string;
  entries: { name: string; count: number }[] | undefined;
}) {
  if (entries === undefined) {
    return null;
  }
  return (
    <li>
      <strong>{title}</strong>
      {entries.length === 0 ? (
        <ul>
          <li>
            <em>{emptyMessage}</em>
          </li>
        </ul>
      ) : (
        <ul>
          {entries.slice(0, MAX_STATS_SHOWN).map((entry) => (
            <li key={entry.name}>
              {entry.count.toLocaleString()} × {entry.name}
            </li>
          ))}
          {entries.length > MAX_STATS_SHOWN && (
            <li>
              <em>...and {entries.length - MAX_STATS_SHOWN} more types.</em>
            </li>
          )}
        </ul>
      )}
    </li>
  );
}

export function WorldHistory({
  snapshots,
  blockStats,
  structureStats,
  entityStats,
  onRestore,
  onDelete,
  onShowStats,
}: Props) {
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
                    <StatsSection
                      title="Blocks"
                      emptyMessage="No blocks found (empty world, or no region files yet)."
                      entries={blockStats?.hash === snapshot.hash ? blockStats.stats : undefined}
                    />
                    <StatsSection
                      title="Structures"
                      emptyMessage="No generated structures found."
                      entries={structureStats?.hash === snapshot.hash ? structureStats.stats : undefined}
                    />
                    <StatsSection
                      title="Entities"
                      emptyMessage="No entities found."
                      entries={entityStats?.hash === snapshot.hash ? entityStats.stats : undefined}
                    />
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
