import type { Point, Vector } from "./coords";

export const GRID_SIZE = 100;
export const SNAP_UNITS = 10;
export const ZOOM_MIN = 0.1;
export const ZOOM_MAX = 2.0;

export function snapToGrid(p: Point, units: number = SNAP_UNITS): Point {
  return {
    x: Math.round(p.x / units) * units,
    y: Math.round(p.y / units) * units,
  };
}

export function screenToWorld(
  screen: Point,
  offset: Vector,
  zoom: number,
): Point {
  return {
    x: (screen.x - offset.x) / zoom,
    y: (screen.y - offset.y) / zoom,
  };
}

export function worldToScreen(
  world: Point,
  offset: Vector,
  zoom: number,
): Point {
  return {
    x: world.x * zoom + offset.x,
    y: world.y * zoom + offset.y,
  };
}

export function clampZoom(zoom: number): number {
  return Math.min(ZOOM_MAX, Math.max(ZOOM_MIN, zoom));
}

export function applyZoom(
  oldZoom: number,
  factor: number,
  cursor: Point,
  offset: Vector,
): { zoom: number; offset: Vector } {
  const newZoom = clampZoom(oldZoom * factor);
  const actualFactor = newZoom / oldZoom;
  return {
    zoom: newZoom,
    offset: {
      x: cursor.x - (cursor.x - offset.x) * actualFactor,
      y: cursor.y - (cursor.y - offset.y) * actualFactor,
    },
  };
}
