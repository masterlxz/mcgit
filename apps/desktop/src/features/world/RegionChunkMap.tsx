import type { ChunkDiff } from "../../api/world";

const CELL_SIZE = 11;
const CELL_GAP = 1;

const STATUS_COLOR: Record<ChunkDiff["status"], string> = {
  added: "#2e7d32",
  removed: "#c62828",
  changed: "#f9a825",
};

const STATUS_LABEL: Record<ChunkDiff["status"], string> = {
  added: "Added",
  removed: "Removed",
  changed: "Changed",
};

type Props = {
  chunks: ChunkDiff[];
  regionX: number;
  regionZ: number;
  openChunkKey: string | null;
  onToggleChunk: (chunkX: number, chunkZ: number) => void;
};

/// A 32×32 map of one region file's chunk slots, colored by diff status —
/// the graphical counterpart to the plain `(x, z) status` text list. Only
/// the differing chunks `diff_region_chunks` reports get a color; every
/// other slot (unchanged, or never generated on either side — the two are
/// indistinguishable from this data alone) renders as an empty cell.
export function RegionChunkMap({ chunks, regionX, regionZ, openChunkKey, onToggleChunk }: Props) {
  const byLocalCoord = new Map<string, ChunkDiff>();
  for (const chunk of chunks) {
    const localX = chunk.chunk_x - regionX * 32;
    const localZ = chunk.chunk_z - regionZ * 32;
    byLocalCoord.set(`${localX},${localZ}`, chunk);
  }

  const cells = [];
  for (let localZ = 0; localZ < 32; localZ++) {
    for (let localX = 0; localX < 32; localX++) {
      const chunk = byLocalCoord.get(`${localX},${localZ}`);
      const isChanged = chunk?.status === "changed";
      const key = chunk ? `${chunk.chunk_x},${chunk.chunk_z}` : `empty-${localX},${localZ}`;
      const isOpen = isChanged && openChunkKey === `${chunk!.chunk_x},${chunk!.chunk_z}`;

      cells.push(
        <div
          key={key}
          title={chunk ? `(${chunk.chunk_x}, ${chunk.chunk_z}) ${chunk.status}` : undefined}
          onClick={isChanged ? () => onToggleChunk(chunk!.chunk_x, chunk!.chunk_z) : undefined}
          style={{
            width: CELL_SIZE,
            height: CELL_SIZE,
            background: chunk ? STATUS_COLOR[chunk.status] : "#8883",
            cursor: isChanged ? "pointer" : "default",
            outline: isOpen ? "2px solid currentColor" : undefined,
            outlineOffset: isOpen ? -1 : undefined,
            boxSizing: "border-box",
          }}
        />,
      );
    }
  }

  return (
    <div>
      <div
        style={{
          display: "grid",
          gridTemplateColumns: `repeat(32, ${CELL_SIZE}px)`,
          gap: CELL_GAP,
          width: "fit-content",
        }}
      >
        {cells}
      </div>
      <p style={{ fontSize: "0.85em" }}>
        {(["added", "removed", "changed"] as const).map((status) => (
          <span key={status} style={{ marginRight: 12 }}>
            <span
              style={{
                display: "inline-block",
                width: 10,
                height: 10,
                background: STATUS_COLOR[status],
                marginRight: 4,
                verticalAlign: "middle",
              }}
            />
            {STATUS_LABEL[status]}
          </span>
        ))}
        <em>Click a changed chunk to see its blocks.</em>
      </p>
    </div>
  );
}
