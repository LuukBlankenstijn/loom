import { Code, ConnectError, createClient } from "@connectrpc/connect";
import { createGrpcWebTransport } from "@connectrpc/connect-web";
import { AdminService } from "@client/v1/admin/admin_pb";
import type {
  ClientCommand,
  Contest,
  StationsResponse,
  SubscribtionMessage,
  TeamsResponse,
  WallpaperResponse,
} from "@client/v1/admin/admin_pb";
import type {
  GetAllMapsResponse,
  MapResponse,
} from "@client/v1/admin/admin_pb";
import {
  ClientCommandSchema,
  SetMapRequestSchema,
} from "@client/v1/admin/admin_pb";
import { create } from "@bufbuild/protobuf";
import { EmptySchema } from "@bufbuild/protobuf/wkt";

const transport = createGrpcWebTransport({
  baseUrl: "/api",
});

const client = createClient(AdminService, transport);

export const adminClient = {
  getNextContest: async (): Promise<Contest> => {
    return (await client.getNextContest(create(EmptySchema))) as Contest;
  },
  getActiveTeams: async (): Promise<TeamsResponse> => {
    return (await client.getActiveTeams(create(EmptySchema))) as TeamsResponse;
  },
  getStations: async (): Promise<StationsResponse> => {
    return (await client.getStations(create(EmptySchema))) as StationsResponse;
  },
  deleteStation: async (id: number): Promise<void> => {
    await client.deleteStation({ id });
  },
  assignTeam: async (ids: number[]): Promise<void> => {
    await client.assignTeam({ ids });
  },
  setIp: async (teamId: string, ip?: string): Promise<void> => {
    await client.setIp({ teamId, ip });
  },
  getWallpaper: async (contestId?: string): Promise<WallpaperResponse> => {
    return (await client.getWallpaper({ contestId })) as WallpaperResponse;
  },
  setWallpaper: async (
    contestId: string,
    imageData: Uint8Array,
  ): Promise<void> => {
    await client.setWallpaper({ contestId, imageData });
  },
  setWallpaperTextColor: async (
    contestId: string,
    color: string,
  ): Promise<void> => {
    await client.setWallpaperTextColor({ contestId, color });
  },
  getAllMaps: async (): Promise<GetAllMapsResponse> => {
    return (await client.getAllMaps(create(EmptySchema))) as GetAllMapsResponse;
  },
  createMap: async (name: string): Promise<MapResponse> => {
    return (await client.createMap({ name })) as MapResponse;
  },
  setMap: async (contestId: string, mapId: number): Promise<void> => {
    await client.setMap(create(SetMapRequestSchema, { contestId, mapId }));
  },
  sendCommand: async (
    ips: string[],
    command: ClientCommand["command"],
  ): Promise<void> => {
    await client.sendCommand(create(ClientCommandSchema, { ips, command }));
  },
  subscribe: async function* (
    signal?: AbortSignal,
  ): AsyncIterable<SubscribtionMessage> {
    const stream = client.subscribe(create(EmptySchema), { signal });

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
