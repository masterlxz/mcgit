import { useState } from "react";
import type { Snapshot } from "../../api/world";

type Props = {
  snapshots: Snapshot[];
  onRestore: (hash: string) => void;
};

export function WorldHistory({ snapshots, onRestore }: Props) {
  const [confirmingHash, setConfirmingHash] = useState<string | null>(null);

  if (snapshots.length === 0) {
    return <p>No snapshots yet.</p>;
  }

  return (
    <ul>
      {snapshots.map((snapshot) => (
        <li key={snapshot.hash}>
          <code>{snapshot.hash.slice(0, 7)}</code> — {new Date(snapshot.date).toLocaleString()} —{" "}
          {snapshot.message}{" "}
          {confirmingHash === snapshot.hash ? (
            <>
              <em>This will replace the world's current state.</em>{" "}
              <button onClick={() => setConfirmingHash(null)}>Cancel</button>
              <button
                onClick={() => {
                  onRestore(snapshot.hash);
                  setConfirmingHash(null);
                }}
              >
                Create Backup and Restore
              </button>
            </>
          ) : (
            <button onClick={() => setConfirmingHash(snapshot.hash)}>Restore</button>
          )}
        </li>
      ))}
    </ul>
  );
}
