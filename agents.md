# Smart Road — Agents Guide

## Project Overview

A Rust-based traffic intersection simulation for autonomous vehicles (AVs) with no traffic lights. Vehicles are managed by a smart intersection algorithm that controls velocity to prevent collisions and minimize congestion. The simulation includes a graphical animation, keyboard-driven vehicle generation, and end-of-session statistics.

---

## Architecture Overview

```
src/
├── main.rs           # Entry point, game loop, event handling
├── intersection/
│   ├── mod.rs        # Intersection state, smart algorithm
│   └── scheduler.rs  # Vehicle scheduling / collision avoidance logic
├── vehicle/
│   ├── mod.rs        # Vehicle struct, physics (velocity, distance, time)
│   └── route.rs      # Route enum (Right, Straight, Left) + path geometry
├── renderer/
│   ├── mod.rs        # SDL2/ggez rendering loop
│   ├── animation.rs  # Sprite rotation and frame management
│   └── assets.rs     # Asset loading
└── stats.rs          # Statistics collection and end-screen display
```

---

## Core Agents / Modules

### 1. Vehicle Agent

**File:** `src/vehicle/mod.rs`

Represents a single autonomous vehicle in the simulation.

**Responsibilities:**
- Store and update position, velocity, distance travelled, and elapsed time.
- Enforce a minimum safety distance behind the vehicle ahead on the same path.
- Expose three discrete velocity levels (e.g. slow / medium / fast) that the intersection scheduler can set.
- Track per-vehicle statistics: entry time, exit time, max/min velocity reached.

**Key types:**
```rust
pub enum Velocity { Stopped, Slow, Medium, Fast }  // Stopped = 0 px/s

pub struct Vehicle {
    pub id: u32,
    pub direction: Direction,
    pub route: Route,
    pub path: Path,             // pre-computed waypoint path
    pub distance_travelled: f32,
    pub velocity: Velocity,
    pub time_in_intersection: f32,
    pub entry_time: f32,
    pub close_call: bool,
}
```

**Rules:**
- Cannot change lanes or routes once spawned.
- Must decelerate (or stop) if the gap to the vehicle ahead is less than `SAFE_DISTANCE`.
- Removed from the canvas once it has fully exited the intersection area.

---

### 2. Route / Path Agent

**File:** `src/vehicle/route.rs`

Defines the geometry each vehicle follows through the intersection.

**Responsibilities:**
- Map `(Direction, Route)` → a sequence of waypoints through the intersection box.
- Provide the current heading angle at any point along the path (used by the renderer for sprite rotation).
- Detect which other paths share grid cells (used by the scheduler for conflict detection).

**Key types:**
```rust
pub enum Route     { Right, Straight, Left }
pub enum Direction { North, South, East, West }

pub struct Path {
    pub waypoints: Vec<(f32, f32)>,
}
```

---

### 3. Intersection Scheduler (Smart Algorithm)

**File:** `src/intersection/scheduler.rs`

The core "smart intersection management" agent. Replaces traditional traffic lights.

**Responsibilities:**
- Maintain a reservation table of intersection grid cells keyed by time slot.
- When a vehicle approaches, compute whether a safe time window exists for it to pass without conflict.
- Assign a velocity level to each vehicle so that it enters its reserved window on time.
- Detect and log "close calls" (any moment two vehicles violate `SAFE_DISTANCE` in the intersection zone).

**Algorithm sketch:**
1. Vehicle requests entry → scheduler projects its path at each supported velocity.
2. Scheduler checks the reservation table for conflicts on shared path cells.
3. If a conflict-free window is found, reserve it and instruct the vehicle to adjust velocity.
4. If no window is available, hold the vehicle at `Velocity::Slow` (or stopped) before the intersection entry line.
5. On vehicle exit, release reservations and update statistics.

**Constants to tune:**
```rust
const SAFE_DISTANCE: f32 = 40.0;        // pixels or world units
const GRID_CELL_SIZE: f32 = 16.0;       // reservation grid resolution
const TIME_SLOT_DURATION: f32 = 0.1;   // seconds per reservation slot
```

---

### 4. Intersection State

**File:** `src/intersection/mod.rs`

Owns the global simulation state and orchestrates all agents.

**Responsibilities:**
- Hold the list of active vehicles.
- Tick the physics of every vehicle each frame (`dt`-based update).
- Delegate scheduling decisions to the `Scheduler`.
- Collect per-frame statistics for the stats module.
- Expose `spawn_vehicle(direction)` called by the event handler.

