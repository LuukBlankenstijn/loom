import { Circle, Group, Line } from "react-konva";
import type Konva from "konva";
import type { Wall as WallEl } from "../../types";

export const WALL_COLOR = "#b02a1c"; // (0.69, 0.16, 0.11)
const SELECTION_COLOR = "rgba(0,128,255,0.3)";

type Props = {
  wall: WallEl;
  selected: boolean;
  scale: number;
  onClick?: (e: Konva.KonvaEventObject<MouseEvent>) => void;
  onContextMenu?: (e: Konva.KonvaEventObject<PointerEvent>) => void;
};

export function Wall({ wall, selected, scale, onClick, onContextMenu }: Props) {
  const strokeWidth = 2.0 * scale;
  const dotRadius = 3.0;
  const points = [wall.start.x, wall.start.y, wall.end.x, wall.end.y];

  return (
    <Group onClick={onClick} onContextMenu={onContextMenu}>
      {selected && (
        <Line points={points} stroke={SELECTION_COLOR} strokeWidth={10} />
      )}
      <Line points={points} stroke={WALL_COLOR} strokeWidth={strokeWidth} />
      <Circle x={wall.start.x} y={wall.start.y} radius={dotRadius} fill={WALL_COLOR} />
      <Circle x={wall.end.x} y={wall.end.y} radius={dotRadius} fill={WALL_COLOR} />
    </Group>
  );
}
