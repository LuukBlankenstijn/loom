import { invoke } from "@tauri-apps/api/core";

export type StationConfig = {
  ip: string;
};

export async function getStationConfig(): Promise<StationConfig> {
  return await invoke<StationConfig>("get_station_config");
}
