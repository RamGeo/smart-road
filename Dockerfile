# --- Build: compile release binary (assets path baked as /app/assets) ---
FROM rust:1-bookworm AS builder

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        libsdl2-dev \
        libsdl2-ttf-dev \
        pkg-config \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY assets ./assets

RUN cargo build --release

# --- Runtime: SDL2, fonts, binary, and assets at the same paths as build ---
FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        libsdl2-2.0-0 \
        libsdl2-ttf-2.0-0 \
        libpulse0 \
        fonts-dejavu-core \
    && rm -rf /var/lib/apt/lists/*

# Prefer PulseAudio when PULSE_SERVER is set (e.g. WSLg mount); avoids ALSA "Unknown PCM default"
ENV SDL_AUDIODRIVER=pulse

COPY --from=builder /app/target/release/smart-road /app/smart-road
COPY assets ./assets

ENTRYPOINT ["/app/smart-road"]
