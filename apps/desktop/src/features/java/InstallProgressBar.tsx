import type { InstallProgress } from "../../api/java";

type Props = {
  progress: InstallProgress;
};

export function InstallProgressBar({ progress }: Props) {
  const percent =
    progress.bytes_total && progress.bytes_total > 0
      ? Math.round(((progress.bytes_done ?? 0) / progress.bytes_total) * 100)
      : null;

  return (
    <div className="progress-block">
      <p>
        Installing Java {progress.major_version} — {progress.stage}
        {percent !== null ? ` (${percent}%)` : ""}
      </p>
      {percent !== null && <progress value={percent} max={100} />}
    </div>
  );
}
