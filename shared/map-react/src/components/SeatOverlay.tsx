import { Circle, Group, Rect, Text } from "react-konva";
import {
  SEAT_TABLE_W,
  SEAT_TABLE_H,
  SEAT_CHAIR_PROTRUSION,
} from "../types";
import type { Seat } from "../types";

const DOT_RADIUS = 4;
const DOT_PADDING = 8;

const IP_FONT_SIZE = 14;
const TEAM_FONT_SIZE = 11;
const LABEL_GAP = 2;

const COLOR_CONNECTED = "#22c55e";
const COLOR_DISCONNECTED = "#ef4444";
const COLOR_IP = "#f3f4f6";
const COLOR_TEAM = "#9ca3af";
const COLOR_ACTIVE = "#f59e0b";
const COLOR_ACTIVE_FILL = "rgba(245,158,11,0.18)";

export type SeatOverlayProps = {
  seat: Seat;
  connected: boolean;
  teamName: string | null;
  /** Highlights this seat as the one currently claimed by this machine. */
  active?: boolean;
};

export function SeatOverlay({
  seat,
  connected,
  teamName,
  active = false,
}: SeatOverlayProps) {
  const tableHalfW = SEAT_TABLE_W / 2;
  const tableHalfH = SEAT_TABLE_H / 2;
  const verticalShift = SEAT_CHAIR_PROTRUSION / 2;
  const dotX = tableHalfW - DOT_PADDING;
  const dotY = -tableHalfH + verticalShift + DOT_PADDING;
  const dotColor = connected ? COLOR_CONNECTED : COLOR_DISCONNECTED;

  const boxW = SEAT_TABLE_W - 16;
  const ipBoxH = IP_FONT_SIZE + 4;
  const teamBoxH = TEAM_FONT_SIZE + 4;
  const showTeam = !!teamName;
  const totalH = showTeam ? ipBoxH + LABEL_GAP + teamBoxH : ipBoxH;

  return (
    <Group>
      {active && (
        <Rect
          x={-tableHalfW}
          y={-tableHalfH - SEAT_CHAIR_PROTRUSION + verticalShift}
          width={SEAT_TABLE_W}
          height={SEAT_TABLE_H + SEAT_CHAIR_PROTRUSION}
          stroke={COLOR_ACTIVE}
          strokeWidth={3}
          cornerRadius={6}
          fill={COLOR_ACTIVE_FILL}
          listening={false}
        />
      )}
      <Circle x={dotX} y={dotY} radius={DOT_RADIUS} fill={dotColor} />
      {seat.ip && (
        <Group x={0} y={verticalShift} rotation={-seat.rotation}>
          <Text
            x={-boxW / 2}
            y={-totalH / 2}
            width={boxW}
            height={ipBoxH}
            text={seat.ip}
            fontSize={IP_FONT_SIZE}
            fontFamily="ui-monospace, SFMono-Regular, Menlo, monospace"
            fontStyle="600"
            fill={COLOR_IP}
            align="center"
            verticalAlign="middle"
            listening={false}
          />
          {showTeam && (
            <Text
              x={-boxW / 2}
              y={-totalH / 2 + ipBoxH + LABEL_GAP}
              width={boxW}
              height={teamBoxH}
              text={teamName}
              fontSize={TEAM_FONT_SIZE}
              fontFamily="system-ui, sans-serif"
              fill={COLOR_TEAM}
              align="center"
              verticalAlign="middle"
              listening={false}
            />
          )}
        </Group>
      )}
    </Group>
  );
}
