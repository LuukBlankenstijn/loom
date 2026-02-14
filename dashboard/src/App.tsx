import { BrowserRouter, Routes, Route } from "react-router-dom";
import { Layout } from "./components/Layout";
import { ContestPage } from "./pages/ContestPage";
import { TeamsPage } from "./pages/TeamsPage";
import { StationsPage } from "./pages/StationsPage";
import { MapsPage } from "./pages/MapsPage";

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<Layout />}>
          <Route index element={<ContestPage />} />
          <Route path="teams" element={<TeamsPage />} />
          <Route path="stations" element={<StationsPage />} />
          <Route path="maps" element={<MapsPage />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}

export default App;