---

### 5. Renderer / Animation Agent

**File:** `src/renderer/`

Draws the road, intersection, and vehicles each frame.

**Responsibilities:**
- Load and cache sprite sheets (`assets.rs`).
- For each vehicle, select the correct animation frame and rotate the sprite to match the vehicle's current heading (from `Path::heading_at(position)`).
- Render at a fixed logical resolution; scale to window size.
- Draw the statistics overlay after `Esc` is pressed.

**Animation rule:** When a vehicle turns, the sprite must rotate continuously to face the new heading — not snap. Interpolate the heading angle along the waypoint path.

---

### 6. Statistics Agent

**File:** `src/stats.rs`

Collects data throughout the run and renders the end screen.

**Tracked metrics:**
| Metric | Description |
|--------|-------------|
| Max vehicles | Peak simultaneous vehicle count |
| Max velocity | Highest speed any vehicle reached |
| Min velocity | Lowest speed any vehicle reached (excluding stopped) |
| Max time in intersection | Slowest vehicle to clear the intersection box |
| Min time in intersection | Fastest vehicle to clear the intersection box |
| Close calls | Count of safety-distance violations between any pair of vehicles |

**End screen:** Displayed in a modal window after `Esc`. Show each metric with its value. The simulation loop stops; the window closes when the user presses `Esc` again or clicks a close button.

---

## Event / Input Mapping

| Key | Action |
|-----|--------|
| Arrow Up | Spawn vehicle from South → heading North |
| Arrow Down | Spawn vehicle from North → heading South |
| Arrow Right | Spawn vehicle from West → heading East |
| Arrow Left | Spawn vehicle from East → heading West |
| R | Toggle continuous random vehicle generation each game-loop tick |
| Esc | Stop simulation, display statistics window |

**Spawn throttle:** Vehicles must not overlap on spawn. Before spawning, check that the tail of the last vehicle on the same incoming lane has cleared `SAFE_DISTANCE` from the spawn point.

---

## Physics Model

Every vehicle update step (called with delta time `dt` in seconds):

```
velocity_px_per_sec = match self.velocity {
    Slow   => SLOW_SPEED,
    Medium => MEDIUM_SPEED,
    Fast   => FAST_SPEED,
}

position = advance_along_path(position, velocity_px_per_sec * dt)
distance_travelled += velocity_px_per_sec * dt
time_in_intersection += dt   // only while inside the intersection box
```

Bonus (acceleration/deceleration): Instead of instant velocity changes, lerp the current speed toward the target speed using each vehicle's `acceleration` and `brake_force` parameters.

---

## Collision / Safety Distance

Safety distance is enforced at two levels:

1. **Same-path following distance** — checked by each vehicle against the vehicle immediately ahead on the same path segment.
2. **Cross-path close-call detection** — the scheduler checks all pairs of vehicles whose paths share intersection cells within the same time window. If two vehicles are simultaneously within `SAFE_DISTANCE` of a shared cell, it is a close call.

---

## Dependency Notes

- **Graphics library:** SDL2 (`sdl2` crate) — chosen and in use.
- **Asset format:** PNG sprite sheets. Vehicles face a canonical direction in the sheet; rotate in code.
- **Time:** Use the game loop's `dt` (delta time) rather than wall-clock calls inside update logic.
- **No threads required** for the simulation loop; keep everything single-threaded and driven by the main loop.

---

## Suggested Development Order

| # | Task | Status |
|---|------|--------|
| 1 | Define `Route`, `Direction`, `Path` with hardcoded waypoints and verify geometry | ✅ Done |
| 2 | Implement `Vehicle` physics (movement along path, safety distance enforcement) | ✅ Done |
| 3 | Build the `Renderer` — static road + moving coloured boxes (no sprites yet) | ✅ Done |
| 4 | Implement the `Scheduler` — stop-line conflict detection, deadlock prevention | ✅ Done |
| 5 | Wire up keyboard events, random routes, and spawn throttle | ✅ Done |
| 6 | Add road map image (replace drawn rectangles) | ⬜ Todo |
| 7 | Add vehicle sprite images and rotation animation | ⬜ Todo |
| 8 | Implement `Stats` collection and end-screen display | ⬜ Next |
| 9 | (Bonus) Add acceleration/deceleration physics | ⬜ Bonus |

---

## What Was Built — Step 1 & 2 Summary

