# MCRust

A Minecraft survival game built from scratch in Rust. No game engine -- just `wgpu`, `winit`, and ~40,000 lines of code across 106 modules and 10 crates.

**1,304 tests. All passing.**

## What works

### World Generation
- **Noise-based terrain** with biome-aware heightmaps (15 biomes: plains, forest, desert, ocean, mountains, taiga, swamp, jungle, savanna, tundra, birch forest, dark forest, beach, river, mushroom island)
- **Cave carving** with Perlin noise
- **Ore distribution** (coal, iron, gold, diamond, copper, lapis, emerald, redstone)
- **Tree generation** -- oak, birch, spruce, jungle, dark oak with species-specific shapes
- **Structure generation** -- villages, dungeons, mineshafts, temples
- **Three dimensions** -- Overworld, Nether (netherrack, soul sand, glowstone), End (end stone)
- **91 block types** with full property registry (hardness, light emission, transparency, per-face textures)

### Rendering
- **wgpu renderer** with Vulkan/Metal/DX12 backends
- **Frustum culling** to skip off-screen chunks
- **Face culling** -- only meshes faces between solid and transparent blocks
- **Procedural texture atlas** -- 107 texture slots generated at startup
- **Directional lighting** with ambient + sun diffuse (WGSL shaders)
- **Day/night cycle** with sky color transitions
- **Water rendering** with transparency and separate mesh pass
- **Particle system** (block break, environment)
- **Entity rendering** with animated mob models
- **Sky dome** renderer

### Physics & Interaction
- **AABB collision** detection and resolution
- **Gravity, jumping**, sprint, sneak movement
- **Raycasting** for block selection and hit detection
- **Fluid simulation** -- water flow and block updates
- **Explosions** with blast radius, block destruction, and entity damage
- **Fire spread** and burn mechanics
- **Climbable blocks** (ladders, vines, scaffolding)

### Entities & Combat
- **Entity-component system** with position, velocity, health, collider, gravity
- **AI system** with goal-based behavior (idle, wander, flee, attack, follow)
- **A* pathfinding** for mob navigation
- **Melee combat** with attack cooldown, knockback, damage calculation
- **Projectiles** -- arrows, snowballs, ender pearls with physics
- **Mob spawning** system with passive/hostile caps and biome-aware configs
- **Fall damage** calculation
- **Armor system** -- 4 slots, 5 materials, damage reduction
- **Status effects** -- speed, slowness, strength, jump boost, and more
- **Hunger & saturation** system with exhaustion from movement
- **Experience & levels** from mining, combat, and smelting
- **Taming & breeding** with baby mobs and feed mechanics
- **Villager trading** with professions, trade offers, and XP levels
- **Loot tables** for blocks and mobs with weighted pools and conditions
- **Raids** with wave-based illager attacks
- **Wither boss** with phase-based AI, wither skulls, and XP reward
- **Fishing** system with fish, junk, and treasure loot categories
- **Vehicles** -- boats and minecarts with movement physics
- **Item drops** and XP orbs from block/mob destruction
- **Decorations** -- banners, paintings with variant selection
- **Special mobs** -- enderman (teleport, stare aggro), witch (potion AI), slime (split on death)
- **Game statistics** and scoreboard tracking
- **Advancement system** with trigger-based progression

### Crafting & Items
- **78 item types** -- tools, food, materials, farming items
- **Crafting table** with shaped/shapeless recipe matching
- **Furnace** with fuel values and smelting recipes
- **Enchanting** system -- 30+ enchantments, level costs, category filtering
- **Brewing stand** with potion recipes and status effect manager
- **Anvil** -- combine, rename, merge enchantments with XP costs
- **Smithing table** with netherite upgrade recipes
- **Grindstone** -- disenchant and repair
- **Stonecutter** recipes and **loom** banner patterns
- **Inventory** management with 36 slots + hotbar
- **5 tool types** (pickaxe, axe, shovel, sword, hoe) x **5 tiers** (wood, stone, iron, gold, diamond)

### Redstone
- **Signal propagation** with power levels 0-15
- **14 redstone components** -- dust, torch, lever, button, repeater, comparator, piston, sticky piston, observer, hopper, dispenser, dropper, note block, redstone lamp
- **Block update queue** for cascading updates

### Farming
- **6 crop types** -- wheat, carrot, potato, beetroot, melon, pumpkin
- **Growth stages** with hydration-based tick speed
- **Farmland** moisture tracking

### World Features
- **Chunk streaming** -- async load/unload of 16x16x384 chunks by player distance
- **Block lighting** and sky light propagation
- **Weather system** -- clear, rain, thunderstorm transitions
- **World border** with configurable center and radius
- **Spawn management** with bed respawn
- **World save/load** via bincode serialization
- **Block entities** -- furnace, chest, hopper, brewing stand data
- **Containers** -- single/double chests, dispensers, hoppers with item transfer
- **Beacon** with pyramid scanning and tiered effects
- **Signs** with colored text
- **Map data** with block-to-color mapping
- **Rail system** -- normal, powered, detector rails with shape determination
- **Bucket** interactions -- water/lava placement, milk collection
- **Difficulty** scaling with regional difficulty

### Networking
- **Packet protocol** with binary encode/decode and framing
- **Client/server architecture** with connection management
- **Chat commands** parser with help system
- **Server configuration**

