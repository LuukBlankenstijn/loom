import { describe, expect, it } from "vitest";
import {
  applyZoom,
  clampZoom,
  screenToWorld,
  snapToGrid,
  worldToScreen,
} from "./geometry";

describe("snapToGrid", () => {
  it("rounds to nearest 10", () => {
    expect(snapToGrid({ x: 14, y: 16 })).toEqual({ x: 10, y: 20 });
    expect(snapToGrid({ x: -14, y: -16 })).toEqual({ x: -10, y: -20 });
    expect(snapToGrid({ x: 0, y: 0 })).toEqual({ x: 0, y: 0 });
  });
});

describe("screenToWorld / worldToScreen", () => {
  it("are inverses", () => {
    const offset = { x: 100, y: 50 };
    const zoom = 1.5;
    const world = { x: 30, y: 70 };
    const screen = worldToScreen(world, offset, zoom);
    const back = screenToWorld(screen, offset, zoom);
    expect(back.x).toBeCloseTo(world.x);
    expect(back.y).toBeCloseTo(world.y);
  });
});

describe("clampZoom", () => {
  it("clamps to [0.1, 2.0]", () => {
    expect(clampZoom(0.05)).toBe(0.1);
    expect(clampZoom(5)).toBe(2);
    expect(clampZoom(1)).toBe(1);
  });
});

describe("applyZoom", () => {
  it("zoom-to-cursor: world point under cursor is preserved", () => {
    const offset = { x: 0, y: 0 };
    const zoom = 1;
    const cursor = { x: 100, y: 100 };
    const worldUnderCursorBefore = screenToWorld(cursor, offset, zoom);
    const r = applyZoom(zoom, 1.5, cursor, offset);
    const worldUnderCursorAfter = screenToWorld(cursor, r.offset, r.zoom);
    expect(worldUnderCursorAfter.x).toBeCloseTo(worldUnderCursorBefore.x);
    expect(worldUnderCursorAfter.y).toBeCloseTo(worldUnderCursorBefore.y);
  });
});