### Step 1 — Geometry (`src/vehicle/route.rs`)
- `Route` enum: `Right`, `Straight`, `Left`
- `Direction` enum: `North`, `South`, `East`, `West`
- `Path` struct with hardcoded waypoints for all 12 `(Direction, Route)` combinations
- 800×800 coordinate system, intersection box `[300,300]→[500,500]`, lane width 40 px
- `position_at(t)` — world position at `t` pixels along the path
- `heading_at(t)` — heading angle in radians at `t` pixels along the path
- `total_length()` — full arc length of the path

### Step 2 — Physics & Safety Distance (`src/vehicle/mod.rs`, `src/intersection/mod.rs`)
- `Velocity` enum: `Slow` (60 px/s), `Medium` (120 px/s), `Fast` (200 px/s)
- `SAFE_DISTANCE = 40.0` px constant
- `Vehicle::update(dt)` advances `distance_travelled` and tracks `time_in_intersection`
- `Vehicle::in_intersection()` checks if the vehicle is inside the box
- `Vehicle::is_done()` returns true when the vehicle has exited the path
- `Intersection::apply_safety_distances()` — buckets vehicles by direction into index vecs, sorts descending by `distance_travelled`, walks leader/follower pairs and sets `Velocity::Slow` when gap < `SAFE_DISTANCE`, restores `Velocity::Medium` otherwise
- Called from `Intersection::update()` each frame before the physics tick

### Step 3 — Renderer (`src/renderer/mod.rs`, `src/main.rs`)
- SDL2 chosen as the graphics library (`sdl2 = "0.37"` in `Cargo.toml`)
- `draw_road(canvas)` — renders grass background, N-S and E-W road strips (280→520 px wide), intersection box, dashed yellow centre lines
- `draw_vehicles(canvas, vehicles)` — renders each vehicle as a 20×20 coloured square: Red=North, Blue=South, Green=East, Yellow=West
- Game loop in `main.rs`: event handling → `intersection.update(dt)` → clear → draw road → draw vehicles → present
- Real `dt` via `std::time::Instant` — no fixed frame rate assumption

### Step 4 — Scheduler (`src/intersection/scheduler.rs`)
- Stop-line zone: 60 px before the intersection box edge
- Vehicles approaching the stop line are held (`Velocity::Stopped`) if any conflicting vehicle is currently inside the box
- `paths_conflict()` — two vehicles conflict if they come from different directions AND at least one is not turning right (right turns stay in their corner and never cross)
- `newly_entering` list prevents multiple conflicting vehicles being released into the intersection on the same frame (solves simultaneous-release collision)
- Scheduler runs after `apply_safety_distances()` each frame so it always gets the final say on velocity

### Step 5 — Events, Routes, Spawn Throttle (`src/main.rs`, `src/intersection/mod.rs`)
- Correct key mapping: Up=South, Down=North, Right=West, Left=East
- R key toggles continuous random spawning (one vehicle every 0.8 s, random direction + route)
- `spawn_random_vehicle()` picks direction and route via `rand`
- `try_spawn()` checks all existing vehicles against the new vehicle's spawn point; skips if any vehicle is within `2 × SAFE_DISTANCE` (prevents stacking on key spam)
- `Velocity::Stopped` added (speed = 0) so the scheduler can fully halt vehicles at the stop line
- `VEHICLE_SIZE = 22.0` px — hard position clamp in `apply_safety_distances()` prevents two same-lane vehicles ever physically overlapping regardless of frame timing
- Cascade-stop bug fixed: safety distance no longer skips `Stopped` followers; the scheduler re-applies `Stopped` afterward if still needed, so queued vehicles always unblock when the leader is released

### Step 6 — Road Map Image (pending)
- Replace the drawn road rectangles in `draw_road()` with a pre-made PNG background image
- The image coordinate system must match the existing 800×800 layout (intersection box at [300,300]→[500,500])
- Load via `sdl2::image` crate (`sdl2` feature `"image"`) in `src/renderer/assets.rs`
- Suggested sources: limezu, finalbossblue, mobilegamegraphics, spriters-resource

### Step 7 — Vehicle Sprite Images (pending)
- Replace the 20×20 coloured squares with sprite images
- Each sprite should face a canonical direction (e.g. north-facing) and be rotated in code using `vehicle.heading()`
- Rotation is handled in `src/renderer/animation.rs` using `canvas.copy_ex()` with the angle in degrees
- Sprite sheet or individual PNGs per direction; loaded and cached in `src/renderer/assets.rs`
