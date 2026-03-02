import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { adminClient } from "../lib/client";
import { AssignModal } from "../components/AssignModal";
import type { Station } from "@client/v1/admin/admin_pb";
import { useStationState } from "../context/station";

export function StationsPage() {
  const queryClient = useQueryClient();
  const [modalOpen, setModalOpen] = useState(false);
  const [selectedStation, setSelectedStation] = useState<Station | null>(null);
  const { getState: getStationsState, connectedCount } = useStationState();

  const { data: stationsData, isLoading } = useQuery({
    queryKey: ["stations"],
    queryFn: () => adminClient.getStations(),
  });

  const { data: teamsData } = useQuery({
    queryKey: ["teams"],
    queryFn: () => adminClient.getActiveTeams(),
  });

  const stations = stationsData?.stations ?? [];
  const teams = teamsData?.teams ?? [];

  const ipToTeam = new Map(teams.filter((t) => t.ip).map((t) => [t.ip!, t]));

  const unassignMutation = useMutation({
    mutationFn: (teamId: string) => adminClient.setIp(teamId, undefined),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["teams"] });
      queryClient.invalidateQueries({ queryKey: ["stations"] });
    },
  });

  const openAssignModal = (station: Station) => {
    setSelectedStation(station);
    setModalOpen(true);
  };

  return (
    <div className="max-w-5xl mx-auto px-6 py-10">
      <div className="flex items-center gap-3 mb-8">
        <div className="w-2 h-8 bg-linear-to-b from-emerald-400 to-emerald-600 rounded-full" />
        <h1 className="text-3xl font-semibold text-white">Stations</h1>
        <div className="ml-auto flex gap-2">
          <span className="px-3 py-1 bg-success-500/20 text-success-500 rounded-full text-sm">
            {connectedCount} online
          </span>
          <span className="px-3 py-1 bg-surface-600 text-gray-400 rounded-full text-sm">
            {stations.length} total
          </span>
        </div>
      </div>
      {isLoading ? (
        <div className="text-gray-400">Loading...</div>
      ) : stations.length === 0 ? (
        <div className="text-center py-20 text-gray-400">No stations found</div>
      ) : (
        <div className="bg-surface-800 rounded-xl border border-surface-600 overflow-hidden shadow-xl">
          <table className="w-full">
            <thead>
              <tr className="border-b border-surface-600 bg-surface-800/50">
                <th className="text-left px-6 py-4 text-sm font-semibold text-gray-300 w-24">
                  ID
                </th>
                <th className="text-left px-6 py-4 text-sm font-semibold text-gray-300 w-44">
                  IP Address
                </th>
                <th className="text-left px-6 py-4 text-sm font-semibold text-gray-300">
                  Team
                </th>
                <th className="text-left px-6 py-4 text-sm font-semibold text-gray-300 w-36">
                  Logged In
                </th>
                <th className="text-left px-6 py-4 text-sm font-semibold text-gray-300 w-36">
                  Status
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-surface-700">
              {stations.map((station) => {
                const team = ipToTeam.get(station.ip);
                const connectionState = getStationsState(station.ip);

                return (
                  <tr
                    key={station.id}
                    className="hover:bg-surface-700/50 transition-colors"
                  >
                    <td className="px-6 py-4 text-gray-400 font-mono text-sm">
                      {station.id}
                    </td>
                    <td className="px-6 py-4 font-mono text-gray-200">
                      {station.ip}
                    </td>
                    <td className="px-6 py-4">
                      {team ? (
                        <div className="flex items-center gap-3">
                          <span className="text-gray-200 bg-purple-500/10 px-2 py-1 rounded">
                            {team.name}
                          </span>
                          <button
                            onClick={() => unassignMutation.mutate(team.id)}
                            disabled={unassignMutation.isPending}
                            className="p-1.5 rounded-md bg-danger-500/10 text-danger-500 hover:bg-danger-500/20 hover:text-danger-400 transition-colors border border-danger-500/20"
                            title="Unassign team"
                          >
                            <svg
                              className="w-3.5 h-3.5"
                              fill="none"
                              stroke="currentColor"
                              viewBox="0 0 24 24"
                            >
                              <path
                                strokeLinecap="round"
                                strokeLinejoin="round"
                                strokeWidth={2}
                                d="M6 18L18 6M6 6l12 12"
                              />
                            </svg>
                          </button>
                        </div>
                      ) : (
                        <button
                          onClick={() => openAssignModal(station)}
                          className="px-3 py-1.5 bg-primary-500 hover:bg-primary-600 text-white text-sm rounded-lg transition-colors"
                        >
                          Assign Team
                        </button>
                      )}
                    </td>
                    <td className="px-6 py-4">
                      <div className="flex items-center gap-2">
                        <span
                          className={`w-2.5 h-2.5 rounded-full ${
                            connectionState.loggedIn
                              ? "bg-success-500 shadow-[0_0_8px_rgba(34,197,94,0.5)]"
                              : "bg-gray-500"
                          }`}
                        />
                        <span
                          className={`text-sm ${connectionState.loggedIn ? "text-success-500" : "text-gray-500"}`}
                        >
                          {connectionState.loggedIn ? "Yes" : "No"}
                        </span>
                      </div>
                    </td>
                    <td className="px-6 py-4">
                      <div className="flex items-center gap-2">
                        <span
                          className={`w-2.5 h-2.5 rounded-full ${
                            connectionState.connected
                              ? "bg-success-500 shadow-[0_0_8px_rgba(34,197,94,0.5)]"
                              : "bg-danger-500"
                          }`}
                        />
                        <span
                          className={`text-sm ${connectionState.connected ? "text-success-500" : "text-gray-500"}`}
                        >
                          {connectionState.connected ? "Online" : "Offline"}
                        </span>
                      </div>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}

      {modalOpen && selectedStation && (
        <AssignModal
          mode="station"
          stationIp={selectedStation.ip}
          stations={stations}
          teams={teams}
          onClose={() => {
            setModalOpen(false);
            setSelectedStation(null);
          }}
        />
      )}
    </div>
  );
}
