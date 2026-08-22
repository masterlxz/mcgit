import type { World } from "../../api/world";

type Props = {
  worlds: World[];
};

export function WorldList({ worlds }: Props) {
  if (worlds.length === 0) {
    return <p>No worlds yet.</p>;
  }

  return (
    <ul>
      {worlds.map((world) => (
        <li key={world.folder_name}>{world.folder_name}</li>
      ))}
    </ul>
  );
}
