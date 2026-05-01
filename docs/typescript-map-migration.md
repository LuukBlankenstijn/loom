# TypeScript Map Migration

Migrate all map rendering and editing from Rust+iced to TypeScript+React. Replace the iframe-embedded WASM editor with a React component the dashboard imports directly. Replace the iced station-registration app with a Tauri 2 app that reuses that same component.

## Decisions (locked in)

- **Canvas tech:** Konva (`react-konva`) — closest 1:1 to the iced Canvas mental model, handles pan/zoom/hit-testing, scales fine for typical contest map sizes.
- **State management:** zustand — one store per `<MapEditor>`/`<MapViewer>` instance (via `createStore`/context), so the Tauri app's overlays can subscribe to the same state without prop drilling.
- **Tauri Rust shell:** minimal. Window, CLI arg parsing, two Tauri commands (`get_local_ip`, `get_auth_token`). Everything else — gRPC, rendering, broadcast subscription, claim flow — lives in TypeScript using the existing `@connectrpc/connect-web` setup.
- **Coordinate model:** keep f32 world-space, `grid_size = 100`, `snap = 10`. Port the math from `shared/map/src/{grid,types/{wall,door,seat}}.rs` literally so saved maps don't need migration.

## Code that goes away

- `shared/map/` — the `Map<T>`, `Grid`, `Drawable` trait, per-element draw/hit-test for wall/door/seat, `create_map_element!` macro.
- `map-editor/` — iced WASM editor served at `/editor/` and embedded via iframe in `MapEditorPage.tsx`.
- `station-registration/` iced UI — `ContestMap`, `SeatDecorator`, `subscription.rs`. The crate itself becomes the Tauri app (Rust shell only).
- `shared/client/` — Rust gRPC client used only by the two iced apps. Backend doesn't need it.
- `shared/proto-bridge/` — only if backend doesn't still rely on it for Rust↔core conversions; check before deleting.

Toolchains to drop from `flake.nix` / `shell.nix` once removed: wasm target, iced/wgpu deps, `console_error_panic_hook`, `tonic-web-wasm-client`, `wasm-pack` if used.

## Code that stays

- All proto schemas in `proto/v1/map/` and `proto/v1/broadcast/` — no changes needed for this migration.
- `gen/ts/v1/map/{types,map}_pb.ts` and `gen/ts/v1/broadcast/broadcast_pb.ts` — already consumed by dashboard.
- Backend (separate rewrite, owned by user).

## End-state architecture

```
shared/map-react/             ← new pnpm workspace package, @loom/map-react
  src/
    coords.ts                 Point, Vector, Rotation, world↔screen
    geometry.ts               snap-to-grid, distance helpers
    hit.ts                    is_hit per element type
    store.ts                  zustand store: elements, selection, grid, mode
    proto.ts                  MapElement union ↔ generated pb.ts conversions
    components/
      MapViewer.tsx           read-only: grid + elements + pan/zoom
      MapEditor.tsx           wraps MapViewer + adds edit interactions + HUD
      Grid.tsx
      elements/Wall.tsx
      elements/Door.tsx
      elements/Seat.tsx
      Hud.tsx
    decorators/
      types.ts                SeatOverlayProps interface
  index.ts                    public exports

dashboard/
  src/pages/MapEditorPage.tsx → import { MapEditor } from '@loom/map-react'
                                (delete the iframe)

station-registration/         ← Tauri 2 app (replaces iced app)
  src-tauri/                  minimal Rust shell
    src/main.rs               tauri::Builder + commands
    src/commands.rs           get_local_ip, get_auth_token
    Cargo.toml                tauri, local-ip-address, clap
    tauri.conf.json
  src/                        React UI
    main.tsx
    App.tsx                   reads IP+token from Tauri commands, renders MapViewer with seat overlay
    overlay.tsx               connection-dot + team-name tooltip + click-to-claim handler
    client.ts                 connect-web client wired to --server URL
  package.json
  index.html
```

