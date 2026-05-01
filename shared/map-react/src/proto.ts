import { create } from "@bufbuild/protobuf";
import {
  ElementSchema,
  type Element as ProtoElement,
  Rotation as ProtoRotation,
} from "@client/v1/map/types_pb";
import { Rotation } from "./coords";
import type { MapElement } from "./types";

function protoRotationToCore(r: ProtoRotation): Rotation {
  switch (r) {
    case ProtoRotation.ROTATION_90:
      return Rotation.Deg90;
    case ProtoRotation.ROTATION_180:
      return Rotation.Deg180;
    case ProtoRotation.ROTATION_270:
      return Rotation.Deg270;
    default:
      return Rotation.Deg0;
  }
}

function coreRotationToProto(r: Rotation): ProtoRotation {
  switch (r) {
    case Rotation.Deg90:
      return ProtoRotation.ROTATION_90;
    case Rotation.Deg180:
      return ProtoRotation.ROTATION_180;
    case Rotation.Deg270:
      return ProtoRotation.ROTATION_270;
    case Rotation.Deg0:
      return ProtoRotation.ROTATION_0;
  }
}

export function fromProtoElements(elements: ProtoElement[]): MapElement[] {
  const out: MapElement[] = [];
  for (const e of elements) {
    switch (e.element.case) {
      case "wall": {
        const w = e.element.value;
        if (!w.start || !w.end) continue;
        out.push({
          kind: "wall",
          id: w.id,
          start: { x: w.start.x, y: w.start.y },
          end: { x: w.end.x, y: w.end.y },
        });
        break;
      }
      case "door": {
        const d = e.element.value;
        if (!d.location) continue;
        out.push({
          kind: "door",
          id: d.id,
          position: { x: d.location.x, y: d.location.y },
          rotation: protoRotationToCore(d.rotation),
        });
        break;
      }
      case "seat": {
        const s = e.element.value;
        if (!s.location) continue;
        out.push({
          kind: "seat",
          id: s.id,
          position: { x: s.location.x, y: s.location.y },
          rotation: protoRotationToCore(s.rotation),
          ip: s.ip ?? null,
        });
        break;
      }
    }
  }
  return out;
}

export function toProtoElements(elements: MapElement[]): ProtoElement[] {
  return elements.map((e) => {
    switch (e.kind) {
      case "wall":
        return create(ElementSchema, {
          element: {
            case: "wall",
            value: {
              $typeName: "map.v1.Wall",
              id: e.id,
              start: { $typeName: "map.v1.Location", x: e.start.x, y: e.start.y },
              end: { $typeName: "map.v1.Location", x: e.end.x, y: e.end.y },
            },
          },
        });
      case "door":
        return create(ElementSchema, {
          element: {
            case: "door",
            value: {
              $typeName: "map.v1.Door",
              id: e.id,
              location: {
                $typeName: "map.v1.Location",
                x: e.position.x,
                y: e.position.y,
              },
              rotation: coreRotationToProto(e.rotation),
            },
          },
        });
      case "seat":
        return create(ElementSchema, {
          element: {
            case: "seat",
            value: {
              $typeName: "map.v1.Seat",
              id: e.id,
              location: {
                $typeName: "map.v1.Location",
                x: e.position.x,
                y: e.position.y,
              },
              rotation: coreRotationToProto(e.rotation),
              ip: e.ip ?? undefined,
            },
          },
        });
    }
  });
}
