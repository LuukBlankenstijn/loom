import { Arc, Circle, Group, Line } from "react-konva";
import type Konva from "konva";
import type { Door as DoorEl } from "../../types";
import { DOOR_WIDTH } from "../../types";

export const DOOR_COLOR = "#fcba30"; // (0.99, 0.73, 0.19)
const SELECTION_COLOR = "rgba(0,128,255,0.3)";

type Props = {
  door: DoorEl;
  selected: boolean;
  scale: number;
  onClick?: (e: Konva.KonvaEventObject<MouseEvent>) => void;
  onContextMenu?: (e: Konva.KonvaEventObject<PointerEvent>) => void;
};

export function Door({ door, selected, scale, onClick, onContextMenu }: Props) {
  const strokeWidth = 2.0 * scale;
  const dotRadius = 3.0;
  const halfW = DOOR_WIDTH / 2;
  const leafX = -halfW;

  return (
    <Group
      x={door.position.x}
      y={door.position.y}
      rotation={door.rotation}
      onClick={onClick}
      onContextMenu={onContextMenu}
    >
      {selected && (
        <>
          <Line
            points={[leafX, 0, leafX, -DOOR_WIDTH]}
            stroke={SELECTION_COLOR}
            strokeWidth={10}
          />
          <Arc
            x={leafX}
            y={0}
            innerRadius={DOOR_WIDTH}
            outerRadius={DOOR_WIDTH}
            angle={90}
            rotation={-90}
            stroke={SELECTION_COLOR}
            strokeWidth={10}
          />
        </>
      )}
      {/* Leaf */}
      <Line
        points={[leafX, 0, leafX, -DOOR_WIDTH]}
        stroke={DOOR_COLOR}
        strokeWidth={strokeWidth}
      />
      {/* Hinge swing arc (dashed) — quarter circle from -90deg to 0deg */}
      <Arc
        x={leafX}
        y={0}
        innerRadius={DOOR_WIDTH}
        outerRadius={DOOR_WIDTH}
        angle={90}
        rotation={-90}
        stroke={DOOR_COLOR}
        strokeWidth={strokeWidth / 2}
        dash={[5, 5]}
      />
      {/* Jamb dots */}
      <Circle x={leafX} y={0} radius={dotRadius} fill={DOOR_COLOR} />
      <Circle x={halfW} y={0} radius={dotRadius} fill={DOOR_COLOR} />
    </Group>
  );
}
