import { useState } from "react";
import type { Branch } from "../../api/world";

type Props = {
  branches: Branch[];
  onCreate: (name: string) => void;
  onSwitch: (name: string) => void;
};

export function WorldBranches({ branches, onCreate, onSwitch }: Props) {
  const [newBranchName, setNewBranchName] = useState("");
  const [confirmingSwitchFor, setConfirmingSwitchFor] = useState<string | null>(null);

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
              <button onClick={() => setConfirmingSwitchFor(branch.name)}>Switch</button>
            )}
          </li>
        ))}
      </ul>
    </div>
  );
}
