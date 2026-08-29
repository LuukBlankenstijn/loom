import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { adminClient } from "../lib/client";
import { getErrorMessage } from "../lib/errors";
import { AssignModal } from "../components/AssignModal";
import type { Team } from "@client/v1/admin/team_pb";

export function TeamsPage() {
  const queryClient = useQueryClient();
  const [modalOpen, setModalOpen] = useState(false);
  const [selectedTeam, setSelectedTeam] = useState<Team | null>(null);

  const { data, isLoading } = useQuery({
    queryKey: ["teams"],
    queryFn: () => adminClient.getActiveTeams(),
  });

  const { data: stationsData } = useQuery({
    queryKey: ["stations"],
    queryFn: () => adminClient.getStations(),
  });

  const unassignMutation = useMutation({
    mutationFn: (teamId: string) => adminClient.setIp(teamId, undefined),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["teams"] });
      queryClient.invalidateQueries({ queryKey: ["stations"] });
    },
  });

  const unassignError = unassignMutation.isError
    ? getErrorMessage(unassignMutation.error)
    : null;

  const teams = data?.teams ?? [];

  const openAssignModal = (team: Team) => {
    setSelectedTeam(team);
    setModalOpen(true);
  };

  return (
    <div className="max-w-5xl mx-auto px-6 py-10">
      <div className="flex items-center gap-3 mb-8">
        <div className="w-2 h-8 bg-linear-to-b from-primary-400 to-primary-600 rounded-full" />
        <h1 className="text-3xl font-semibold text-white">Teams</h1>
        <span className="ml-auto px-3 py-1 bg-primary-500/20 text-primary-400 rounded-full text-sm">
          {teams.length} total
        </span>
      </div>
      {unassignError && (
        <div className="mb-4 px-4 py-3 rounded-lg bg-danger-500/10 border border-danger-500/30 text-danger-500 text-sm">
          {unassignError}
        </div>
      )}
      {isLoading ? (
        <div className="text-gray-400">Loading...</div>
      ) : teams.length === 0 ? (
        <div className="text-center py-20 text-gray-400">No teams found</div>
      ) : (
        <div className="bg-surface-800 rounded-xl border border-surface-600 overflow-hidden shadow-xl">
          <table className="w-full">
            <thead>
              <tr className="border-b border-surface-600 bg-surface-800/50">
                <th className="text-left px-6 py-4 text-sm font-semibold text-gray-300 w-24">
                  ID
                </th>
                <th className="text-left px-6 py-4 text-sm font-semibold text-gray-300 w-64">
                  IP Address
                </th>
                <th className="text-left px-6 py-4 text-sm font-semibold text-gray-300">
                  Name
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-surface-700">
              {teams.map((team) => (
                <tr
                  key={team.id}
                  className="hover:bg-surface-700/50 transition-colors"
                >
                  <td className="px-6 py-4 text-gray-400 font-mono text-sm">
                    {team.id}
                  </td>
                  <td className="px-6 py-4">
                    {team.ip ? (
                      <div className="flex items-center gap-3">
                        <span className="font-mono text-emerald-400 bg-emerald-500/10 px-2 py-1 rounded">
                          {team.ip}
                        </span>
                        <button
                          onClick={() => unassignMutation.mutate(team.id)}
                          disabled={unassignMutation.isPending}
                          className="p-1.5 rounded-md bg-danger-500/10 text-danger-500 hover:bg-danger-500/20 hover:text-danger-400 transition-colors border border-danger-500/20"
                          title="Unassign IP"
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
                        onClick={() => openAssignModal(team)}
                        className="px-3 py-1.5 bg-primary-500 hover:bg-primary-600 text-white text-sm rounded-lg transition-colors"
                      >
                        Assign IP
                      </button>
                    )}
                  </td>
                  <td className="px-6 py-4 text-gray-200">{team.name}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {modalOpen && selectedTeam && (
        <AssignModal
          mode="team"
          teamId={selectedTeam.id}
          stations={stationsData?.stations ?? []}
          teams={teams}
          onClose={() => {
            setModalOpen(false);
            setSelectedTeam(null);
          }}
        />
      )}
    </div>
  );
}
