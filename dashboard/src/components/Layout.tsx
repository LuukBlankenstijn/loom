import { NavLink, Outlet } from "react-router-dom";

const navItems = [
  { to: "/", label: "Contest" },
  { to: "/teams", label: "Teams" },
  { to: "/stations", label: "Stations" },
  { to: "/maps", label: "Maps" },
];

export function Layout() {
  return (
    <div className="h-screen flex flex-col">
      <header className="bg-surface-800 border-b border-surface-600 shrink-0">
        <nav className="max-w-7xl mx-auto px-6">
          <div className="flex items-center h-16 gap-8">
            <span className="text-xl font-semibold bg-linear-to-r from-primary-400 to-emerald-400 bg-clip-text text-transparent">
              Loom
            </span>
            <div className="flex gap-1">
              {navItems.map((item) => (
                <NavLink
                  key={item.to}
                  to={item.to}
                  className={({ isActive }) =>
                    `px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
                      isActive
                        ? "bg-primary-500/20 text-primary-400 border border-primary-500/30"
                        : "text-gray-400 hover:text-white hover:bg-surface-700"
                    }`
                  }
                >
                  {item.label}
                </NavLink>
              ))}
            </div>
          </div>
        </nav>
      </header>
      <main className="flex-1 overflow-auto">
        <Outlet />
      </main>
    </div>
  );
}
