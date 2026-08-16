import type { InstanceInstallProgress } from "../../api/instance";

type Props = {
  progress: InstanceInstallProgress;
};

const PHASE_LABELS: Record<InstanceInstallProgress["phase"], string> = {
  resolving_java: "Resolving Java",
  downloading_client: "Downloading Minecraft",
  downloading_libraries: "Downloading libraries",
  downloading_assets: "Downloading assets",
  verifying: "Verifying",
  done: "Done",
};

export function InstanceInstallProgressBar({ progress }: Props) {
  const percent =
    progress.bytes_total && progress.bytes_total > 0
      ? Math.round(((progress.bytes_done ?? 0) / progress.bytes_total) * 100)
      : null;

  const fileCount =
    progress.files_total !== null ? ` (${progress.files_done ?? 0}/${progress.files_total} files)` : "";

  return (
    <div>
      <p>
        {PHASE_LABELS[progress.phase]}
        {fileCount}
        {percent !== null ? ` — ${percent}%` : ""}
      </p>
      {percent !== null && <progress value={percent} max={100} />}
    </div>
  );
}
