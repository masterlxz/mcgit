import { Link } from "react-router-dom";
import type { Instance } from "../../api/instance";

type Props = {
  instances: Instance[];
};

export function InstanceList({ instances }: Props) {
  if (instances.length === 0) {
    return <p>No instances yet.</p>;
  }

  return (
    <ul>
      {instances.map((instance) => (
        <li key={instance.id}>
          <Link to={`/instances/${instance.id}`}>
            <strong>{instance.name}</strong> — Minecraft {instance.mc_version} (
            {instance.loader}) — {instance.status}
          </Link>
        </li>
      ))}
    </ul>
  );
}
