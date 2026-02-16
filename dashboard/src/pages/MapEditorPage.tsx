import { useParams, Link } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { adminClient } from "../lib/client";

export function MapEditorPage() {
  const { mapId } = useParams<{ mapId: string }>();

  const { data: mapsData } = useQuery({
    queryKey: ["maps"],
    queryFn: () => adminClient.getAllMaps(),
  });

  const map = mapsData?.maps.find((m) => m.id === Number(mapId));

  // Build the editor URL with the map ID as a query parameter
  const editorUrl = `/editor/?mapId=${mapId}`;

  return (
    <div className="h-screen flex flex-col bg-surface-900">
      <div className="bg-surface-800 border-b border-surface-600 px-6 py-3 flex items-center gap-4 shrink-0">
        <Link
          to="/maps"
          className="p-2 rounded-lg text-gray-400 hover:text-white hover:bg-surface-700 transition-colors"
        >
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
          </svg>
        </Link>
        <div className="w-px h-6 bg-surface-600" />
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-violet-500/20 to-purple-500/20 border border-violet-500/30 flex items-center justify-center">
            <svg className="w-4 h-4 text-violet-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9 20l-5.447-2.724A1 1 0 013 16.382V5.618a1 1 0 011.447-.894L9 7m0 13l6-3m-6 3V7m6 10l4.553 2.276A1 1 0 0021 18.382V7.618a1 1 0 00-.553-.894L15 4m0 13V4m0 0L9 7" />
            </svg>
          </div>
          <div>
            <h1 className="text-white font-medium">{map?.name ?? `Map #${mapId}`}</h1>
            <p className="text-xs text-gray-500">Editing map</p>
          </div>
        </div>
      </div>
      <div className="flex-1 relative">
        <iframe
          src={editorUrl}
          className="absolute inset-0 w-full h-full border-0"
          title="Map Editor"
        />
      </div>
    </div>
  );
}
