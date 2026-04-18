# MCRust - Minecraft in Rust

## Build

```bash
cargo build          # debug build
cargo build --release  # release build (much faster rendering)
cargo run --release    # run release
cargo test             # run all tests (~1700)
```

Requires Rust 1.75+ (tested on 1.94.1, edition 2024).

## Worktree Development (Important)

When spawning parallel agents in isolated git worktrees, **all worktrees share a single `target/` directory** via `.cargo/config.toml`:

```toml
[build]
target-dir = "/Users/fanxiyao.3/workspace/mcrust/target"
```

This prevents each worktree from creating its own ~1.2 GB build artifact directory. Without this, 10 parallel agents would consume ~12 GB of disk just for build artifacts.

**Rules for agents in worktrees:**
- Use `cargo check` instead of `cargo build` — it's faster and avoids lock contention on the shared target directory
- Only one `cargo build` can run at a time due to the shared target lock
- `cargo check` can run in parallel (it acquires a read lock)
- Never delete or modify `.cargo/config.toml`

## Architecture

10-crate Cargo workspace:

| Crate | Purpose |
|-------|---------|
| `mc-core` | Block/item/biome registries, positions, directions, portals |
| `mc-world` | Chunks, terrain gen, caves, ores, structures, redstone, weather, farming, save/load |
| `mc-entity` | ECS, mobs, AI, combat, survival, armor, projectiles, vehicles, villagers, bosses |
| `mc-craft` | Recipes, inventory, furnace, enchanting, brewing, anvil, smithing |
| `mc-render` | wgpu renderer, meshing, camera, shaders, sky, particles, water |
| `mc-physics` | AABB collision, movement, raycasting |
| `mc-audio` | Sound events, music player, disc playback |
| `mc-network` | Packet protocol, TCP server, chat commands |
| `mc-ui` | HUD data model, widget system |
| `mc-client` | Game binary + bridge modules connecting all systems |

## Conventions

- Explicit imports, no wildcards
- One domain concept per file
- `#[cfg(test)] mod tests { use super::*; }` for unit tests
- No `unwrap()` in production code
- Use `glam` for math, `bytemuck` for GPU data
- `#[repr(C)]` + Pod/Zeroable on all GPU structs
