import { useCallback, useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { listInstances, type Instance } from "../../api/instance";
import {
  createWorldSnapshot,
  disableWorldVersioning,
  enableWorldVersioning,
  listWorlds,
  type World,
} from "../../api/world";
import { WorldList } from "../world/WorldList";

export function InstanceDetailScreen() {
  const { id } = useParams<{ id: string }>();
  const instanceId = Number(id);

  const [instance, setInstance] = useState<Instance | null>(null);
  const [worlds, setWorlds] = useState<World[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [status, setStatus] = useState<string | null>(null);

  useEffect(() => {
    listInstances()
      .then((instances) => setInstance(instances.find((i) => i.id === instanceId) ?? null))
      .catch((err) => setError(String(err)));
  }, [instanceId]);

  const refreshWorlds = useCallback(async () => {
    try {
      setWorlds(await listWorlds(instanceId));
    } catch (err) {
      setError(String(err));
    }
  }, [instanceId]);

  useEffect(() => {
    refreshWorlds();
  }, [refreshWorlds]);

  async function handleToggleVersioning(folderName: string, enable: boolean) {
    setError(null);
    setStatus(null);
    try {
      if (enable) {
        await enableWorldVersioning(instanceId, folderName);
      } else {
        await disableWorldVersioning(instanceId, folderName);
      }
      await refreshWorlds();
    } catch (err) {
      setError(String(err));
    }
  }

  async function handleSaveSnapshot(folderName: string, message: string) {
    setError(null);
    setStatus(null);
    try {
      const result = await createWorldSnapshot(instanceId, folderName, message);
      setStatus(result.created ? "Snapshot saved." : "Nothing changed since the last snapshot.");
    } catch (err) {
      setError(String(err));
    }
  }

  return (
    <section>
      <p>
        <Link to="/">← Instances</Link>
      </p>
      <h1>{instance ? instance.name : `Instance ${instanceId}`}</h1>
      {error && <p style={{ color: "crimson" }}>{error}</p>}
      {status && <p>{status}</p>}

      <h2>Worlds</h2>
      <WorldList
        worlds={worlds}
        onToggleVersioning={handleToggleVersioning}
        onSaveSnapshot={handleSaveSnapshot}
      />
    </section>
  );
}
