import { useEffect, useMemo, useRef, useState } from "react";
import { Layer, Line, Rect, Stage } from "react-konva";
import type Konva from "konva";
import { Rotation } from "../coords";
import { screenToWorld, snapToGrid } from "../geometry";
import { getSeatTotalBounds, type Door as DoorEl, type MapElement, type Seat as SeatEl } from "../types";
import { createEditorStore } from "../store/editorStore";
import { Grid } from "./Grid";
import { Hud } from "./Hud";
import { Door } from "./elements/Door";
import { Seat } from "./elements/Seat";
import { Wall } from "./elements/Wall";

export type MapChanges = {
  deleted: string[];
  updated: MapElement[];
};

export type MapEditorProps = {
  initialElements: MapElement[];
  onSave: (changes: MapChanges) => Promise<void>;
};

type DragState =
  | { kind: "none" }
  | { kind: "panning"; lastScreen: { x: number; y: number } }
  | { kind: "drawing"; start: { x: number; y: number } }
  | { kind: "moving"; lastWorld: { x: number; y: number } };

export function MapEditor({ initialElements, onSave }: MapEditorProps) {
  const useStore = useMemo(() => createEditorStore(), []);
  const state = useStore();

  const containerRef = useRef<HTMLDivElement | null>(null);
  const [size, setSize] = useState({ w: 0, h: 0 });
  const dragRef = useRef<DragState>({ kind: "none" });
  const clickStartRef = useRef<{ x: number; y: number } | null>(null);
  const modifiersRef = useRef({ shift: false, alt: false });
  const [cursorPos, setCursorPos] = useState<{ x: number; y: number } | null>(
    null,
  );
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Load initial elements
  useEffect(() => {
    state.loadElements(initialElements);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [initialElements]);

  // Resize
  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    const ro = new ResizeObserver((entries) => {
      const e = entries[0];
      if (!e) return;
      setSize({ w: e.contentRect.width, h: e.contentRect.height });
    });
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  // Modifiers + keyboard shortcuts (edit mode only)
  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      modifiersRef.current.shift = e.shiftKey;
      modifiersRef.current.alt = e.altKey;
      const target = e.target as HTMLElement | null;
      if (
        target &&
        (target.tagName === "INPUT" ||
          target.tagName === "TEXTAREA" ||
          target.isContentEditable)
      ) {
        return;
      }
      const s = useStore.getState();
      if (s.mode !== "edit") return;
      switch (e.key) {
        case "Delete":
          s.deleteSelection();
          break;
        case "Escape":
          s.clearSelection();
          break;
        case "c":
          s.duplicateSelection();
          break;
        case "r":
          s.rotateSelection();
          break;
      }
    };
    const onKeyUp = (e: KeyboardEvent) => {
      modifiersRef.current.shift = e.shiftKey;
      modifiersRef.current.alt = e.altKey;
    };
    window.addEventListener("keydown", onKeyDown);
    window.addEventListener("keyup", onKeyUp);
    return () => {
      window.removeEventListener("keydown", onKeyDown);
      window.removeEventListener("keyup", onKeyUp);
    };
  }, [useStore]);

  const screenToWorldNow = (screen: { x: number; y: number }) => {
    const s = useStore.getState();
    return screenToWorld(screen, s.offset, s.zoom);
  };

  const onMouseDown = (e: Konva.KonvaEventObject<MouseEvent>) => {
    const stage = e.target.getStage();
    if (!stage) return;
    const pos = stage.getPointerPosition();
    if (!pos) return;
    const s = useStore.getState();
    const canEdit = s.mode === "edit";

    if (e.evt.button === 0) {
      clickStartRef.current = pos;
      if (canEdit && modifiersRef.current.shift) {
        const world = screenToWorldNow(pos);
        dragRef.current = { kind: "drawing", start: snapToGrid(world) };
      } else if (canEdit && modifiersRef.current.alt) {
        dragRef.current = { kind: "moving", lastWorld: screenToWorldNow(pos) };
      } else {
        dragRef.current = { kind: "panning", lastScreen: pos };
      }
    } else if (e.evt.button === 2 && canEdit) {
      e.evt.preventDefault();
      const world = screenToWorldNow(pos);
      const id = s.selectByHit(world);
      if (id) s.toggleSelect(id);
    }
  };

  const onMouseMove = (e: Konva.KonvaEventObject<MouseEvent>) => {
    const stage = e.target.getStage();
    if (!stage) return;
    const pos = stage.getPointerPosition();
    if (!pos) return;
    setCursorPos(pos);

    const drag = dragRef.current;
    const s = useStore.getState();
    if (drag.kind === "panning") {
      const dx = pos.x - drag.lastScreen.x;
      const dy = pos.y - drag.lastScreen.y;
      dragRef.current = { kind: "panning", lastScreen: pos };
      s.panBy({ x: dx, y: dy });
    } else if (drag.kind === "moving") {
      const currentWorld = screenToWorldNow(pos);
      const lastSnapped = snapToGrid(drag.lastWorld);
      const currentSnapped = snapToGrid(currentWorld);
      const delta = {
        x: currentSnapped.x - lastSnapped.x,
        y: currentSnapped.y - lastSnapped.y,
      };
      if (delta.x !== 0 || delta.y !== 0) {
        s.moveSelection(delta);
        dragRef.current = { kind: "moving", lastWorld: currentWorld };
      }
    }
  };

  const onMouseUp = (e: Konva.KonvaEventObject<MouseEvent>) => {
    const stage = e.target.getStage();
    const pos = stage?.getPointerPosition() ?? null;
    const start = clickStartRef.current;
    const drag = dragRef.current;
    dragRef.current = { kind: "none" };
    clickStartRef.current = null;

    const s = useStore.getState();

    if (drag.kind === "drawing" && pos) {
      const end = snapToGrid(screenToWorldNow(pos));
      const startWorld = drag.start;
      const dx = startWorld.x - end.x;
      const dy = startWorld.y - end.y;
      if (Math.sqrt(dx * dx + dy * dy) > 1) {
        s.finishWallDraw(startWorld, end);
      }
    }

    // emit click if minimal travel
    if (pos && start) {
      const dx = pos.x - start.x;
      const dy = pos.y - start.y;
      if (dx * dx + dy * dy < 1 && drag.kind === "panning") {
        const world = screenToWorldNow(pos);
        const id = s.selectByHit(world);
        if (id) s.toggleSelect(id);
      }
    }
  };

  const onMouseLeave = () => {
    dragRef.current = { kind: "none" };
    clickStartRef.current = null;
  };

  const onWheel = (e: Konva.KonvaEventObject<WheelEvent>) => {
    e.evt.preventDefault();
    const stage = e.target.getStage();
    if (!stage) return;
    const pos = stage.getPointerPosition();
    if (!pos) return;
    const factor = e.evt.deltaY < 0 ? 1.1 : 0.9;
    state.zoomTo(factor, pos);
  };

  const onContextMenu = (e: Konva.KonvaEventObject<PointerEvent>) => {
    e.evt.preventDefault();
  };

  const handleSave = async () => {
    setSaving(true);
    setError(null);
    try {
      await onSave(state.getChanges());
      // After successful save, treat current state as new baseline
      const els = Array.from(state.elements.values());
      state.loadElements(els);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setSaving(false);
    }
  };

  const elementsArray = Array.from(state.elements.values());
  const editing = state.mode === "edit";

  // Ghost wall while drawing
  let ghost: { points: number[] } | null = null;
  if (
    editing &&
    dragRef.current.kind === "drawing" &&
    cursorPos
  ) {
    const start = dragRef.current.start;
    const end = snapToGrid(screenToWorldNow(cursorPos));
    ghost = { points: [start.x, start.y, end.x, end.y] };
  }

  return (
    <div
      ref={containerRef}
      className="w-full h-full relative"
      onContextMenu={(e) => e.preventDefault()}
    >
      <Stage
        width={size.w}
        height={size.h}
        x={state.offset.x}
        y={state.offset.y}
        scaleX={state.zoom}
        scaleY={state.zoom}
        onMouseDown={onMouseDown}
        onMouseMove={onMouseMove}
        onMouseUp={onMouseUp}
        onMouseLeave={onMouseLeave}
        onWheel={onWheel}
        onContextMenu={onContextMenu}
      >
        <Layer listening={false}>
          {editing && (
            <Grid
              width={size.w}
              height={size.h}
              offsetX={state.offset.x}
              offsetY={state.offset.y}
              zoom={state.zoom}
            />
          )}
        </Layer>
        <Layer>
          {elementsArray.map((el) => {
            const scale = 1 / state.zoom;
            const selected = state.selected.has(el.id);
            switch (el.kind) {
              case "wall":
                return <Wall key={el.id} wall={el} selected={selected} scale={scale} />;
              case "door":
                return <Door key={el.id} door={el} selected={selected} scale={scale} />;
              case "seat": {
                const { w, h } = getSeatTotalBounds(el);
                return (
                  <Seat
                    key={el.id}
                    seat={el}
                    selected={selected}
                    scale={scale}
                    overlay={
                      <Rect
                        x={-w / 2}
                        y={-h / 2}
                        width={w}
                        height={h}
                        fill="transparent"
                      />
                    }
                  />
                );
              }
            }
          })}
          {ghost && (
            <Line
              points={ghost.points}
              stroke="rgba(255,255,255,0.5)"
              strokeWidth={1 / state.zoom}
            />
          )}
        </Layer>
      </Stage>

      <Hud
        mode={state.mode}
        onToggleMode={() => state.toggleMode()}
        onDelete={() => state.deleteSelection()}
        onClearSelection={() => state.clearSelection()}
        onDuplicate={() => state.duplicateSelection()}
        onRotate={() => state.rotateSelection()}
        onAddDoor={() =>
          state.addAtViewportCenter((p) => {
            const door: DoorEl = {
              kind: "door",
              id: crypto.randomUUID(),
              position: p,
              rotation: Rotation.Deg0,
            };
            return door;
          })
        }
        onAddSeat={() =>
          state.addAtViewportCenter((p) => {
            const seat: SeatEl = {
              kind: "seat",
              id: crypto.randomUUID(),
              position: p,
              rotation: Rotation.Deg0,
              ip: null,
            };
            return seat;
          })
        }
        onSave={handleSave}
        saving={saving}
        error={error}
        onClearError={() => setError(null)}
      />
    </div>
  );
}
