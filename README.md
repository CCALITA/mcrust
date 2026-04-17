# MCRust

A Minecraft survival game built from scratch in Rust. No game engine — just wgpu, winit, and ~2,000 lines of code.

## What works right now

- **Flat voxel world** with textured grass, dirt, stone, and bedrock
- **First-person camera** — WASD movement, mouse look, sprint (Ctrl), sneak (Shift)
- **Physics** — gravity, jumping, AABB collision against blocks
- **Chunk streaming** — loads/unloads 16x16x384 chunks based on player distance
- **Face culling** — only renders faces between solid and transparent blocks
- **Procedural texture atlas** — 27 distinct block textures generated at startup
- **Directional lighting** — WGSL shader with ambient + sun diffuse

```
cargo run
```

You'll spawn on a flat world at y=64. Walk around. Jump with Space. Escape releases the cursor; press Escape again to quit.

## Architecture

10-crate Cargo workspace. Each crate owns one concern.

```
mcrust/
├── mc-core       Block registry, positions, directions — no dependencies
├── mc-world      Chunks, terrain generation, chunk manager
├── mc-render     wgpu renderer, meshing, camera, shaders, texture atlas
├── mc-physics    AABB collision, player movement resolution
├── mc-entity     (planned) ECS, mobs, AI
├── mc-craft      (planned) Recipes, inventory, crafting stations
├── mc-network    (planned) QUIC multiplayer via quinn
├── mc-audio      (planned) Spatial audio via kira
├── mc-ui         (planned) HUD, menus, inventory screens
└── mc-client     Main binary — ties everything together
```

### Dependency graph

```
mc-core  ← everything depends on this
  ├── mc-world
  ├── mc-render  (+ mc-world)
  ├── mc-physics (+ mc-world)
  ├── mc-craft
  ├── mc-audio
  ├── mc-entity  (+ mc-world)
  ├── mc-ui      (+ mc-craft)
  ├── mc-network (+ mc-world, mc-entity)
  └── mc-client  (all crates)
```

Crates without shared dependencies can be developed in parallel.

## Tech stack

| What | Crate | Why |
|------|-------|-----|
| Rendering | wgpu 24 | Cross-platform GPU (Vulkan, Metal, DX12) |
| Windowing | winit 0.30 | Standard Rust windowing with ApplicationHandler |
| Math | glam 0.29 | SIMD-accelerated vectors and matrices |
| GPU data | bytemuck | Zero-copy casting for vertex/uniform buffers |
| Terrain noise | noise 0.9 | Perlin/simplex for world generation |
| Logging | env_logger | `RUST_LOG=debug cargo run` for verbose output |

Planned: bevy_ecs (standalone ECS), quinn (QUIC networking), kira (spatial audio), rapier3d (advanced physics).

## Roadmap

Development follows 8 milestones tracked as [GitHub milestones](https://github.com/CCALITA/mcrust/milestones).

| Phase | Name | Status |
|-------|------|--------|
| 1 | Foundation — window, rendering, camera, physics | **Done** |
| 2 | World Generation — biomes, caves, ores, trees, day/night | Next |
| 3 | Interaction — block break/place, inventory, crafting | Planned |
| 4 | Entities & Combat — mobs, AI, survival mechanics | Planned |
| 5 | Advanced World — Nether, End, structures, boss fights | Planned |
| 6 | Redstone & Crafting — enchanting, brewing, redstone | Planned |
| 7 | Multiplayer — server-client, QUIC networking | Planned |
| 8 | Polish — particles, water, UI, performance | Planned |

## Building

Requires Rust 1.75+ (tested on 1.94.1).

```bash
# Debug build
cargo build

# Release build (much faster rendering)
cargo build --release

# Run
cargo run --release

# Run tests
cargo test
```

macOS, Linux, and Windows are supported through wgpu's backend abstraction.

## Controls

| Key | Action |
|-----|--------|
| W/A/S/D | Move |
| Mouse | Look |
| Space | Jump |
| Ctrl | Sprint |
| Shift | Sneak |
| Escape | Release cursor / Quit |

## License

MIT
