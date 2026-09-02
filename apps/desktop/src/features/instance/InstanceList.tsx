import { Link } from "react-router-dom";
import type { Instance } from "../../api/instance";

type Props = {
  instances: Instance[];
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
      <p className="card-hint">Available after Microsoft login</p>
    </>
  );
}

export function InstanceList({ instances }: Props) {
  if (instances.length === 0) {
    return <p>No instances yet.</p>;
  }

  return (
    <ul className="card-grid">
      {instances.map((instance) => (
        <li key={instance.id} className="card">
          <Link to={`/instances/${instance.id}`}>
            <h3>{instance.name}</h3>
          </Link>
          <p className="card-meta">
            Minecraft {instance.mc_version} ({instance.loader})
          </p>
          <PlayButton instance={instance} />
        </li>
      ))}
    </ul>
  );
}
