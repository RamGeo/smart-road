# Smart Road

A traffic intersection simulation for autonomous vehicles (AVs), built in Rust with SDL2.

Licensed under the [MIT License](LICENSE). There are no traffic lights — a smart scheduling algorithm controls each vehicle's velocity so they pass through the intersection without collisions and with minimal congestion.

Traditional intersections use signals designed for human drivers. Here, fully autonomous vehicles follow fixed routes and the scheduler coordinates their speed in real time — slowing, stopping, or releasing them through the intersection.

Each arm has three lanes with a fixed route:

| Lane | Route |
|------|-------|
| **r** | Turn right |
| **s** | Go straight |
| **l** | Turn left |

Vehicles cannot change lanes or routes once spawned.

## Features

- Smart scheduler with geometric path-conflict detection
- Four velocity levels and same-lane safety distance enforcement
- SVG vehicle sprites that rotate smoothly along waypoint paths
- Win95-style UI — title bar, HUD, lane markers, street signs, scenery
- Live stats overlay and end-of-session statistics screen
- Optional audio feedback for spawn and random-mode toggle
- 37 unit tests in `src/tests.rs`

## Controls

| Key | Action |
|-----|--------|
| **↑** | Spawn vehicle from South → heading North |
| **↓** | Spawn vehicle from North → heading South |
| **→** | Spawn vehicle from West → heading East |
| **←** | Spawn vehicle from East → heading West |
| **R** | Toggle continuous random spawning (every 0.8 s) |
| **Space** | Pause / resume simulation |
| **Shift** (hold) | Slow motion (25% speed) |
| **Esc** | Stop simulation and show statistics |
| **Esc** (again) | Close the application |

Spamming a spawn key is safe — a vehicle is only created when the lane entry point is clear. Route (r / s / l) is assigned randomly on spawn.

## Build & Run

