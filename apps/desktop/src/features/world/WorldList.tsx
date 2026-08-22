import type { World } from "../../api/world";

type Props = {
  worlds: World[];
  onToggleVersioning: (folderName: string, enable: boolean) => void;
};

export function WorldList({ worlds, onToggleVersioning }: Props) {
  if (worlds.length === 0) {
    return <p>No worlds yet.</p>;
  }

  return (
    <ul>
      {worlds.map((world) => (
        <li key={world.folder_name}>
          {world.folder_name}
          {world.git_enabled ? (
            <button onClick={() => onToggleVersioning(world.folder_name, false)}>
              Disable versioning
            </button>
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
