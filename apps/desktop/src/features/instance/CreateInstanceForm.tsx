import { useState } from "react";
import type { McVersion } from "../../api/instance";

type Props = {
  versions: McVersion[];
  onCreate: (name: string, mcVersion: string) => void;
  disabled: boolean;
};

export function CreateInstanceForm({ versions, onCreate, disabled }: Props) {
  const [name, setName] = useState("");
  const [selectedVersion, setSelectedVersion] = useState<string | null>(null);

  if (versions.length === 0) {
    return <p>Loading available Minecraft versions…</p>;
  }

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (name.trim() === "" || selectedVersion === null) {
      return;
    }
    onCreate(name.trim(), selectedVersion);
    setName("");
  }

  return (
    <form className="stacked-form" onSubmit={handleSubmit}>
      <input
        type="text"
        placeholder="Instance name"
        value={name}
        onChange={(e) => setName(e.target.value)}
        disabled={disabled}
      />
      <select
        value={selectedVersion ?? ""}
        onChange={(e) => setSelectedVersion(e.target.value)}
        disabled={disabled}
      >
        <option value="" disabled>
          Pick a Minecraft version
        </option>
        {versions.map((version) => (
          <option key={version.id} value={version.id}>
            {version.id}
          </option>
        ))}
      </select>
      <button
        type="submit"
        className="btn-primary"
        disabled={disabled || name.trim() === "" || selectedVersion === null}
      >
        Create instance
      </button>
    </form>
  );
}
