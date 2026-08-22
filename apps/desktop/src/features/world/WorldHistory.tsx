import type { Snapshot } from "../../api/world";

type Props = {
  snapshots: Snapshot[];
};

export function WorldHistory({ snapshots }: Props) {
  if (snapshots.length === 0) {
    return <p>No snapshots yet.</p>;
  }

  return (
    <ul>
      {snapshots.map((snapshot) => (
        <li key={snapshot.hash}>
          <code>{snapshot.hash.slice(0, 7)}</code> — {new Date(snapshot.date).toLocaleString()} —{" "}
          {snapshot.message}
        </li>
      ))}
    </ul>
  );
}
