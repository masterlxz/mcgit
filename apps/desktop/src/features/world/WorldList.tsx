import type { World } from "../../api/world";
import { SaveSnapshotForm } from "./SaveSnapshotForm";

type Props = {
  worlds: World[];
  onToggleVersioning: (folderName: string, enable: boolean) => void;
  onSaveSnapshot: (folderName: string, message: string) => void;
};

export function WorldList({ worlds, onToggleVersioning, onSaveSnapshot }: Props) {
  if (worlds.length === 0) {
    return <p>No worlds yet.</p>;
  }

  return (
    <ul>
      {worlds.map((world) => (
        <li key={world.folder_name}>
          {world.folder_name}
          {world.git_enabled ? (
            <>
              <button onClick={() => onToggleVersioning(world.folder_name, false)}>
                Disable versioning
              </button>
              <SaveSnapshotForm onSave={(message) => onSaveSnapshot(world.folder_name, message)} />
            </>
          ) : (
            <button onClick={() => onToggleVersioning(world.folder_name, true)}>
              Enable versioning
            </button>
          )}
        </li>
      ))}
    </ul>
  );
}
