import { create } from "zustand";
import type { Point, Vector } from "../coords";
import { Rotation, rotateCw } from "../coords";
import { applyZoom, snapToGrid } from "../geometry";
import { elementHit } from "../hit";
import type { MapElement, Wall } from "../types";

export type MapMode = "view" | "edit";

export type EditorState = {
  mode: MapMode;
  zoom: number;
  offset: Vector;
  elements: Map<string, MapElement>;
  startElements: Map<string, MapElement>;
  selected: Set<string>;
};

export type EditorActions = {
  setMode: (m: MapMode) => void;
  toggleMode: () => void;

  setOffset: (o: Vector) => void;
  panBy: (delta: Vector) => void;
  zoomTo: (factor: number, cursor: Point) => void;

  loadElements: (els: MapElement[]) => void;
  insertElement: (el: MapElement) => void;
  addAtViewportCenter: (factory: (snapped: Point) => MapElement) => void;
  finishWallDraw: (start: Point, end: Point) => void;

  toggleSelect: (id: string) => void;
  selectByHit: (worldPoint: Point) => string | null;
  clearSelection: () => void;
  deleteSelection: () => void;
  duplicateSelection: () => void;
  rotateSelection: () => void;
  moveSelection: (delta: Vector) => void;

  getChanges: () => { deleted: string[]; updated: MapElement[] };
};

export type EditorStore = EditorState & EditorActions;

function makeId(): string {
  return crypto.randomUUID();
}

function moveElementBy(el: MapElement, delta: Vector): MapElement {
  switch (el.kind) {
    case "wall":
      return {
        ...el,
        start: { x: el.start.x + delta.x, y: el.start.y + delta.y },
        end: { x: el.end.x + delta.x, y: el.end.y + delta.y },
      };
    case "door":
    case "seat":
      return {
        ...el,
        position: {
          x: el.position.x + delta.x,
          y: el.position.y + delta.y,
        },
      };
  }
}

function rotateElement(el: MapElement): MapElement {
  switch (el.kind) {
    case "wall":
      return el;
    case "door":
    case "seat":
      return { ...el, rotation: rotateCw(el.rotation) };
  }
}

function duplicateElement(el: MapElement): MapElement {
  const newId = makeId();
  const delta: Vector = { x: 10, y: 10 };
  switch (el.kind) {
    case "wall":
      return {
        ...el,
        id: newId,
        start: { x: el.start.x + delta.x, y: el.start.y + delta.y },
        end: { x: el.end.x + delta.x, y: el.end.y + delta.y },
      };
    case "door":
    case "seat":
      return {
        ...el,
        id: newId,
        position: {
          x: el.position.x + delta.x,
          y: el.position.y + delta.y,
        },
      };
  }
}

function elementsEqual(a: MapElement, b: MapElement): boolean {
  if (a.kind !== b.kind || a.id !== b.id) return false;
  if (a.kind === "wall" && b.kind === "wall") {
    return (
      a.start.x === b.start.x &&
      a.start.y === b.start.y &&
      a.end.x === b.end.x &&
      a.end.y === b.end.y
    );
  }
  if ((a.kind === "door" || a.kind === "seat") && a.kind === b.kind) {
    const sameBase =
      a.position.x === b.position.x &&
      a.position.y === b.position.y &&
      a.rotation === b.rotation;
    if (!sameBase) return false;
    if (a.kind === "seat" && b.kind === "seat") {
      return a.ip === b.ip;
    }
    return true;
  }
  return false;
}

export function createEditorStore() {
  return create<EditorStore>((set, get) => ({
    mode: "view",
    zoom: 1,
    offset: { x: 0, y: 0 },
    elements: new Map(),
    startElements: new Map(),
    selected: new Set(),

    setMode: (m) => set({ mode: m }),
    toggleMode: () =>
      set((s) =>
        s.mode === "edit"
          ? { mode: "view", selected: new Set() }
          : { mode: "edit" },
      ),

    setOffset: (o) => set({ offset: o }),
    panBy: (delta) =>
      set((s) => ({
        offset: { x: s.offset.x + delta.x, y: s.offset.y + delta.y },
      })),
    zoomTo: (factor, cursor) =>
      set((s) => {
        const r = applyZoom(s.zoom, factor, cursor, s.offset);
        return { zoom: r.zoom, offset: r.offset };
      }),

    loadElements: (els) => {
      const m = new Map<string, MapElement>();
      for (const e of els) m.set(e.id, e);
      set({ elements: m, startElements: new Map(m), selected: new Set() });
    },
    insertElement: (el) =>
      set((s) => {
        const m = new Map(s.elements);
        m.set(el.id, el);
        return { elements: m };
      }),
    addAtViewportCenter: (factory) =>
      set((s) => {
        // Mirrors iced: pos = -offset + (200, 200), then snap.
        const p = snapToGrid({
          x: -s.offset.x + 200,
          y: -s.offset.y + 200,
        });
        const el = factory(p);
        const m = new Map(s.elements);
        m.set(el.id, el);
        return { elements: m };
      }),
    finishWallDraw: (start, end) =>
      set((s) => {
        const wall: Wall = {
          kind: "wall",
          id: makeId(),
          start,
          end,
        };
        const m = new Map(s.elements);
        m.set(wall.id, wall);
        return { elements: m };
      }),

    toggleSelect: (id) =>
      set((s) => {
        const next = new Set(s.selected);
        if (next.has(id)) next.delete(id);
        else next.add(id);
        return { selected: next };
      }),
    selectByHit: (worldPoint) => {
      for (const el of get().elements.values()) {
        if (elementHit(el, worldPoint)) return el.id;
      }
      return null;
    },
    clearSelection: () => set({ selected: new Set() }),
    deleteSelection: () =>
      set((s) => {
        const m = new Map(s.elements);
        for (const id of s.selected) m.delete(id);
        return { elements: m, selected: new Set() };
      }),
    duplicateSelection: () =>
      set((s) => {
        const m = new Map(s.elements);
        const next = new Set<string>();
        for (const id of s.selected) {
          const el = m.get(id);
          if (!el) continue;
          const dup = duplicateElement(el);
          m.set(dup.id, dup);
          next.add(dup.id);
        }
        return { elements: m, selected: next };
      }),
    rotateSelection: () =>
      set((s) => {
        const m = new Map(s.elements);
        for (const id of s.selected) {
          const el = m.get(id);
          if (el) m.set(id, rotateElement(el));
        }
        return { elements: m };
      }),
    moveSelection: (delta) =>
      set((s) => {
        const m = new Map(s.elements);
        for (const id of s.selected) {
          const el = m.get(id);
          if (el) m.set(id, moveElementBy(el, delta));
        }
        return { elements: m };
      }),

    getChanges: () => {
      const { startElements, elements } = get();
      const deleted: string[] = [];
      for (const id of startElements.keys()) {
        if (!elements.has(id)) deleted.push(id);
      }
      const updated: MapElement[] = [];
      for (const [id, el] of elements) {
        const prev = startElements.get(id);
        if (!prev || !elementsEqual(prev, el)) updated.push(el);
      }
      return { deleted, updated };
    },
  }));
}

export { Rotation };
