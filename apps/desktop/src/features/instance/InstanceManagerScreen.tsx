import { useCallback, useEffect, useState } from "react";
import {
  createVanillaInstance,
  listInstances,
  listMcVersions,
  onInstanceInstallProgress,
  type Instance,
  type InstanceInstallProgress,
  type McVersion,
} from "../../api/instance";
import { Modal } from "../../components/Modal";
import { CreateInstanceForm } from "./CreateInstanceForm";
import { InstanceInstallProgressBar } from "./InstanceInstallProgressBar";
import { InstanceList } from "./InstanceList";

export function InstanceManagerScreen() {
  const [instances, setInstances] = useState<Instance[]>([]);
  const [mcVersions, setMcVersions] = useState<McVersion[]>([]);
  const [progress, setProgress] = useState<InstanceInstallProgress | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showCreateForm, setShowCreateForm] = useState(false);

  const refresh = useCallback(async () => {
    try {
      setInstances(await listInstances());
    } catch (err) {
      setError(String(err));
    }
  }, []);

  useEffect(() => {
    refresh();
    listMcVersions(false)
      .then(setMcVersions)
      .catch((err) => setError(String(err)));
  }, [refresh]);

  useEffect(() => {
    const unlisten = onInstanceInstallProgress((event) => {
      setProgress(event.phase === "done" ? null : event);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  async function handleCreate(name: string, mcVersion: string) {
    setBusy(true);
    setError(null);
    try {
      await createVanillaInstance(name, mcVersion);
      await refresh();
      setShowCreateForm(false);
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(false);
      setProgress(null);
    }
  }

  return (
    <section>
      <div className="page-header">
        <h1>Instances</h1>
        <button className="btn-primary" onClick={() => setShowCreateForm(true)}>
          + Add instance
        </button>
      </div>
      {showCreateForm && (
        <Modal title="New instance" onClose={() => setShowCreateForm(false)}>
          <CreateInstanceForm versions={mcVersions} onCreate={handleCreate} disabled={busy} />
        </Modal>
      )}
      {error && <p className="banner banner-error">{error}</p>}

      <InstanceList instances={instances} />

      {progress && <InstanceInstallProgressBar progress={progress} />}
    </section>
  );
}
