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
  return (
    p.x >= door.position.x - DOOR_WIDTH / 2 &&
    p.x <= door.position.x + DOOR_WIDTH / 2 &&
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


export function doorOverlap(door: Door, TL: Point, BR: Point): boolean {
    const seatTL={x: door.position.x - DOOR_WIDTH / 2, y: door.position.y - DOOR_WIDTH}
    const seatBR={x: door.position.x + DOOR_WIDTH / 2, y: door.position.y}


    return (!(
        (seatTL.x > BR.x || TL.x > seatBR.x) || 
        (seatBR.y < TL.y || BR.y < seatTL.y) 
    ));
}
export function seatOverlap(seat: Seat, TL: Point, BR: Point): boolean {
    const { w, h } = getSeatTotalBounds(seat);
    const seatTL={x: seat.position.x, y: seat.position.y}
    const seatBR={x: seat.position.x + w, y: seat.position.y + h}

    return (!(
        (seatTL.x > BR.x || TL.x > seatBR.x) || 
        (seatBR.y < TL.y || BR.y < seatTL.y) 
    ));
}

export function elementHit(el: MapElement, p: Point): boolean {
  switch (el.kind) {
    case "door":
      return doorHit(el, p);
    case "seat":
      return seatHit(el, p);
  }
}

export function elementOverlap(el: MapElement, TL: Point, BR: Point): boolean {
  switch (el.kind) {
    case "door":
      return doorOverlap(el, TL, BR);
    case "seat":
      return seatOverlap(el, TL, BR);
  }
}
