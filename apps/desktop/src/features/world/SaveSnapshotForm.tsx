import { useState } from "react";

type Props = {
  onSave: (message: string) => void;
};

export function SaveSnapshotForm({ onSave }: Props) {
  const [message, setMessage] = useState("");

  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        onSave(message.trim() || `Snapshot — ${new Date().toLocaleString()}`);
        setMessage("");
      }}
    >
      <input
        value={message}
        onChange={(e) => setMessage(e.target.value)}
        placeholder="What changed? (optional)"
      />
      <button type="submit">Save snapshot</button>
    </form>
  );
}
