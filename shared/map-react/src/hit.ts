import type { Point } from "./coords";
import { distance, rotationToRadians } from "./coords";
import {
  DOOR_WIDTH,
  getSeatTotalBounds,
  type Door,
  type MapElement,
  type Seat,
  type Wall,
} from "./types";

const WALL_THRESHOLD = 5;
const DOOR_THRESHOLD = 10;

export function wallHit(wall: Wall, p: Point): boolean {
  const { start: a, end: b } = wall;
  const dx = b.x - a.x;
  const dy = b.y - a.y;
  const l2 = dx * dx + dy * dy;
  if (l2 === 0) return distance(p, a) < WALL_THRESHOLD;

  let t = ((p.x - a.x) * dx + (p.y - a.y) * dy) / l2;
  t = Math.max(0, Math.min(1, t));
  const closestX = a.x + t * dx;
  const closestY = a.y + t * dy;
  const dxC = p.x - closestX;
  const dyC = p.y - closestY;
  return dxC * dxC + dyC * dyC < WALL_THRESHOLD * WALL_THRESHOLD;
}

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

  if (
    distance(local, leftJamb) < DOOR_THRESHOLD ||
    distance(local, rightJamb) < DOOR_THRESHOLD
  ) {
    return true;
  }

  const leafStart: Point = { x: -DOOR_WIDTH / 2, y: 0 };
  const leafEnd: Point = { x: -DOOR_WIDTH / 2, y: -DOOR_WIDTH };
  if (distanceToSegment(local, leafStart, leafEnd) < DOOR_THRESHOLD) return true;

  const distToHinge = distance(local, leftJamb);
  const isWithinRadius = Math.abs(distToHinge - DOOR_WIDTH) < DOOR_THRESHOLD;
  const isWithinAngles = local.x >= -DOOR_WIDTH / 2 && local.y <= 0;
  return isWithinRadius && isWithinAngles;
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
    case "wall":
      return wallHit(el, p);
    case "door":
      return doorHit(el, p);
    case "seat":
      return seatHit(el, p);
  }
}
