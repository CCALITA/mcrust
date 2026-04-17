use wgpu::util::DeviceExt;

/// Number of textures in the atlas (width in tiles).
const ATLAS_COLS: u32 = 16;
/// Number of rows in the atlas.
const ATLAS_ROWS: u32 = 8;
/// Pixels per tile side.
const TILE_SIZE: u32 = 16;
/// Total atlas width in pixels.
pub const ATLAS_WIDTH: u32 = ATLAS_COLS * TILE_SIZE;
/// Total atlas height in pixels.
pub const ATLAS_HEIGHT: u32 = ATLAS_ROWS * TILE_SIZE;

/// Total number of texture slots in the atlas.
pub const ATLAS_TILE_COUNT: u32 = ATLAS_COLS * ATLAS_ROWS;

/// Holds the GPU texture, sampler, and bind group for the block atlas.
pub struct TextureAtlas {
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl TextureAtlas {
    /// Create the procedural texture atlas and upload it to the GPU.
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let rgba_data = generate_atlas_rgba();

        let texture_size = wgpu::Extent3d {
            width: ATLAS_WIDTH,
            height: ATLAS_HEIGHT,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("block_atlas"),
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &rgba_data,
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("block_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("texture_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("texture_bind_group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        Self {
            bind_group,
            bind_group_layout,
        }
    }
}

/// UV coordinates for a given texture index in the atlas.
/// Returns (u_min, v_min, u_max, v_max).
pub fn atlas_uv(tex_index: u16) -> (f32, f32, f32, f32) {
    let col = (tex_index as u32) % ATLAS_COLS;
    let row = (tex_index as u32) / ATLAS_COLS;
    let u_min = col as f32 / ATLAS_COLS as f32;
    let v_min = row as f32 / ATLAS_ROWS as f32;
    let u_max = (col + 1) as f32 / ATLAS_COLS as f32;
    let v_max = (row + 1) as f32 / ATLAS_ROWS as f32;
    (u_min, v_min, u_max, v_max)
}

/// Generate the full RGBA pixel data for the atlas.
fn generate_atlas_rgba() -> Vec<u8> {
    let mut data = vec![0u8; (ATLAS_WIDTH * ATLAS_HEIGHT * 4) as usize];

    for idx in 0..ATLAS_TILE_COUNT {
        let col = idx % ATLAS_COLS;
        let row = idx / ATLAS_COLS;
        let color = tile_color(idx);
        fill_tile(&mut data, col, row, color);
    }

    data
}

/// Fill a single 16x16 tile at the given column and row with a base color
/// and some simple procedural variation for visual distinction.
fn fill_tile(data: &mut [u8], col: u32, row: u32, base: [u8; 4]) {
    let x0 = col * TILE_SIZE;
    let y0 = row * TILE_SIZE;

    for py in 0..TILE_SIZE {
        for px in 0..TILE_SIZE {
            let gx = x0 + px;
            let gy = y0 + py;
            let offset = ((gy * ATLAS_WIDTH + gx) * 4) as usize;

            // Simple noise-like variation based on position
            let noise = ((px.wrapping_mul(7) ^ py.wrapping_mul(13)) % 16) as i16 - 8;

            data[offset] = (base[0] as i16 + noise).clamp(0, 255) as u8;
            data[offset + 1] = (base[1] as i16 + noise).clamp(0, 255) as u8;
            data[offset + 2] = (base[2] as i16 + noise).clamp(0, 255) as u8;
            data[offset + 3] = base[3];
        }
    }
}

/// Map a texture index to a base RGBA color.
/// Indices match the tex_indices in mc-core BlockProperties.
fn tile_color(index: u32) -> [u8; 4] {
    match index {
        0 => [255, 0, 255, 0], // 0: air (magenta, transparent — should never be seen)
        1 => [128, 128, 128, 255], // 1: stone (gray)
        2 => [139, 90, 43, 255], // 2: dirt (brown)
        3 => [76, 153, 0, 255], // 3: grass_top (green)
        4 => [100, 140, 50, 255], // 4: grass_side (green-brown)
        5 => [50, 50, 50, 255], // 5: bedrock (dark gray)
        6 => [30, 100, 200, 180], // 6: water (blue, semi-transparent)
        7 => [220, 200, 130, 255], // 7: sand (yellow)
        8 => [140, 130, 120, 255], // 8: gravel (gray-brown)
        9 => [160, 130, 70, 255], // 9: oak_log_top (light brown ring)
        10 => [110, 80, 40, 255], // 10: oak_log_side (bark brown)
        11 => [50, 120, 30, 255], // 11: oak_leaves (dark green)
        12 => [180, 140, 80, 255], // 12: oak_planks (warm wood)
        13 => [110, 110, 110, 255], // 13: cobblestone (medium gray)
        14 => [90, 90, 90, 255], // 14: coal_ore (dark spots on stone)
        15 => [160, 140, 130, 255], // 15: iron_ore (beige spots on stone)
        16 => [200, 180, 60, 255], // 16: gold_ore (gold spots)
        17 => [80, 220, 220, 255], // 17: diamond_ore (cyan spots)
        18 => [200, 220, 240, 100], // 18: glass (light blue, semi-transparent)
        19 => [255, 200, 50, 255], // 19: torch (yellow-orange)
        20 => [160, 120, 70, 255], // 20: crafting_table_top
        21 => [150, 110, 60, 255], // 21: crafting_table_side
        22 => [100, 100, 100, 255], // 22: furnace_front (darker gray)
        23 => [120, 120, 120, 255], // 23: furnace_side
        24 => [140, 100, 50, 255], // 24: chest_top
        25 => [160, 110, 40, 255], // 25: chest_front (latch)
        26 => [130, 95, 45, 255], // 26: chest_side
        // --- New texture tiles ---
        27 => [200, 190, 160, 255], // 27: birch_log_top (pale ring)
        28 => [220, 210, 195, 255], // 28: birch_log_side (white bark)
        29 => [80, 140, 50, 255],   // 29: birch_leaves (bright green)
        30 => [195, 175, 120, 255], // 30: birch_planks (light wood)
        31 => [100, 80, 40, 255],   // 31: spruce_log_top (dark ring)
        32 => [60, 40, 20, 255],    // 32: spruce_log_side (dark bark)
        33 => [30, 80, 30, 255],    // 33: spruce_leaves (dark green)
        34 => [120, 90, 50, 255],   // 34: spruce_planks (dark wood)
        35 => [150, 110, 50, 255],  // 35: jungle_log_top (olive ring)
        36 => [90, 70, 30, 255],    // 36: jungle_log_side (brownish bark)
        37 => [40, 130, 20, 255],   // 37: jungle_leaves (lush green)
        38 => [170, 120, 70, 255],  // 38: jungle_planks (reddish wood)
        39 => [70, 50, 25, 255],    // 39: dark_oak_log_top (very dark ring)
        40 => [50, 35, 15, 255],    // 40: dark_oak_log_side (very dark bark)
        41 => [25, 90, 15, 255],    // 41: dark_oak_leaves (deep green)
        42 => [80, 55, 30, 255],    // 42: dark_oak_planks (dark wood)
        43 => [130, 100, 80, 255],  // 43: copper_ore (orange-brown spots)
        44 => [40, 60, 170, 255],   // 44: lapis_ore (blue spots)
        45 => [60, 190, 80, 255],   // 45: emerald_ore (green spots)
        46 => [180, 30, 30, 255],   // 46: redstone_ore (red spots)
        47 => [15, 10, 25, 255],    // 47: obsidian (very dark purple)
        48 => [240, 250, 255, 255], // 48: snow (white)
        49 => [230, 240, 250, 255], // 49: snow_block (slightly off-white)
        50 => [160, 200, 240, 140], // 50: ice (light blue, semi-transparent)
        51 => [140, 180, 220, 255], // 51: packed_ice (opaque blue-white)
        52 => [160, 165, 175, 255], // 52: clay (gray-blue)
        53 => [150, 95, 65, 255],   // 53: terracotta (earthy orange)
        54 => [180, 40, 40, 255],   // 54: red_wool
        55 => [40, 50, 180, 255],   // 55: blue_wool
        56 => [50, 150, 50, 255],   // 56: green_wool
        57 => [220, 210, 50, 255],  // 57: yellow_wool
        58 => [230, 230, 230, 255], // 58: white_wool
        59 => [25, 25, 25, 255],    // 59: black_wool
        60 => [20, 130, 40, 255],   // 60: cactus (dark green)
        61 => [110, 180, 70, 255],  // 61: sugar_cane (light green)
        62 => [200, 130, 20, 255],  // 62: pumpkin_top (orange)
        63 => [210, 140, 30, 255],  // 63: pumpkin_side (orange with ridges)
        64 => [120, 150, 40, 255],  // 64: melon_top (pale green)
        65 => [100, 140, 30, 255],  // 65: melon_side (striped green)
        66 => [200, 180, 150, 255], // 66: tnt_top (tan)
        67 => [170, 150, 120, 255], // 67: tnt_bottom (tan)
        68 => [200, 50, 40, 255],   // 68: tnt_side (red)
        69 => [100, 70, 40, 255],   // 69: bookshelf_side (books)
        70 => [90, 115, 90, 255],   // 70: mossy_cobblestone (gray-green)
        71 => [160, 80, 60, 255],   // 71: bricks (red-brown)
        72 => [120, 120, 120, 255], // 72: stone_bricks (light gray)
        73 => [110, 50, 50, 255],   // 73: netherrack (dark red)
        74 => [80, 65, 50, 255],    // 74: soul_sand (dark brown)
        75 => [220, 200, 120, 255], // 75: glowstone (bright yellow)
        76 => [220, 220, 170, 255], // 76: end_stone (pale yellow)
        77 => [130, 100, 130, 255], // 77: mycelium_top (purple-gray)
        78 => [120, 90, 120, 255],  // 78: mycelium_side (purple-gray-brown)
        79 => [90, 70, 30, 255],    // 79: podzol_top (dark brown)
        80 => [100, 80, 40, 255],   // 80: podzol_side (brown-dirt)
        81 => [200, 40, 40, 255],   // 81: red_mushroom
        82 => [160, 130, 90, 255],  // 82: brown_mushroom
        83 => [80, 160, 50, 255],   // 83: tall_grass (green)
        84 => [240, 220, 50, 255],  // 84: dandelion (yellow)
        85 => [220, 40, 40, 255],   // 85: poppy (red)
        _ => [200, 200, 200, 255],  // fallback: light gray
    }
}
