import type { Point } from "./coords";
import type { Rotation } from "./coords";

export type Door = {
  kind: "door";
  id: string;
  position: Point;
  rotation: Rotation;
};

export type Seat = {
  kind: "seat";
  id: string;
  position: Point;
  rotation: Rotation;
  ip: string | null;
};

export type MapElement = Door | Seat;

export const SEAT_TABLE_W = 200.0;
export const SEAT_TABLE_H = 80.0;
export const SEAT_CHAIR_ARC_RADIUS = 25.0;
export const SEAT_CHAIR_PROTRUSION = 20.0;

export const DOOR_WIDTH = 100.0;

export function getSeatTotalBounds(s: Seat): { w: number; h: number } {
  const w = SEAT_TABLE_W;
  const h = SEAT_TABLE_H + SEAT_CHAIR_PROTRUSION;
  if (s.rotation === 90 || s.rotation === 270) {
    return { w: h, h: w };
  }
  return { w, h };
}
