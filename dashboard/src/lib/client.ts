import { createClient } from "@connectrpc/connect";
import { createConnectTransport } from "@connectrpc/connect-web";
import { AdminService } from "@client/v1/admin/admin_pb";
import type {
  Contest,
  StationsResponse,
  TeamsResponse,
  WallpaperResponse,
} from "@client/v1/admin/admin_pb";
import type {
  GetAllMapsResponse,
  MapResponse,
} from "@client/v1/admin/admin_pb";
import { SetMapRequestSchema } from "@client/v1/admin/admin_pb";
import { create } from "@bufbuild/protobuf";
import { EmptySchema } from "@bufbuild/protobuf/wkt";

const transport = createConnectTransport({
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
};
