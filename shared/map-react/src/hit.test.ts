import { describe, expect, it } from "vitest";
import { Rotation } from "./coords";
import { doorHit, seatHit, wallHit } from "./hit";
import type { Door, Seat, Wall } from "./types";

const wall: Wall = {
  kind: "wall",
  id: "w1",
  start: { x: 0, y: 0 },
  end: { x: 100, y: 0 },
};

describe("wallHit", () => {
  it("hits on the segment", () => {
    expect(wallHit(wall, { x: 50, y: 0 })).toBe(true);
    expect(wallHit(wall, { x: 50, y: 4 })).toBe(true);
  });

  it("misses far from segment", () => {
    expect(wallHit(wall, { x: 50, y: 10 })).toBe(false);
    expect(wallHit(wall, { x: -10, y: 0 })).toBe(false);
    expect(wallHit(wall, { x: 110, y: 0 })).toBe(false);
  });

  it("zero-length segment behaves as point", () => {
    const w: Wall = {
      kind: "wall",
      id: "w2",
      start: { x: 10, y: 10 },
      end: { x: 10, y: 10 },
    };
    expect(wallHit(w, { x: 11, y: 11 })).toBe(true);
    expect(wallHit(w, { x: 20, y: 20 })).toBe(false);
  });
});

const seat: Seat = {
  kind: "seat",
  id: "s1",
  position: { x: 100, y: 100 },
  rotation: Rotation.Deg0,
  ip: null,
};

describe("seatHit", () => {
  it("hits within AABB", () => {
    expect(seatHit(seat, { x: 150, y: 150 })).toBe(true);
    expect(seatHit(seat, { x: 100, y: 100 })).toBe(true);
    expect(seatHit(seat, { x: 300, y: 215 })).toBe(true);
  });

  it("misses outside AABB", () => {
    expect(seatHit(seat, { x: 99, y: 150 })).toBe(false);
    expect(seatHit(seat, { x: 301, y: 150 })).toBe(false);
    expect(seatHit(seat, { x: 150, y: 220 })).toBe(false);
  });

  it("rotated 90 swaps width/height", () => {
    const rotated: Seat = { ...seat, rotation: Rotation.Deg90 };
    // total bounds rotate: w=115, h=200
    expect(seatHit(rotated, { x: 100 + 114, y: 100 + 199 })).toBe(true);
    expect(seatHit(rotated, { x: 100 + 116, y: 100 + 199 })).toBe(false);
  });
});

const door: Door = {
  kind: "door",
  id: "d1",
  position: { x: 0, y: 0 },
  rotation: Rotation.Deg0,
};

describe("doorHit", () => {
  it("hits on left jamb", () => {
    expect(doorHit(door, { x: -50, y: 0 })).toBe(true);
  });

  it("hits on right jamb", () => {
    expect(doorHit(door, { x: 50, y: 0 })).toBe(true);
  });

  it("hits on leaf segment", () => {
    expect(doorHit(door, { x: -50, y: -50 })).toBe(true);
  });

  it("hits on hinge arc", () => {
    // arc center is (-50, 0), radius 100; point at (-50, -100) is on arc
    expect(doorHit(door, { x: -50, y: -100 })).toBe(true);
    // point at (50, 0) is right jamb (already hit)
    // point at (0, -100) is on arc (sqrt(50^2+100^2)≈111.8, NOT within radius+thr)
    expect(doorHit(door, { x: 0, y: -100 })).toBe(false);
  });

  it("misses points well outside", () => {
    expect(doorHit(door, { x: 200, y: 200 })).toBe(false);
    expect(doorHit(door, { x: 0, y: 200 })).toBe(false);
  });

  it("respects rotation", () => {
    const rotated: Door = { ...door, rotation: Rotation.Deg90 };
    // After 90deg rotation, what was (-50, 0) in local -> world (0, -50)
    expect(doorHit(rotated, { x: 0, y: -50 })).toBe(true);
    expect(doorHit(rotated, { x: -50, y: 0 })).toBe(false);
  });
});
