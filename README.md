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
| R | Continuously spawn random vehicles |
| Esc | Stop simulation and show statistics |

## Building and Running

Requires SDL2 development libraries. On WSL / Linux:

```bash
sudo apt-get install libsdl2-dev
cargo run
```

## Statistics

When you press Esc the simulation ends and displays:
- Max simultaneous vehicles in the intersection
- Max and min velocity reached across all vehicles
- Max and min time any vehicle took to cross the intersection
- Number of close calls (safety distance violations)

## Project Structure

```
src/
├── main.rs                 # Game loop, window, event handling
├── intersection/
│   ├── mod.rs              # Simulation state, safety distance logic
│   └── scheduler.rs        # Smart scheduling algorithm
├── vehicle/
│   ├── mod.rs              # Vehicle struct and physics
│   └── route.rs            # Path geometry and waypoints
├── renderer/
│   ├── mod.rs              # draw_road() and draw_vehicles()
│   ├── animation.rs        # Sprite rotation
│   └── assets.rs           # Image loading and caching
└── stats.rs                # Statistics collection and end screen
```

## Coordinate System

- Window: 800×800 px
- Intersection box: (300, 300) → (500, 500)
- Lane width: 40 px — 3 incoming lanes per road arm

## Physics

Each vehicle has three speed levels (px/s):

| Velocity | Speed |
|----------|-------|
| Slow | 60 |
| Medium | 120 |
| Fast | 200 |

The scheduler adjusts speed to avoid conflicts. Vehicles also automatically slow down when the gap to the vehicle ahead on the same lane drops below the safety distance (40 px).
