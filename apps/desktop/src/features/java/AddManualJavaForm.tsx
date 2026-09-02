import { useState } from "react";

type Props = {
  onAdd: (path: string) => void;
};

export function AddManualJavaForm({ onAdd }: Props) {
  const [path, setPath] = useState("");

  return (
    <form
      className="inline-form"
      onSubmit={(e) => {
        e.preventDefault();
        if (path.trim()) {
          onAdd(path.trim());
          setPath("");
        }
      }}
    >
      <label>Point at an existing Java installation:</label>
      <input
        value={path}
        onChange={(e) => setPath(e.target.value)}
        placeholder="/path/to/java"
      />
      <button type="submit">Add</button>
    </form>
  );
}
