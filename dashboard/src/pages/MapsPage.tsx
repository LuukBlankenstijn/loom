import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { Link } from "react-router-dom";
import { adminClient } from "../lib/client";

export function MapsPage() {
  const queryClient = useQueryClient();
  const [isCreating, setIsCreating] = useState(false);
  const [newMapName, setNewMapName] = useState("");

  const { data, isLoading } = useQuery({
    queryKey: ["maps"],
    queryFn: () => adminClient.getAllMaps(),
  });

  const createMutation = useMutation({
    mutationFn: (name: string) => adminClient.createMap(name),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["maps"] });
      setIsCreating(false);
      setNewMapName("");
    },
  });

  const maps = data?.maps ?? [];

  const handleCreate = () => {
    if (!newMapName.trim()) return;
    createMutation.mutate(newMapName.trim());
  };

  return (
    <div className="max-w-5xl mx-auto px-6 py-10">
      <div className="flex items-center gap-3 mb-8">
        <div className="w-2 h-8 bg-linear-to-b from-violet-400 to-violet-600 rounded-full" />
        <h1 className="text-3xl font-semibold text-white">Maps</h1>
        <span className="ml-auto px-3 py-1 bg-violet-500/20 text-violet-400 rounded-full text-sm">
          {maps.length} total
        </span>
        <button
          onClick={() => setIsCreating(true)}
          className="px-4 py-2 bg-linear-to-r from-violet-500 to-violet-600 hover:from-violet-600 hover:to-violet-700 text-white rounded-lg transition-all font-medium flex items-center gap-2"
        >
          <svg
            className="w-5 h-5"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M12 4v16m8-8H4"
            />
          </svg>
          New Map
        </button>
      </div>

      {isCreating && (
        <div className="mb-6 p-4 bg-surface-800 rounded-xl border border-surface-600">
          <h3 className="text-lg font-medium text-white mb-4">
            Create New Map
          </h3>
          <div className="flex gap-3">
            <input
              type="text"
              value={newMapName}
              onChange={(e) => setNewMapName(e.target.value)}
              placeholder="Map name"
              className="flex-1 bg-surface-700 border border-surface-500 rounded-lg px-4 py-2.5 text-gray-200 placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-violet-500"
              onKeyDown={(e) => e.key === "Enter" && handleCreate()}
              autoFocus
            />
            <button
              onClick={() => {
                setIsCreating(false);
                setNewMapName("");
              }}
              className="px-4 py-2 text-gray-400 hover:text-white transition-colors"
            >
              Cancel
            </button>
            <button
              onClick={handleCreate}
              disabled={!newMapName.trim() || createMutation.isPending}
              className="px-4 py-2 bg-violet-500 hover:bg-violet-600 disabled:bg-surface-600 disabled:text-gray-500 text-white rounded-lg transition-colors"
            >
              {createMutation.isPending ? "Creating..." : "Create"}
            </button>
          </div>
        </div>
      )}

      {isLoading ? (
        <div className="text-gray-400">Loading...</div>
      ) : maps.length === 0 ? (
        <div className="text-center py-20">
          <div className="w-20 h-20 mx-auto mb-4 rounded-full bg-surface-700 flex items-center justify-center">
            <svg
              className="w-10 h-10 text-gray-500"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={1.5}
                d="M9 20l-5.447-2.724A1 1 0 013 16.382V5.618a1 1 0 011.447-.894L9 7m0 13l6-3m-6 3V7m6 10l4.553 2.276A1 1 0 0021 18.382V7.618a1 1 0 00-.553-.894L15 4m0 13V4m0 0L9 7"
              />
            </svg>
          </div>
          <p className="text-gray-400 text-lg">No maps yet</p>
          <p className="text-gray-500 text-sm mt-1">
            Create your first map to get started
          </p>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {maps.map((map) => (
            <div
              key={map.id}
              className="bg-surface-800 rounded-xl border border-surface-600 p-5 hover:border-violet-500/50 transition-colors group"
            >
              <div className="flex items-start justify-between mb-3">
                <div className="w-10 h-10 rounded-lg bg-linear-to-br from-violet-500/20 to-purple-500/20 border border-violet-500/30 flex items-center justify-center">
                  <svg
                    className="w-5 h-5 text-violet-400"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={1.5}
                      d="M9 20l-5.447-2.724A1 1 0 013 16.382V5.618a1 1 0 011.447-.894L9 7m0 13l6-3m-6 3V7m6 10l4.553 2.276A1 1 0 0021 18.382V7.618a1 1 0 00-.553-.894L15 4m0 13V4m0 0L9 7"
                    />
                  </svg>
                </div>
                <span className="text-xs text-gray-500 font-mono">
                  #{map.id}
                </span>
              </div>
              <h3 className="text-lg font-medium text-white mb-4">
                {map.name}
              </h3>
              <div className="flex gap-2">
                <Link
                  to={`/maps/${map.id}/view`}
                  className="flex-1 px-3 py-2 bg-violet-500/15 hover:bg-violet-500/25 border border-violet-500/40 text-violet-300 hover:text-violet-200 text-sm rounded-lg transition-colors flex items-center justify-center gap-2"
                >
                  <svg
                    className="w-4 h-4"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                    />
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"
                    />
                  </svg>
                  View
                </Link>
                <Link
                  to={`/maps/${map.id}/edit`}
                  className="flex-1 px-3 py-2 bg-surface-700 hover:bg-surface-600 text-gray-300 hover:text-white text-sm rounded-lg transition-colors flex items-center justify-center gap-2"
                >
                  <svg
                    className="w-4 h-4"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"
                    />
                  </svg>
                  Edit
                </Link>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
