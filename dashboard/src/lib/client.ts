import { Code, ConnectError, createClient } from "@connectrpc/connect";
import { createGrpcWebTransport } from "@connectrpc/connect-web";
import { create } from "@bufbuild/protobuf";
import { EmptySchema } from "@bufbuild/protobuf/wkt";
import {
  ContestService,
  SetMapRequestSchema,
  type Contest,
} from "@client/v1/admin/contest_pb";
import {
  WallpaperKind,
  WallpaperService,
  type ListWallpapersResponse,
  type Wallpaper,
} from "@client/v1/admin/wallpaper_pb";
import {
  AdminEventSchema,
  CommandOutputRequestSchema,
  StationService,
  type AdminEvent,
  type CustomCommandOutput,
  type StationsResponse,
} from "@client/v1/admin/station_pb";
import { TeamService, type TeamsResponse } from "@client/v1/admin/team_pb";
import {
  MapService,
  GetMapRequestSchema,
  UpdateMapRequestSchema,
  AssignStationRequestSchema,
  type GetAllMapMetadataResponse,
  type MapResponse,
} from "@client/v1/map/map_pb";
import type { Element as ProtoElement } from "@client/v1/map/types_pb";
import {
  BroadcastService,
  BroadcastType,
  SubscribeBroadcastRequestSchema,
  type BroadcastEvent,
} from "@client/v1/broadcast/broadcast_pb";
import { SESSION_ID } from "../session";

const transport = createGrpcWebTransport({
  baseUrl: "/rpc",
});

const contest_client = createClient(ContestService, transport);
const team_client = createClient(TeamService, transport);
const station_client = createClient(StationService, transport);
const map_client = createClient(MapService, transport);
const broadcast_client = createClient(BroadcastService, transport);
const wallpaper_client = createClient(WallpaperService, transport);

export const wallpaperPreviewUrl = (wallpaper: Wallpaper): string => {
  if (wallpaper.source.case !== "url") {
    return `/api/wallpapers/${wallpaper.id}/content`;
  }
  return wallpaper.kind === WallpaperKind.YOUTUBE
    ? `https://i.ytimg.com/vi/${new URL(wallpaper.source.value).searchParams.get("v")}/hqdefault.jpg`
    : wallpaper.source.value;
};

export const adminClient = {
  // contest
  getNextContest: async (): Promise<Contest> => {
    return await contest_client.getNextContest(create(EmptySchema));
  },
  setWallpaper: async (
    contestId: string,
    wallpaperId?: number,
  ): Promise<void> => {
    await contest_client.setWallpaper({ contestId, wallpaperId });
  },
  setMap: async (contestId: string, mapId: number): Promise<void> => {
    await contest_client.setMap(
      create(SetMapRequestSchema, { contestId, mapId }),
    );
  },

  listWallpapers: async (): Promise<ListWallpapersResponse> => {
    return await wallpaper_client.listWallpapers(create(EmptySchema));
  },
  createWallpaperFromFile: async (
    name: string,
    imageData: Uint8Array,
  ): Promise<Wallpaper> => {
    return await wallpaper_client.createWallpaper({
      name,
      source: { case: "imageData", value: imageData },
    });
  },
  createWallpaperFromUrl: async (
    name: string,
    url: string,
    kind: WallpaperKind,
  ): Promise<Wallpaper> => {
    return await wallpaper_client.createWallpaper({
      name,
      source: { case: "url", value: { url, kind } },
    });
  },
  updateWallpaper: async (
    id: number,
    changes: { name?: string; textColor?: string },
  ): Promise<void> => {
    await wallpaper_client.updateWallpaper({ id, ...changes });
  },
  deleteWallpaper: async (id: number): Promise<void> => {
    await wallpaper_client.deleteWallpaper({ id });
  },
  setDefaultWallpaper: async (wallpaperId?: number): Promise<void> => {
    await wallpaper_client.setDefaultWallpaper({ wallpaperId });
  },

  // teams
  getActiveTeams: async (): Promise<TeamsResponse> => {
    return await team_client.getActiveTeams(create(EmptySchema));
  },
  setIp: async (teamId: string, ip?: string): Promise<void> => {
    await team_client.setIp({ teamId, ip });
  },

  // station
  getStations: async (): Promise<StationsResponse> => {
    return station_client.getStations(create(EmptySchema));
  },
  deleteStation: async (ip: string): Promise<void> => {
    await station_client.deleteStation({ ip });
  },
  setStationLabel: async (ip: string, label?: string): Promise<void> => {
    await station_client.setStationLabel({ ip, label });
  },
  assignTeam: async (ips: string[]): Promise<void> => {
    await station_client.assignTeam({ ips });
  },
  sendEvent: async (
    ips: string[],
    command: AdminEvent["command"],
  ): Promise<void> => {
    // inject the session id for the custom command
    if (command.case === "custom") {
      command.value.adminId = SESSION_ID;
    }
    await station_client.sendCommand(
      create(AdminEventSchema, { ips, command }),
    );
  },

  // map
  getAllMaps: async (): Promise<GetAllMapMetadataResponse> => {
    return await map_client.getAllMapMetadata(create(EmptySchema));
  },
  createMap: async (name: string): Promise<MapResponse> => {
    return await map_client.createMap({ name });
  },
  getMap: async (id: number): Promise<MapResponse> => {
    return await map_client.getMap(create(GetMapRequestSchema, { id }));
  },
  updateMap: async (
    id: number,
    deleted: string[],
    updated: ProtoElement[],
  ): Promise<void> => {
    await map_client.updateMap(
      create(UpdateMapRequestSchema, { id, deleted, updated }),
    );
  },
  assignStationToSeat: async (
    seatId: string,
    stationIp: string | undefined,
  ): Promise<void> => {
    await map_client.assignStationToSeat(
      create(AssignStationRequestSchema, { seatId, stationIp }),
    );
  },

  subscribe: async function* (
    signal?: AbortSignal,
    types: BroadcastType[] = [BroadcastType.CONNECTION_STATE],
  ): AsyncIterable<BroadcastEvent> {
    const stream = broadcast_client.subscribe(
      create(SubscribeBroadcastRequestSchema, { types }),
      { signal },
    );

    try {
      for await (const response of stream) {
        yield response;
      }
    } catch (err: unknown) {
      const isGrpcCanceled =
        err instanceof ConnectError && err.code === Code.Canceled;
      const isBrowserAbort = err instanceof Error && err.name === "AbortError";
      const isStreamAbort =
        err instanceof Error && err.message.includes("input stream");

      if (
        isGrpcCanceled ||
        isBrowserAbort ||
        isStreamAbort ||
        signal?.aborted
      ) {
        return;
      }

      throw err;
    }
  },

  commandOutput: async function* (
    signal?: AbortSignal,
  ): AsyncIterable<CustomCommandOutput> {
    const stream = station_client.commandOutput(
      create(CommandOutputRequestSchema, { adminId: SESSION_ID }),
      { signal },
    );

    try {
      for await (const response of stream) {
        yield response;
      }
    } catch (err: unknown) {
      const isGrpcCanceled =
        err instanceof ConnectError && err.code === Code.Canceled;
      const isBrowserAbort = err instanceof Error && err.name === "AbortError";
      const isStreamAbort =
        err instanceof Error && err.message.includes("input stream");

      if (
        isGrpcCanceled ||
        isBrowserAbort ||
        isStreamAbort ||
        signal?.aborted
      ) {
        return;
      }

      throw err;
    }
  },
};
