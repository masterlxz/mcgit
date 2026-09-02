import { useCallback, useEffect, useState } from "react";
import {
  addManualJava,
  installJava,
  listInstallableJavaVersions,
  listJavaInstallations,
  onInstallProgress,
  scanSystemJava,
  setDefaultJava,
  type InstallProgress,
  type JavaInstallation,
} from "../../api/java";
import { AddManualJavaForm } from "./AddManualJavaForm";
import { InstallProgressBar } from "./InstallProgressBar";
import { JavaInstallationList } from "./JavaInstallationList";
import { JavaVersionPicker } from "./JavaVersionPicker";

export function JavaManagerScreen() {
  const [installations, setInstallations] = useState<JavaInstallation[]>([]);
  const [installableVersions, setInstallableVersions] = useState<number[]>([]);
  const [progress, setProgress] = useState<InstallProgress | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      setInstallations(await listJavaInstallations());
    } catch (err) {
      setError(String(err));
    }
  }, []);

  useEffect(() => {
    refresh();
    listInstallableJavaVersions()
      .then(setInstallableVersions)
      .catch((err) => setError(String(err)));
  }, [refresh]);

  useEffect(() => {
    const unlisten = onInstallProgress((event) => {
      setProgress(event.stage === "done" ? null : event);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  async function handleScan() {
    setBusy(true);
    setError(null);
    try {
      setInstallations(await scanSystemJava());
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(false);
    }
  }

  async function handleInstall(majorVersion: number) {
    setBusy(true);
    setError(null);
    try {
      await installJava(majorVersion);
      await refresh();
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(false);
      setProgress(null);
    }
  }

  async function handleSetDefault(id: number) {
    setError(null);
    try {
      await setDefaultJava(id);
      await refresh();
    } catch (err) {
      setError(String(err));
    }
  }

  async function handleAddManual(path: string) {
    setError(null);
    try {
      await addManualJava(path);
      await refresh();
    } catch (err) {
      setError(String(err));
    }
  }

  return (
    <section>
      <h1>Java</h1>
      {error && <p className="banner banner-error">{error}</p>}

      <JavaInstallationList
        installations={installations}
        onSetDefault={handleSetDefault}
      />

      <button onClick={handleScan} disabled={busy}>
        Scan for Java
      </button>

      {progress && <InstallProgressBar progress={progress} />}

      <hr className="section-divider" />
      <h2>Install a Java version</h2>
      <JavaVersionPicker
        versions={installableVersions}
        onInstall={handleInstall}
        disabled={busy}
      />

      <h2>Add manually</h2>
      <AddManualJavaForm onAdd={handleAddManual} />
    </section>
  );
}
