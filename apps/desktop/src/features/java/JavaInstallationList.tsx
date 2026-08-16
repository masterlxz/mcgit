import type { JavaInstallation } from "../../api/java";

type Props = {
  installations: JavaInstallation[];
  onSetDefault: (id: number) => void;
};

export function JavaInstallationList({ installations, onSetDefault }: Props) {
  if (installations.length === 0) {
    return <p>No Java installations found yet.</p>;
  }

  return (
    <ul>
      {installations.map((installation) => (
        <li key={installation.id}>
          <strong>Java {installation.major_version}</strong> — {installation.vendor} (
          {installation.source}) — <code>{installation.path}</code>
          {installation.is_default ? (
            <span> — default</span>
          ) : (
            <button onClick={() => onSetDefault(installation.id)}>
              Set as default
            </button>
          )}
        </li>
      ))}
    </ul>
  );
}
