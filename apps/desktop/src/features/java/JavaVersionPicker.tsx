import { useState } from "react";

type Props = {
  versions: number[];
  onInstall: (majorVersion: number) => void;
  disabled: boolean;
};

export function JavaVersionPicker({ versions, onInstall, disabled }: Props) {
  const [selected, setSelected] = useState<number | null>(null);

  if (versions.length === 0) {
    return <p>Loading available Java versions…</p>;
  }

  return (
    <div className="stacked-form">
      <select
        value={selected ?? ""}
        onChange={(e) => setSelected(Number(e.target.value))}
      >
        <option value="" disabled>
          Pick a version
        </option>
        {versions.map((version) => (
          <option key={version} value={version}>
            Java {version} (LTS)
          </option>
        ))}
      </select>
      <button
        className="btn-primary"
        disabled={disabled || selected === null}
        onClick={() => selected !== null && onInstall(selected)}
      >
        Install
      </button>
    </div>
  );
}