### Audio
- **Sound system** with spatial volume falloff
- **Music player** with dimension-aware track selection
- **Music discs** with jukebox events
- **Sound categories** (master, music, weather, blocks, hostile, players)

### UI
- **HUD renderer** with health, hunger, XP bars
- **Widget system** with draw commands, rects, and colors

## Architecture

10-crate Cargo workspace. Each crate owns one concern.

| Crate | Modules | Purpose |
|-------|---------|---------|
| `mc-core` | 6 | Block/item/biome registries, positions, directions, portals |
| `mc-world` | 32 | Chunks, terrain gen, biomes, caves, ores, structures, redstone, lighting, weather, rails, farming, fluid, fire, containers, save/load |
| `mc-entity` | 27 | Entities, AI, combat, armor, effects, hunger, XP, loot, spawning, pathfinding, projectiles, vehicles, villagers, raids, wither, fishing, taming, statistics, advancements |
| `mc-render` | 10 | wgpu renderer, meshing, camera, shaders, textures, sky, water, particles, entity models, frustum culling |
| `mc-craft` | 8 | Recipes, inventory, furnace, enchanting, brewing, anvil, smithing, workstations |
| `mc-network` | 5 | Packet protocol, client/server connections, chat commands |
| `mc-physics` | 3 | AABB collision, movement resolution, raycasting |
| `mc-audio` | 3 | Sound events, music player, music discs |
| `mc-ui` | 2 | HUD rendering, widget system |
| `mc-client` | 1 | Main binary -- ties everything together |
| **Total** | **97+** | **~40,000 lines of Rust** |

### Dependency graph

```
mc-core  <-- everything depends on this
  |-- mc-world     (+ noise, bincode)
  |-- mc-render    (+ mc-world, wgpu, winit, bytemuck, image)
  |-- mc-physics   (+ mc-world)
  |-- mc-craft
  |-- mc-audio     (+ rand)
  |-- mc-entity    (+ mc-world, rand)
  |-- mc-ui        (+ mc-craft)
  |-- mc-network   (+ bincode)
  '-- mc-client    (all crates + wgpu, winit, pollster)
```

## Feature coverage vs vanilla Minecraft

| Category | Implemented | Vanilla |
|----------|------------|---------|
| Block types | 91 | ~800 |
| Item types | 78 | ~1,400 |
| Biomes | 15 | ~60 |
| Dimensions | 3 | 3 |
| Tool types | 5 | 5 |
| Tool tiers | 5 | 6 (missing netherite) |
| Enchantments | 30+ | ~40 |
| Crops | 6 | ~10 |
| Redstone components | 14 | ~25 |
| Mob behaviors | AI, pathfinding, spawning, taming, breeding, raids | Full mob roster |
| Crafting stations | 7 (table, furnace, anvil, smithing, brewing, grindstone, stonecutter) | 10+ |
| Bosses | Wither | 3 |
| Vehicles | Boats, minecarts | Boats, minecarts |
| Villager professions | Multi-profession with trades | 15 professions |

## Tech stack

| Layer | Crate | Version | Role |
|-------|-------|---------|------|
| GPU | wgpu | 24 | Cross-platform rendering (Vulkan, Metal, DX12) |
| Window | winit | 0.30 | Window management with ApplicationHandler |
| Math | glam | 0.29 | SIMD-accelerated vectors and matrices |
| GPU data | bytemuck | 1 | Zero-copy casting for vertex/uniform buffers |
| Images | image | 0.25 | PNG texture loading |
| Terrain | noise | 0.9 | Perlin/simplex noise for world generation |
| Serialization | serde + bincode | 1 | World save/load and network packets |
| RNG | rand | 0.9 | Mob spawning, loot, world gen |
| Logging | env_logger | 0.11 | `RUST_LOG=debug cargo run` |
| Async | pollster | 0.4 | Block on wgpu futures |

Rust edition 2024. Zero runtime dependencies on game engines.

## Controls

| Key | Action |
|-----|--------|
| W / A / S / D | Move |
| Mouse | Look around |
| Space | Jump |
| Left Ctrl | Sprint |
| Left Shift | Sneak |
| Escape | Release cursor / Quit |

## Building

Requires Rust 1.85+ (edition 2024).

```bash
# Debug build
cargo build

# Release build (much faster rendering)
cargo build --release

# Run
cargo run --release

# Run all 1,304 tests
cargo test
```

Tested on macOS (Metal), Linux (Vulkan), and Windows (DX12/Vulkan).

## Roadmap

Development follows 8 milestones tracked as [GitHub milestones](https://github.com/CCALITA/mcrust/milestones).

| Phase | Name | Status |
|-------|------|--------|
| 1 | Foundation -- window, rendering, camera, physics | Done |
| 2 | World Generation -- biomes, caves, ores, trees, day/night | Done |
| 3 | Interaction -- block break/place, inventory, crafting | Done |
| 4 | Entities & Combat -- mobs, AI, survival mechanics | Done |
| 5 | Advanced World -- Nether, End, structures, boss fights | Done |
| 6 | Redstone & Crafting -- enchanting, brewing, redstone | Done |
| 7 | Multiplayer -- server-client, packet protocol | In progress |
| 8 | Polish -- particles, water, UI, performance | In progress |

## License

MIT
