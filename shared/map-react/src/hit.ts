import type { Point } from "./coords";
import { distance, rotationToRadians } from "./coords";
import {
  DOOR_WIDTH,
  getSeatTotalBounds,
  type Door,
  type MapElement,
  type Seat,
} from "./types";

const DOOR_THRESHOLD = 10;

function distanceToSegment(p: Point, a: Point, b: Point): number {
  const vx = b.x - a.x;
  const vy = b.y - a.y;
  const wx = p.x - a.x;
  const wy = p.y - a.y;
  const c1 = wx * vx + wy * vy;
  const c2 = vx * vx + vy * vy;
  if (c1 <= 0) return distance(p, a);
  if (c2 <= c1) return distance(p, b);
  const tt = c1 / c2;
  return distance(p, { x: a.x + vx * tt, y: a.y + vy * tt });
}

export function doorHit(door: Door, p: Point): boolean {
  // world -> local: undo translation
  const tx = p.x - door.position.x;
  const ty = p.y - door.position.y;

  // undo rotation
  const angle = -rotationToRadians(door.rotation);
  const cos = Math.cos(angle);
  const sin = Math.sin(angle);
  const lx = tx * cos - ty * sin;
  const ly = tx * sin + ty * cos;
  const local: Point = { x: lx, y: ly };

  const leftJamb: Point = { x: -DOOR_WIDTH / 2, y: 0 };
  const rightJamb: Point = { x: DOOR_WIDTH / 2, y: 0 };

  return (
    p.x >= door.position.x - DOOR_WIDTH / 2 &&
    p.x <= door.position.x + DOOR_WIDTH - DOOR_WIDTH / 2 &&
    p.y >= door.position.y - DOOR_WIDTH &&
    p.y <= door.position.y 
  );
}

export function seatHit(seat: Seat, p: Point): boolean {
  const { w, h } = getSeatTotalBounds(seat);
  return (
    p.x >= seat.position.x &&
    p.x <= seat.position.x + w &&
    p.y >= seat.position.y &&
    p.y <= seat.position.y + h
  );
}

export function elementHit(el: MapElement, p: Point): boolean {
  switch (el.kind) {
    case "door":
      return doorHit(el, p);
    case "seat":
      return seatHit(el, p);
  }
}
