//! Item ID constants for crafting recipes.
//!
//! These mirror conceptual Minecraft items. `N` values are arbitrary
//! identifiers; they will be reconciled with `mc_core::item::ItemId` later.

use crate::SlotItem;

// ── Basic items ────────────────────────────────────────────────────────────
pub const ITEM_OAK_LOG: SlotItem = 100;
pub const ITEM_OAK_PLANKS: SlotItem = 101;
pub const ITEM_STICK: SlotItem = 102;
pub const ITEM_CRAFTING_TABLE: SlotItem = 103;
pub const ITEM_COBBLESTONE: SlotItem = 104;
pub const ITEM_FURNACE: SlotItem = 105;
pub const ITEM_CHEST: SlotItem = 106;
pub const ITEM_COAL: SlotItem = 107;
pub const ITEM_TORCH: SlotItem = 108;
pub const ITEM_IRON_INGOT: SlotItem = 109;
pub const ITEM_GOLD_INGOT: SlotItem = 110;
pub const ITEM_DIAMOND: SlotItem = 111;

// ── Tool items ─────────────────────────────────────────────────────────────
pub const ITEM_WOODEN_PICKAXE: SlotItem = 200;
pub const ITEM_WOODEN_AXE: SlotItem = 201;
pub const ITEM_WOODEN_SHOVEL: SlotItem = 202;
pub const ITEM_WOODEN_SWORD: SlotItem = 203;
pub const ITEM_STONE_PICKAXE: SlotItem = 210;
pub const ITEM_STONE_AXE: SlotItem = 211;
pub const ITEM_STONE_SHOVEL: SlotItem = 212;
pub const ITEM_STONE_SWORD: SlotItem = 213;
pub const ITEM_IRON_PICKAXE: SlotItem = 220;
pub const ITEM_IRON_AXE: SlotItem = 221;
pub const ITEM_IRON_SHOVEL: SlotItem = 222;
pub const ITEM_IRON_SWORD: SlotItem = 223;
pub const ITEM_DIAMOND_PICKAXE: SlotItem = 230;
pub const ITEM_DIAMOND_AXE: SlotItem = 231;
pub const ITEM_DIAMOND_SHOVEL: SlotItem = 232;
pub const ITEM_DIAMOND_SWORD: SlotItem = 233;

// ── Armor items ────────────────────────────────────────────────────────────
pub const ITEM_LEATHER: SlotItem = 112;
pub const ITEM_LEATHER_HELMET: SlotItem = 300;
pub const ITEM_LEATHER_CHESTPLATE: SlotItem = 301;
pub const ITEM_LEATHER_LEGGINGS: SlotItem = 302;
pub const ITEM_LEATHER_BOOTS: SlotItem = 303;
pub const ITEM_IRON_HELMET: SlotItem = 310;
pub const ITEM_IRON_CHESTPLATE: SlotItem = 311;
pub const ITEM_IRON_LEGGINGS: SlotItem = 312;
pub const ITEM_IRON_BOOTS: SlotItem = 313;
pub const ITEM_GOLD_HELMET: SlotItem = 320;
pub const ITEM_GOLD_CHESTPLATE: SlotItem = 321;
pub const ITEM_GOLD_LEGGINGS: SlotItem = 322;
pub const ITEM_GOLD_BOOTS: SlotItem = 323;
pub const ITEM_DIAMOND_HELMET: SlotItem = 330;
pub const ITEM_DIAMOND_CHESTPLATE: SlotItem = 331;
pub const ITEM_DIAMOND_LEGGINGS: SlotItem = 332;
pub const ITEM_DIAMOND_BOOTS: SlotItem = 333;

// ── Building items ─────────────────────────────────────────────────────────
pub const ITEM_OAK_STAIRS: SlotItem = 400;
pub const ITEM_COBBLESTONE_STAIRS: SlotItem = 401;
pub const ITEM_OAK_SLAB: SlotItem = 402;
pub const ITEM_COBBLESTONE_SLAB: SlotItem = 403;
pub const ITEM_OAK_FENCE: SlotItem = 404;
pub const ITEM_OAK_FENCE_GATE: SlotItem = 405;
pub const ITEM_OAK_DOOR: SlotItem = 406;
pub const ITEM_OAK_TRAPDOOR: SlotItem = 407;
pub const ITEM_LADDER: SlotItem = 408;
pub const ITEM_OAK_SIGN: SlotItem = 409;
pub const ITEM_WOOL: SlotItem = 410;
pub const ITEM_BED: SlotItem = 411;

// ── Utility items ──────────────────────────────────────────────────────────
pub const ITEM_BUCKET: SlotItem = 500;
pub const ITEM_REDSTONE_DUST: SlotItem = 501;
pub const ITEM_COMPASS: SlotItem = 502;
pub const ITEM_CLOCK: SlotItem = 503;
pub const ITEM_SHEARS: SlotItem = 504;
pub const ITEM_FISHING_ROD: SlotItem = 505;
pub const ITEM_STRING: SlotItem = 506;
pub const ITEM_BOOKSHELF: SlotItem = 507;
pub const ITEM_BOOK: SlotItem = 508;
pub const ITEM_PAPER: SlotItem = 509;
pub const ITEM_SUGAR_CANE: SlotItem = 510;
pub const ITEM_GUNPOWDER: SlotItem = 511;
pub const ITEM_SAND: SlotItem = 512;
pub const ITEM_TNT: SlotItem = 513;
pub const ITEM_PUMPKIN: SlotItem = 514;
pub const ITEM_JACK_O_LANTERN: SlotItem = 515;

// ── Weapon items ───────────────────────────────────────────────────────────
pub const ITEM_BOW: SlotItem = 600;
pub const ITEM_ARROW: SlotItem = 601;
pub const ITEM_FLINT: SlotItem = 602;
pub const ITEM_FEATHER: SlotItem = 603;
pub const ITEM_SHIELD: SlotItem = 604;

// ── Redstone items ─────────────────────────────────────────────────────────
pub const ITEM_REDSTONE_TORCH: SlotItem = 700;
pub const ITEM_REPEATER: SlotItem = 701;
pub const ITEM_COMPARATOR: SlotItem = 702;
pub const ITEM_QUARTZ: SlotItem = 703;
pub const ITEM_PISTON: SlotItem = 704;
pub const ITEM_OBSERVER: SlotItem = 705;
pub const ITEM_DISPENSER: SlotItem = 706;
pub const ITEM_DROPPER: SlotItem = 707;
pub const ITEM_HOPPER: SlotItem = 708;
pub const ITEM_LEVER: SlotItem = 709;
pub const ITEM_STONE_BUTTON: SlotItem = 710;
pub const ITEM_STONE: SlotItem = 711;

// ── Misc items ─────────────────────────────────────────────────────────────
pub const ITEM_NOTE_BLOCK: SlotItem = 800;
pub const ITEM_RAIL: SlotItem = 801;
pub const ITEM_PAINTING: SlotItem = 802;
pub const ITEM_ITEM_FRAME: SlotItem = 803;
pub const ITEM_FLOWER_POT: SlotItem = 804;
pub const ITEM_BRICK: SlotItem = 805;
