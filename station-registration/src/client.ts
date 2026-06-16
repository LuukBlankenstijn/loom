import { Code, ConnectError, createClient, type Interceptor } from "@connectrpc/connect";
import { createGrpcWebTransport } from "@connectrpc/connect-web";
import { create } from "@bufbuild/protobuf";
import { EmptySchema } from "@bufbuild/protobuf/wkt";
import {
  MapService,
  GetMapRequestSchema,
  AssignStationRequestSchema,
  type MapResponse,
} from "@client/v1/map/map_pb";
import {
  BroadcastService,
  BroadcastType,
  SubscribeBroadcastRequestSchema,
  type BroadcastEvent,
} from "@client/v1/broadcast/broadcast_pb";
import { ContestService, type Contest } from "@client/v1/admin/contest_pb";

export type ClientConfig = {
  server: string;
  auth: string | null;
};

export function createBackendClient(config: ClientConfig) {
  const authInterceptor: Interceptor = (next) => async (req) => {
    if (config.auth) {
      req.header.set("authorization", `Bearer ${config.auth}`);
    }
    return await next(req);
  };

  const transport = createGrpcWebTransport({
    baseUrl: config.server,
    interceptors: [authInterceptor],
  });

  const map = createClient(MapService, transport);
  const broadcast = createClient(BroadcastService, transport);
  const contest = createClient(ContestService, transport);

  return {
    // Returns the current/next contest, or null when none is scheduled.
    getNextContest: async (): Promise<Contest | null> => {
      try {
        return await contest.getNextContest(create(EmptySchema));
      } catch (err: unknown) {
        if (err instanceof ConnectError && err.code === Code.NotFound) {
          return null;
        }
        throw err;
      }
    },
    getMap: async (id: number): Promise<MapResponse> => {
      return await map.getMap(create(GetMapRequestSchema, { id }));
    },
    assignStationToSeat: async (
      seatId: string,
      stationIp: string,
    ): Promise<void> => {
      await map.assignStationToSeat(
        create(AssignStationRequestSchema, { seatId, stationIp }),
      );
    },
    subscribe: async function* (
      signal?: AbortSignal,
    ): AsyncIterable<BroadcastEvent> {
      const stream = broadcast.subscribe(
        create(SubscribeBroadcastRequestSchema, {
          types: [
            BroadcastType.CONNECTION_STATE,
            BroadcastType.STATION_ASSIGNMENTS,
          ],
        }),
        { signal },
      );

      try {
        for await (const ev of stream) {
          yield ev;
        }
      } catch (err: unknown) {
        if (
          (err instanceof ConnectError && err.code === Code.Canceled) ||
          (err instanceof Error && err.name === "AbortError") ||
          signal?.aborted
        ) {
          return;
        }
        throw err;
      }
    },
  };
}

export function emptyRequest() {
  return create(EmptySchema);
}
