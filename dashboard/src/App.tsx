import { BrowserRouter, Routes, Route } from "react-router-dom";
import { Layout } from "./components/Layout";
import { ContestPage } from "./pages/ContestPage";
import { TeamsPage } from "./pages/TeamsPage";
import { StationsPage } from "./pages/StationsPage";
import { MapsPage } from "./pages/MapsPage";
import { MapEditorPage } from "./pages/MapEditorPage";
import { StationsProvider } from "./context/station";

function App() {
  return (
    <StationsProvider>
      <BrowserRouter>
        <Routes>
          <Route path="/" element={<Layout />}>
            <Route index element={<ContestPage />} />
            <Route path="teams" element={<TeamsPage />} />
            <Route path="stations" element={<StationsPage />} />
            <Route path="maps" element={<MapsPage />} />
          </Route>
          {/* Editor is outside layout - needs full screen for WASM canvas */}
          <Route path="/maps/:mapId/edit" element={<MapEditorPage />} />
        </Routes>
      </BrowserRouter>
    </StationsProvider>
  );
}

export default App;