Requires [Rust](https://rustup.rs/) and SDL2 + SDL2_ttf.

**Linux / WSL**
```bash
sudo apt-get install libsdl2-dev libsdl2-ttf-dev fonts-dejavu-core
cargo run
```

**macOS (Homebrew)**
```bash
brew install sdl2 sdl2_ttf
cargo run
```

macOS SDL2 linking is configured in `.cargo/config.toml` — no extra env vars needed.

## Docker

The image bundles SDL2, fonts, the release binary, and `assets/`. The build uses `WORKDIR /app` so runtime asset paths match what was compiled in.

If Docker denies access to the socket, prefix with `sudo` or add your user to the `docker` group (`sudo usermod -aG docker "$USER"`, then open a new terminal).

### Quick start (Linux / WSL)

Short helper scripts (work with **`docker.io`** — no Compose plugin required):

**Build:**

```bash
./scripts/docker-build.sh
```

**Run** (calls `xhost` for you, then starts the container with display + WSL2 audio):

```bash
./scripts/docker-run.sh
```

### With Docker Compose (optional)

Ubuntu’s `docker.io` package does **not** include `docker compose` by default. If `docker compose run` fails with `unknown flag: --rm` or `unknown command: docker compose`, either use the scripts above or install the plugin:

```bash
sudo apt install docker-compose-v2
```

Then:

```bash
xhost +local:docker
docker compose build
docker compose run --rm smart-road
```

Build and run in one step: `docker compose run --rm --build smart-road`

`docker-compose.yml` sets up the window (X11) and WSL2 audio (WSLg PulseAudio).

### Optional shell aliases

Add to `~/.bashrc` if you like shorter typing:

```bash
alias sr-build='./scripts/docker-build.sh'
alias sr-run='./scripts/docker-run.sh'
```

Or, if you installed `docker-compose-v2`:

```bash
alias sr-build='docker compose build'
alias sr-run='xhost +local:docker 2>/dev/null; docker compose run --rm smart-road'
```

### Troubleshooting

- **`unknown command: docker compose`** or **`unknown flag: --rm`** — use `./scripts/docker-run.sh` or install `docker-compose-v2` (see above).
- **No window** — run `xhost +local:docker` again; check `echo $DISPLAY` (often `:0` on WSL).
- **No sound** — confirm WSLg Pulse exists: `ls /mnt/wslg/PulseServer`. The run script mounts it when present.
- **Audio warning is OK** — the app still runs; `Audio::try_init` fails gracefully.

### Manual `docker run` (without Compose)

Display only:

```bash
docker run --rm \
  -e DISPLAY="$DISPLAY" \
  -v /tmp/.X11-unix:/tmp/.X11-unix:rw \
  smart-road:local
```

Display + WSL2 audio:

```bash
docker run --rm \
  -e DISPLAY="$DISPLAY" \
  -e SDL_AUDIODRIVER=pulse \
  -e PULSE_SERVER=unix:/mnt/wslg/PulseServer \
  -v /tmp/.X11-unix:/tmp/.X11-unix:rw \
  -v /mnt/wslg/PulseServer:/mnt/wslg/PulseServer:ro \
  smart-road:local
```

Plain `docker build -t smart-road:local .` also works if you prefer not to use Compose.

## Tests

```bash
cargo test --lib
```

All tests live in `src/tests.rs` and cover path geometry, vehicle physics, scheduler conflict rules, spawn throttling, safety distance, close-call detection, and statistics.

---

## How the Algorithm Works

Each simulation frame runs three passes in order:

### 1. Safety distance (same lane)

Vehicles on the same `(direction, route)` are sorted by distance travelled. If the gap to the vehicle ahead drops below **40 px**, the follower slows. If the leader is stopped, the follower stops too. A hard position clamp (**22 px**) prevents visual overlap regardless of frame timing.

### 2. Scheduler (intersection access)

As a vehicle approaches, it enters a **stop zone** (60 px before the intersection box). The scheduler then:

1. Marks the vehicle as detected — crossing-time clock starts here.
2. Checks whether any vehicle inside the box has a **geometrically conflicting path**.
3. If blocked → velocity set to `Stopped`.
4. If clear → vehicle released at `Fast` speed.
5. Only **one conflicting vehicle** released per frame — prevents simultaneous-entry collisions.

Path conflicts are precomputed by sampling all 12 `(direction, route)` waypoint paths. Right turns from different arms and parallel opposite straights (e.g. North straight vs South straight) typically do not block each other.

### 3. Physics

```
distance += speed × dt
```

Delta time uses `std::time::Instant`, so the simulation runs at correct speed regardless of frame rate.

| Velocity | Speed | When used |
|----------|-------|-----------|
| Stopped | 0 px/s | Waiting at stop line |
| Slow | 60 px/s | Following too close |
| Medium | 120 px/s | Normal travel (default on spawn) |
| Fast | 200 px/s | Released into intersection |

---

## Statistics

Press **Esc** to end the session. The statistics screen shows:

| Metric | Description |
|--------|-------------|
| Session duration | Total elapsed simulation time |
| Vehicles passed | Vehicles that completed their route |
| Max simultaneous | Peak vehicles inside the intersection at once |
| Max / min speed | Fastest and slowest non-zero speeds reached |
| Max / min crossing time | Longest and shortest scheduler-detection → exit times |
| Close calls | Conflicting pairs that passed within safe distance (counted once) |

---

## Project Structure

```
smart-road/
├── Cargo.toml
├── README.md
├── .cargo/
│   └── config.toml              # macOS SDL2 linker paths
│
├── assets/
│   ├── sprites/                 # Vehicle SVGs (red / blue / green / yellow)
│   ├── scenery/                 # Buildings, trees, park
│   ├── ui/                      # Grass tiles
│   └── icons/                   # Lane arrow icons
│
└── src/
    ├── main.rs                  # Entry point — game loop, input, stats screen
    ├── lib.rs                   # Library root
    ├── tests.rs                 # All unit tests (37)
    ├── stats.rs                 # Metrics collection + end-screen rendering
    ├── audio.rs                 # Procedural spawn / toggle sounds
    │
    ├── intersection/
    │   ├── mod.rs               # Simulation state, spawn logic, safety distance
    │   └── scheduler.rs         # Stop-zone scheduling + path-conflict map
    │
    ├── vehicle/
    │   ├── mod.rs               # Vehicle struct, physics, velocity levels
    │   └── route.rs             # 12 hardcoded waypoint paths, heading angles
    │
    └── renderer/
        ├── mod.rs               # Road, HUD, lane markers, street names
        ├── assets.rs            # SVG → SDL2 texture loading (resvg)
        └── animation.rs         # Sprite rotation helpers
```

### Coordinate system

| | Value |
|---|-------|
| Window | 1200 × 800 px |
| Intersection box | (500, 300) → (700, 500) |
| Lane width | 40 px — 3 lanes per arm |
| Safe distance | 40 px |
| Stop zone | 60 px before box edge |

Origin is top-left; y increases downward.

---

## Tech Stack

`sdl2` · `resvg` + `tiny-skia` (SVG assets) · `rand`

## License

This project is licensed under the [MIT License](LICENSE).

## Authors

Discord profile links use your numeric user ID ([how to copy your ID](https://support.discord.com/hc/en-us/articles/206346498-Where-can-I-find-my-User-Server-Message-ID)):

- **Stamatis Manousis** — [Discord](https://discordapp.com/users/350760622180270090)
- **Dilhan Aslamaci** — [Discord](https://discordapp.com/users/1277217326256881736)
- **Georgia Marouli** — [Discord](https://discordapp.com/users/1277216244910522371)
