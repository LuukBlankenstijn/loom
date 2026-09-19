import { describe, expect, it } from "vitest";
import { Rotation } from "./coords";
import { doorHit, seatHit } from "./hit";
import type { Door, Seat } from "./types";

const seat: Seat = {
  kind: "seat",
  id: "s1",
  position: { x: 100, y: 100 },
  rotation: Rotation.Deg0,
  ip: null,
};

describe("seatHit", () => {
  it("hits within the table and chair bounds", () => {
    expect(seatHit(seat, { x: 150, y: 150 })).toBe(true);
    expect(seatHit(seat, { x: 100, y: 100 })).toBe(true);
    expect(seatHit(seat, { x: 300, y: 200 })).toBe(true);
  });

  it("misses outside the bounds", () => {
    expect(seatHit(seat, { x: 99, y: 150 })).toBe(false);
    expect(seatHit(seat, { x: 301, y: 150 })).toBe(false);
    expect(seatHit(seat, { x: 150, y: 201 })).toBe(false);
  });

  it("rotated 90 swaps width and height", () => {
    const rotated: Seat = { ...seat, rotation: Rotation.Deg90 };
    expect(seatHit(rotated, { x: 200, y: 300 })).toBe(true);
    expect(seatHit(rotated, { x: 201, y: 300 })).toBe(false);
  });
});

const door: Door = {
  kind: "door",
  id: "d1",
  position: { x: 0, y: 0 },
  rotation: Rotation.Deg0,
};

describe("doorHit", () => {
  it("hits on both jambs", () => {
    expect(doorHit(door, { x: -50, y: 0 })).toBe(true);
    expect(doorHit(door, { x: 50, y: 0 })).toBe(true);
  });

  it("hits inside the swing area", () => {
    expect(doorHit(door, { x: 0, y: -100 })).toBe(true);
  });

  it("misses points outside the swing area", () => {
    expect(doorHit(door, { x: 51, y: 0 })).toBe(false);
    expect(doorHit(door, { x: 0, y: 1 })).toBe(false);
    expect(doorHit(door, { x: 0, y: -101 })).toBe(false);
  });
});
