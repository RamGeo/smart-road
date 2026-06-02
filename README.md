# Smart Road

A traffic intersection simulation for autonomous vehicles (AVs) built in Rust with SDL2. No traffic lights — a smart intersection algorithm controls vehicle velocities to prevent collisions and minimise congestion.

## Concept

Vehicles approach a cross intersection from all four directions. Each lane has a fixed route:
- **r** — turn right
- **s** — go straight
- **l** — turn left

A smart scheduling algorithm assigns velocities so vehicles pass without colliding, instead of relying on traffic lights.

## Controls

| Key | Action |
|-----|--------|
| Arrow Up | Spawn vehicle from South → heading North |
| Arrow Down | Spawn vehicle from North → heading South |
| Arrow Right | Spawn vehicle from West → heading East |
| Arrow Left | Spawn vehicle from East → heading West |
| R | Toggle continuous random vehicle generation |
| Esc | Stop simulation and show statistics |

Spamming a key is safe — a new vehicle is only spawned if the entry point for that lane is clear.

## Building and Running

Requires SDL2 development libraries. On WSL / Linux:

```bash
sudo apt-get install libsdl2-dev
cargo run
```

## How the Smart Algorithm Works

1. Each vehicle is assigned a random route (right / straight / left) on spawn.
2. As it approaches the intersection, it enters a **stop zone** (60 px before the box).
3. The scheduler checks whether any vehicle already inside has a **conflicting path**.
4. If blocked → the vehicle waits (`Stopped`).
5. If clear → the vehicle enters at `Medium` speed.
6. Only one conflicting group is released per frame, preventing simultaneous collisions.

Two right-turning vehicles from different arms are never blocked by each other — their paths do not cross.

## Safety Distance

- `SAFE_DISTANCE = 40 px` — if the gap to the vehicle ahead (same lane) drops below this, the follower slows to `Slow`.
- If the leader is fully stopped, the follower also stops.
- A hard position clamp (`VEHICLE_SIZE = 22 px`) ensures vehicles never visually overlap regardless of frame timing.

## Statistics

> **Pending (Step 8)** — the statistics screen is not yet implemented.

When complete, pressing Esc will display:
- Max simultaneous vehicles in the intersection
- Max and min velocity reached
- Max and min time any vehicle took to cross the intersection
- Number of close calls (safety distance violations)

## Project Structure

```
src/
├── main.rs                 # Game loop, window, event handling
├── intersection/
│   ├── mod.rs              # Simulation state, safety distance, spawn logic
│   └── scheduler.rs        # Smart scheduling algorithm
├── vehicle/
│   ├── mod.rs              # Vehicle struct and physics
│   └── route.rs            # Path geometry and waypoints
├── renderer/
│   ├── mod.rs              # draw_road() and draw_vehicles()
│   ├── animation.rs        # Sprite rotation (pending)
│   └── assets.rs           # Image loading (pending)
└── stats.rs                # Statistics collection and end screen (pending)
```

## Coordinate System

- Window: 800×800 px
- Intersection box: (300, 300) → (500, 500)
- Road width: 280 → 520 px (240 px, 20 px padding each side of outermost lane)
- Lane width: 40 px — 3 incoming lanes per road arm

## Physics

Each vehicle has four speed levels (px/s):

| Velocity | Speed | When used |
|----------|-------|-----------|
| Stopped | 0 | Waiting at stop line |
| Slow | 60 | Following too close |
| Medium | 120 | Normal travel |
| Fast | 200 | Reserved for scheduler optimisation |

Delta time (`dt`) is measured with `std::time::Instant` so the simulation runs at the correct physical speed regardless of frame rate.
