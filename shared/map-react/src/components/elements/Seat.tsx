import { Arc, Group, Line, Rect } from "react-konva";
import type Konva from "konva";
import type { ReactNode } from "react";
import {
  SEAT_CHAIR_ARC_RADIUS,
  SEAT_CHAIR_PROTRUSION,
  SEAT_TABLE_H,
  SEAT_TABLE_W,
  type Seat as SeatEl,
} from "../../types";

export const SEAT_ACCENT_COLOR = "#4a82c2"; // (0.29, 0.51, 0.76)
const LAPTOP_COLOR = "#4d4d4d"; // (0.3, 0.3, 0.3)
const CHAIR_COLOR = "rgba(128,128,128,0.8)"; // (0.5, 0.5, 0.5, 0.8)
const SELECTION_COLOR = "rgba(0,128,255,0.2)";

type Props = {
  seat: SeatEl;
  selected: boolean;
  scale: number;
  onClick?: (e: Konva.KonvaEventObject<MouseEvent>) => void;
  onContextMenu?: (e: Konva.KonvaEventObject<PointerEvent>) => void;
  overlay?: ReactNode;
};

export function Seat({
  seat,
  selected,
  scale,
  onClick,
  onContextMenu,
  overlay,
}: Props) {
  const totalW = SEAT_TABLE_W;
  const totalH = SEAT_TABLE_H + SEAT_CHAIR_PROTRUSION;
  const verticalShift = SEAT_CHAIR_PROTRUSION / 2;
  const chairY = -SEAT_TABLE_H / 2 + verticalShift - 5;
  const xOffsets = [-65, 0, 65];

  const laptopW = 40;
  const laptopH = 25;

  return (
    <Group
      x={seat.position.x + totalW / 2}
      y={seat.position.y + totalH / 2}
      rotation={seat.rotation}
      onClick={onClick}
      onContextMenu={onContextMenu}
    >
      {selected && (
        <Rect
          x={-SEAT_TABLE_W / 2}
          y={-SEAT_TABLE_H / 2 - SEAT_CHAIR_PROTRUSION + verticalShift}
          width={SEAT_TABLE_W}
          height={SEAT_TABLE_H + SEAT_CHAIR_PROTRUSION}
          fill={SELECTION_COLOR}
        />
      )}
      {xOffsets.map((x) => (
        <Arc
          key={x}
          x={x}
          y={chairY}
          innerRadius={SEAT_CHAIR_ARC_RADIUS}
          outerRadius={SEAT_CHAIR_ARC_RADIUS}
          angle={180}
          rotation={180}
          stroke={CHAIR_COLOR}
          strokeWidth={2.0 * scale}
        />
      ))}
      <Rect
        x={-SEAT_TABLE_W / 2}
        y={-SEAT_TABLE_H / 2 + verticalShift}
        width={SEAT_TABLE_W}
        height={SEAT_TABLE_H}
        stroke={SEAT_ACCENT_COLOR}
        strokeWidth={2.5 * scale}
      />
      <Rect
        x={-laptopW / 2}
        y={-laptopH / 2 + verticalShift}
        width={laptopW}
        height={laptopH}
        stroke={LAPTOP_COLOR}
        strokeWidth={1.5 * scale}
      />
      <Line
        points={[-laptopW / 2, verticalShift, laptopW / 2, verticalShift]}
        stroke={LAPTOP_COLOR}
        strokeWidth={1.0 * scale}
      />
      {overlay}
    </Group>
  );
}
