import { Link } from "react-router-dom";
import type { CSSProperties } from "react";
import type { Instance } from "../../api/instance";

type Props = {
  instances: Instance[];
};

const cardStyle: CSSProperties = {
  border: "1px solid #ccc",
  borderRadius: 8,
  padding: "0.75rem 1rem",
  margin: "0.5rem 0",
  textAlign: "left",
};

function PlayButton({ instance }: { instance: Instance }) {
  if (instance.status === "installing") {
    return <button disabled>Installing…</button>;
  }
  if (instance.status === "failed") {
    return <button disabled>Installation failed</button>;
  }
  return (
    <>
      <button disabled title="Available after Microsoft login">
        Play
      </button>
      <p style={{ fontSize: "0.85em", color: "#666", margin: "0.25rem 0 0" }}>
        Available after Microsoft login
      </p>
    </>
  );
}

export function InstanceList({ instances }: Props) {
  if (instances.length === 0) {
    return <p>No instances yet.</p>;
  }

  return (
    <ul style={{ listStyle: "none", padding: 0 }}>
      {instances.map((instance) => (
        <li key={instance.id} style={cardStyle}>
          <Link to={`/instances/${instance.id}`}>
            <strong>{instance.name}</strong>
          </Link>
          <p style={{ margin: "0.25rem 0" }}>
            Minecraft {instance.mc_version} ({instance.loader})
          </p>
          <PlayButton instance={instance} />
        </li>
      ))}
    </ul>
  );
}
