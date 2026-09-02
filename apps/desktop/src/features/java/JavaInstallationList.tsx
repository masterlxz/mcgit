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
    <ul className="install-list">
      {installations.map((installation) => (
        <li key={installation.id}>
          <span>
            <strong>Java {installation.major_version}</strong>{" "}
            <span className="install-info">
              — {installation.vendor} ({installation.source}) — <code>{installation.path}</code>
            </span>
          </span>
          {installation.is_default ? (
            <span className="install-default">Default</span>
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
