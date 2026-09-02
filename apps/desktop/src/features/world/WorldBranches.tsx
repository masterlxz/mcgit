import { useState } from "react";
import type { Branch, FileChange } from "../../api/world";

type Props = {
  branches: Branch[];
  diff: { otherBranch: string; changes: FileChange[] } | undefined;
  onCreate: (name: string) => void;
  onSwitch: (name: string) => void;
  onCompare: (name: string) => void;
};

function formatSize(bytes: number | null): string {
  if (bytes === null) return "";
  if (bytes < 1024) return `${bytes} bytes`;
  return `${(bytes / 1024).toFixed(1)} KB`;
}

function describeChange(change: FileChange): string {
  switch (change.status) {
    case "added":
      return `new file, ${formatSize(change.new_size)}`;
    case "deleted":
      return `deleted, was ${formatSize(change.old_size)}`;
    case "modified":
      return `${formatSize(change.old_size)} → ${formatSize(change.new_size)}`;
  }
}

export function WorldBranches({ branches, diff, onCreate, onSwitch, onCompare }: Props) {
  const [newBranchName, setNewBranchName] = useState("");
  const [confirmingSwitchFor, setConfirmingSwitchFor] = useState<string | null>(null);
  const [openDiffFor, setOpenDiffFor] = useState<string | null>(null);

  function toggleDiff(name: string) {
    if (openDiffFor === name) {
      setOpenDiffFor(null);
    } else {
      setOpenDiffFor(name);
      onCompare(name);
    }
  }

  return (
    <div>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          if (!newBranchName.trim()) return;
          onCreate(newBranchName.trim());
          setNewBranchName("");
        }}
      >
        <input
          value={newBranchName}
          onChange={(e) => setNewBranchName(e.target.value)}
          placeholder="New branch name"
        />
        <button type="submit">Create branch</button>
      </form>

      <ul>
        {branches.map((branch) => (
          <li key={branch.name}>
            {branch.is_current ? <strong>{branch.name} (current)</strong> : branch.name}{" "}
            {!branch.is_current && confirmingSwitchFor === branch.name && (
              <>
                <em>
                  Switching branches changes the world's files. Any pending change on the
                  current branch is checkpointed automatically first.
                </em>{" "}
                <button onClick={() => setConfirmingSwitchFor(null)}>Cancel</button>
                <button
                  onClick={() => {
                    onSwitch(branch.name);
                    setConfirmingSwitchFor(null);
                  }}
                >
                  Checkpoint and Switch
                </button>
              </>
            )}
            {!branch.is_current && confirmingSwitchFor !== branch.name && (
              <>
                <button onClick={() => setConfirmingSwitchFor(branch.name)}>Switch</button>
                <button onClick={() => toggleDiff(branch.name)}>
                  {openDiffFor === branch.name ? "Hide compare" : "Compare"}
                </button>
              </>
            )}
            {!branch.is_current && openDiffFor === branch.name && (
              <ul>
                {diff && diff.otherBranch === branch.name && diff.changes.length === 0 && (
                  <li>
                    <em>No differences from the current branch.</em>
                  </li>
                )}
                {diff &&
                  diff.otherBranch === branch.name &&
                  diff.changes.map((change) => (
                    <li key={change.path}>
                      {change.status} — {change.path} — {describeChange(change)}
                    </li>
                  ))}
              </ul>
            )}
          </li>
        ))}
      </ul>
    </div>
  );
}
