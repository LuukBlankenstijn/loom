export type Point = { x: number; y: number };
export type Vector = { x: number; y: number };

export enum Rotation {
  Deg0 = 0,
  Deg90 = 90,
  Deg180 = 180,
  Deg270 = 270,
}

export function rotateCw(r: Rotation): Rotation {
  switch (r) {
    case Rotation.Deg0:
      return Rotation.Deg90;
    case Rotation.Deg90:
      return Rotation.Deg180;
    case Rotation.Deg180:
      return Rotation.Deg270;
    case Rotation.Deg270:
      return Rotation.Deg0;
  }
}

export function rotationToRadians(r: Rotation): number {
  return (r / 180) * Math.PI;
}

export function addPoint(a: Point, v: Vector): Point {
  return { x: a.x + v.x, y: a.y + v.y };
}

export function subPoint(a: Point, b: Point): Vector {
  return { x: a.x - b.x, y: a.y - b.y };
}

export function distance(a: Point, b: Point): number {
  const dx = a.x - b.x;
  const dy = a.y - b.y;
  return Math.sqrt(dx * dx + dy * dy);
}