## Phased plan

### Phase 0 — Scaffolding

- Add `shared/map-react/` directory; add to `pnpm-workspace.yaml`.
- `package.json` deps: `react`, `react-dom`, `konva`, `react-konva`, `zustand`, `@bufbuild/protobuf`, `@loom/gen` (workspace), peer-deps for React.
- Set up `tsconfig.json`, `vitest`, basic `index.ts` exports.
- Decide build: keep as `.tsx` source consumed by Vite (no separate bundle step) — simplest for a workspace package.

### Phase 1 — Pure logic (no UI), with tests

Port these without React or Konva so they're trivially testable:

- `coords.ts`: `Point`, `Vector`, `Rotation` (mirror `gen/ts/v1/map/types_pb.ts`'s enum). Helpers: add/sub vectors, rotate.
- `geometry.ts`: `snapToGrid(p, units=10)`, `screenToWorld(p, offset, zoom)`, `worldToScreen`.
- `hit.ts`:
  - `wallHit(wall, point, threshold=5)` — segment distance squared (port `wall.rs:50-85`).
  - `doorHit(door, point, threshold=10)` — inverse-rotate point, jamb distance, leaf segment, hinge arc check (port `door.rs:95-124`).
  - `seatHit(seat, point)` — AABB on `position` + `getTotalBounds()` (port `seat.rs:103-109`).
- `proto.ts`: convert `Element` (the `pb.ts` oneof) ↔ a discriminated `MapElement` TS union: `{ kind: 'wall' | 'door' | 'seat', ... }`.
- Vitest: a handful of fixtures for hit-tests, especially the rotated door arc.

### Phase 2 — `<MapViewer>` (read-only)

- `Grid.tsx`: Konva `Layer` drawing vertical/horizontal lines at world coords, computed from `top_left`/`bottom_right` like `grid.rs:56-98`.
- `Wall.tsx`: dot at start, dot at end, line. Selection halo as wider semi-transparent line (port `wall.rs:13-43`).
- `Door.tsx`: rotated `Group` containing two jamb dots, leaf line, dashed quarter-arc. Selection halo similar (port `door.rs:13-88`).
- `Seat.tsx`: rotated `Group` containing chair arcs, table rectangle, laptop rect+screen line. Selection halo as background rect (port `seat.rs:11-96`).
- Stage with pan via mouse drag, zoom via wheel (port `grid.rs:212-238`).
- Props:
  ```ts
  type MapViewerProps = {
    elements: MapElement[];
    onElementClick?: (el: MapElement) => void;
    seatOverlay?: (seat: Seat) => ReactNode; // Konva node, rendered above the seat
  };
  ```

### Phase 3 — `<MapEditor>` (edit mode)

- zustand store: `{ elements: Map<id, MapElement>, startElements, selected: Set<id>, mode: 'view'|'edit', grid: { offset, zoom } }`. Actions mirror `loom_map::Message`.
- Selection: left-click hit-test → toggle; right-click in edit mode → toggle (matches current Rust behavior).
- Keyboard: Delete, Escape (clear selection), `c` (duplicate), `r` (rotate). Use `useEffect` global listener scoped to focused editor.
- Modifiers: shift+drag → draw wall; alt+drag → move selection. Track in store like `grid.rs:Interaction`.
- HUD: port the buttons from `map-editor/src/map.rs:94-167` (delete/clear/dup/rotate/new door/new seat/save). Tailwind, matches dashboard styling.
- `getChanges()` mirrors `Map::get_changes` — returns `{ deleted: string[], updated: MapElement[] }`.
- Props:
  ```ts
  type MapEditorProps = {
    initialElements: MapElement[];
    onSave: (changes: { deleted: string[]; updated: MapElement[] }) => Promise<void>;
  };
  ```

### Phase 4 — Dashboard integration

- Replace `MapEditorPage.tsx` body: load elements via existing `map_client.getMap`, render `<MapEditor>`, on save call `map_client.updateMap`.
- Delete the `/editor/` static-asset route from dashboard build/serve config (likely in `vite.config.ts` or backend's static handler).
- Verify in browser: load a map, edit a wall, save, reload — diff against the iced editor's output before deleting `map-editor/`.

### Phase 5 — Tauri station-registration

**Rust shell (`src-tauri/`):**
- `Cargo.toml`: `tauri = "2"`, `local-ip-address`, `clap` for `--server`/`--auth`.
- Two commands:
  ```rust
  #[tauri::command] fn get_local_ip(state: State<Args>) -> Result<String, String>
  #[tauri::command] fn get_auth_token(state: State<Args>) -> Option<String>
  ```
- Args parsed at startup, stored in Tauri state.

**React UI:**
- `App.tsx`: invoke Tauri commands once at mount to get IP and token; build a connect-web `Transport` pointing at `--server` URL with the token in metadata.
- `overlay.tsx`: subscribes (via `useEffect`) to `BroadcastService.subscribe` for `CONNECTION_STATE` and `ASSIGNMENT_STATE`. Renders a small Konva `Circle` (green/red) on each seat plus a hover tooltip (Konva `Label`).
- Click handler: `MapService.AssignStationToSeat({ seatId, stationIp: localIp })`.
- Renders `<MapViewer elements={...} seatOverlay={renderOverlay} onElementClick={handleClaim} />`.

**Backend CORS:** allow `tauri://localhost` (and `https://tauri.localhost` on Windows) for gRPC-Web. One-line config change on the backend.

### Phase 6 — Cleanup

- Delete `shared/map/`, `shared/client/`, `map-editor/`, old iced files in `station-registration/`.
- Drop wasm/iced/tonic-wasm deps from workspace `Cargo.toml`s.
- Update `flake.nix`: remove wasm target; add Node toolchain for Tauri build if not already there.
- Update Nix packaging: replace `loom-map-editor` derivation with the new Tauri bundle (consider `nix-tauri` or just a `pkgs.callPackage` wrapping `pnpm build` + `cargo build` for the Tauri shell).

## Things to watch

- **Rotation visuals.** Seat's `vertical_shift = CHAIR_PROTRUSION/2.0`, door's dashed quarter-arc, wall stroke widths scaled by `scale` (`= 1/zoom` in iced). Port literally and pixel-diff against iced output before deleting Rust crate.
- **Hover-driven tooltip cost.** The current `is_hit(cursor_pos)` runs on every iced render frame. In React, use Konva's `onMouseEnter`/`onMouseLeave` per shape (free hit-test) instead of running our own hit-test on `mousemove`.
- **gRPC-Web from Tauri webview.** Backend CORS must allowlist the Tauri origin. Worth checking before Phase 5 starts.
- **Auth token storage.** Currently passed as `--auth <token>` CLI arg; loomd already does the same on station boxes. Keep that interface for the Tauri app; Tauri command exposes it to the frontend without persisting.
- **Workspace package consumption.** dashboard's Vite dev server needs to resolve `@loom/map-react` from source (workspace symlink). Standard pnpm workspace setup handles this; just ensure `tsconfig.app.json` paths and Vite `optimizeDeps` don't choke on the workspace import.
- **Element IDs.** Rust uses `Uuid::now_v7()`. In TS use `crypto.randomUUID()` (v4 — fine, IDs are opaque to the backend) or pull a v7 lib if ordering matters anywhere (it doesn't currently — backend assigns nothing on UUIDs).
- **Selection halo z-order.** In Rust, `MapCanvas` draws halos as part of each element's `draw()`. In Konva, draw the halo as a separate child node beneath the element so selection state can change without recomputing the shape.

## Open items (not blocking)

- Will the Tauri app eventually need offline-first / cache the last-known map? Not in scope now, but if yes, IndexedDB caching layer in Phase 5.
- Hot-reload UX during contest: today loomd handles auto-reload; the Tauri app will need a similar restart-on-config-change story. Defer until needed.
