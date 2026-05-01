import { useEffect, useRef, useState } from "react";
import { Layer, Rect, Stage } from "react-konva";
import type Konva from "konva";
import { applyZoom } from "../geometry";
import type { MapElement } from "../types";
import {
  DOOR_WIDTH,
  SEAT_CHAIR_PROTRUSION,
  SEAT_TABLE_H,
  SEAT_TABLE_W,
  getSeatTotalBounds,
} from "../types";
import type { SeatOverlayRenderer } from "../decorators/types";
import { Grid } from "./Grid";
import { Door } from "./elements/Door";
import { Seat } from "./elements/Seat";
import { Wall } from "./elements/Wall";

export type MapViewerProps = {
  elements: MapElement[];
  onElementClick?: (el: MapElement) => void;
  seatOverlay?: SeatOverlayRenderer;
  showGrid?: boolean;
  fitToContent?: boolean;
};

type Bounds = { minX: number; minY: number; maxX: number; maxY: number };

function computeBounds(elements: MapElement[]): Bounds | null {
  if (elements.length === 0) return null;
  let minX = Infinity;
  let minY = Infinity;
  let maxX = -Infinity;
  let maxY = -Infinity;
  const expand = (x: number, y: number) => {
    if (x < minX) minX = x;
    if (y < minY) minY = y;
    if (x > maxX) maxX = x;
    if (y > maxY) maxY = y;
  };
  for (const el of elements) {
    if (el.kind === "wall") {
      expand(el.start.x, el.start.y);
      expand(el.end.x, el.end.y);
    } else if (el.kind === "door") {
      const half = DOOR_WIDTH / 2;
      expand(el.position.x - half, el.position.y - half);
      expand(el.position.x + half, el.position.y + half);
    } else {
      const { w, h } = getSeatTotalBounds(el);
      expand(el.position.x - w / 2, el.position.y - h / 2);
      expand(el.position.x + w / 2, el.position.y + h / 2);
    }
  }
  if (!isFinite(minX)) {
    // Fallback for unexpected bounds
    return {
      minX: -SEAT_TABLE_W,
      minY: -SEAT_TABLE_H,
      maxX: SEAT_TABLE_W,
      maxY: SEAT_TABLE_H + SEAT_CHAIR_PROTRUSION,
    };
  }
  return { minX, minY, maxX, maxY };
}

export function MapViewer({
  elements,
  onElementClick,
  seatOverlay,
  showGrid = false,
  fitToContent = false,
}: MapViewerProps) {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const [size, setSize] = useState({ w: 0, h: 0 });
  const [offset, setOffset] = useState({ x: 0, y: 0 });
  const [zoom, setZoom] = useState(0.5);
  const fittedRef = useRef(false);
  const isPanningRef = useRef(false);
  const lastPosRef = useRef<{ x: number; y: number } | null>(null);
  const clickStartRef = useRef<{ x: number; y: number } | null>(null);

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

  useEffect(() => {
    if (!fitToContent) return;
    if (fittedRef.current) return;
    if (size.w === 0 || size.h === 0) return;
    const bounds = computeBounds(elements);
    if (!bounds) return;
    const z = 0.5;
    const cx = (bounds.minX + bounds.maxX) / 2;
    const cy = (bounds.minY + bounds.maxY) / 2;
    setZoom(z);
    setOffset({ x: size.w / 2 - cx * z, y: size.h / 2 - cy * z });
    fittedRef.current = true;
  }, [fitToContent, elements, size.w, size.h]);

  const onMouseDown = (e: Konva.KonvaEventObject<MouseEvent>) => {
    if (e.evt.button !== 0) return;
    const stage = e.target.getStage();
    if (!stage) return;
    const pos = stage.getPointerPosition();
    if (!pos) return;
    isPanningRef.current = true;
    lastPosRef.current = pos;
    clickStartRef.current = pos;
  };

  const onMouseMove = (e: Konva.KonvaEventObject<MouseEvent>) => {
    if (!isPanningRef.current) return;
    const stage = e.target.getStage();
    if (!stage) return;
    const pos = stage.getPointerPosition();
    if (!pos || !lastPosRef.current) return;
    const dx = pos.x - lastPosRef.current.x;
    const dy = pos.y - lastPosRef.current.y;
    lastPosRef.current = pos;
    setOffset((o) => ({ x: o.x + dx, y: o.y + dy }));
  };

  const onMouseUp = () => {
    isPanningRef.current = false;
    lastPosRef.current = null;
  };

  const onWheel = (e: Konva.KonvaEventObject<WheelEvent>) => {
    e.evt.preventDefault();
    const stage = e.target.getStage();
    if (!stage) return;
    const pos = stage.getPointerPosition();
    if (!pos) return;
    const factor = e.evt.deltaY < 0 ? 1.1 : 0.9;
    const next = applyZoom(zoom, factor, pos, offset);
    setZoom(next.zoom);
    setOffset(next.offset);
  };

  const wasDrag = (current: { x: number; y: number } | null) => {
    const start = clickStartRef.current;
    if (!start || !current) return false;
    const dx = current.x - start.x;
    const dy = current.y - start.y;
    return dx * dx + dy * dy >= 1;
  };

  const handleElementClick = (el: MapElement) => {
    return (e: Konva.KonvaEventObject<MouseEvent>) => {
      const stage = e.target.getStage();
      const pos = stage?.getPointerPosition() ?? null;
      if (wasDrag(pos)) return;
      onElementClick?.(el);
    };
  };

  return (
    <div
      ref={containerRef}
      style={{ position: "absolute", inset: 0 }}
    >
      <Stage
        width={size.w}
        height={size.h}
        x={offset.x}
        y={offset.y}
        scaleX={zoom}
        scaleY={zoom}
        onMouseDown={onMouseDown}
        onMouseMove={onMouseMove}
        onMouseUp={onMouseUp}
        onMouseLeave={onMouseUp}
        onWheel={onWheel}
      >
        <Layer listening={false}>
          {showGrid && (
            <Grid
              width={size.w}
              height={size.h}
              offsetX={offset.x}
              offsetY={offset.y}
              zoom={zoom}
            />
          )}
        </Layer>
        <Layer>
          {elements.map((el) => {
            const scale = 1 / zoom;
            switch (el.kind) {
              case "wall":
                return (
                  <Wall
                    key={el.id}
                    wall={el}
                    selected={false}
                    scale={scale}
                    onClick={
                      onElementClick ? handleElementClick(el) : undefined
                    }
                  />
                );
              case "door":
                return (
                  <Door
                    key={el.id}
                    door={el}
                    selected={false}
                    scale={scale}
                    onClick={
                      onElementClick ? handleElementClick(el) : undefined
                    }
                  />
                );
              case "seat": {
                const { w, h } = getSeatTotalBounds(el);
                return (
                  <Seat
                    key={el.id}
                    seat={el}
                    selected={false}
                    scale={scale}
                    onClick={
                      onElementClick ? handleElementClick(el) : undefined
                    }
                    overlay={
                      <>
                        {/* Invisible AABB for full-area hit-testing */}
                        <Rect
                          x={-w / 2}
                          y={-h / 2}
                          width={w}
                          height={h}
                          fill="transparent"
                        />
                        {seatOverlay?.(el)}
                      </>
                    }
                  />
                );
              }
            }
          })}
        </Layer>
      </Stage>
    </div>
  );
}
