import { useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { adminClient } from "../lib/client";
import type { Station, Team } from "@client/admin/v1/admin_pb";

type AssignModalProps = {
  mode: "team" | "station";
  teamId?: string;
  stationIp?: string;
  stations: Station[];
  teams: Team[];
  onClose: () => void;
};

export function AssignModal({
  mode,
  teamId,
  stationIp,
  stations,
  teams,
  onClose,
}: AssignModalProps) {
  const queryClient = useQueryClient();
  const [selectedValue, setSelectedValue] = useState("");

  const usedIps = new Set(teams.filter((t) => t.ip).map((t) => t.ip!));
  const availableStations = stations.filter((s) => !usedIps.has(s.ip));
  const unassignedTeams = teams.filter((t) => !t.ip);

  const assignMutation = useMutation({
    mutationFn: ({ teamId, ip }: { teamId: string; ip: string }) =>
      adminClient.setIp(teamId, ip),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["teams"] });
      queryClient.invalidateQueries({ queryKey: ["stations"] });
      onClose();
    },
  });

  const handleAssign = () => {
    if (!selectedValue) return;

    if (mode === "team" && teamId) {
      assignMutation.mutate({ teamId, ip: selectedValue });
    } else if (mode === "station" && stationIp) {
      assignMutation.mutate({ teamId: selectedValue, ip: stationIp });
    }
  };

  return (
    <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50">
      <div className="bg-surface-800 rounded-xl border border-surface-600 p-6 w-full max-w-md shadow-2xl">
        <h2 className="text-xl font-semibold text-white mb-6">
          {mode === "team" ? "Assign Station" : "Assign Team"}
        </h2>

        <div className="mb-6">
          <label className="block text-sm text-gray-400 mb-2">
            {mode === "team" ? "Select Station" : "Select Team"}
          </label>
          <select
            value={selectedValue}
            onChange={(e) => setSelectedValue(e.target.value)}
            className="w-full bg-surface-700 border border-surface-500 rounded-lg px-4 py-2.5 text-gray-200 focus:outline-none focus:ring-2 focus:ring-primary-500"
          >
            <option value="">Choose...</option>
            {mode === "team"
              ? availableStations.map((station) => (
                  <option key={station.id} value={station.ip}>
                    Station {station.id} - {station.ip}
                  </option>
                ))
              : unassignedTeams.map((team) => (
                  <option key={team.id} value={team.id}>
                    {team.name}
                  </option>
                ))}
          </select>
          {mode === "team" && availableStations.length === 0 && (
            <p className="text-sm text-gray-500 mt-2">
              No available stations
            </p>
          )}
          {mode === "station" && unassignedTeams.length === 0 && (
            <p className="text-sm text-gray-500 mt-2">
              All teams are already assigned
            </p>
          )}
        </div>

        <div className="flex justify-end gap-3">
          <button
            onClick={onClose}
            className="px-4 py-2 text-gray-400 hover:text-white transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={handleAssign}
            disabled={!selectedValue || assignMutation.isPending}
            className="px-4 py-2 bg-primary-500 hover:bg-primary-600 disabled:bg-surface-600 disabled:text-gray-500 text-white rounded-lg transition-colors"
          >
            {assignMutation.isPending ? "Assigning..." : "Assign"}
          </button>
        </div>
      </div>
    </div>
  );
}
