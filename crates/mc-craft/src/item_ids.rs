//! Item ID constants for crafting recipes.
//!
//! These mirror conceptual Minecraft items. `SlotItem(N)` values are arbitrary
//! identifiers; they will be reconciled with `mc_core::item::ItemId` later.

use crate::SlotItem;

// ── Basic items ────────────────────────────────────────────────────────────
pub const ITEM_OAK_LOG: SlotItem = SlotItem(100);
pub const ITEM_OAK_PLANKS: SlotItem = SlotItem(101);
pub const ITEM_STICK: SlotItem = SlotItem(102);
pub const ITEM_CRAFTING_TABLE: SlotItem = SlotItem(103);
pub const ITEM_COBBLESTONE: SlotItem = SlotItem(104);
pub const ITEM_FURNACE: SlotItem = SlotItem(105);
pub const ITEM_CHEST: SlotItem = SlotItem(106);
pub const ITEM_COAL: SlotItem = SlotItem(107);
pub const ITEM_TORCH: SlotItem = SlotItem(108);
pub const ITEM_IRON_INGOT: SlotItem = SlotItem(109);
pub const ITEM_GOLD_INGOT: SlotItem = SlotItem(110);
pub const ITEM_DIAMOND: SlotItem = SlotItem(111);

// ── Tool items ─────────────────────────────────────────────────────────────
pub const ITEM_WOODEN_PICKAXE: SlotItem = SlotItem(200);
pub const ITEM_WOODEN_AXE: SlotItem = SlotItem(201);
pub const ITEM_WOODEN_SHOVEL: SlotItem = SlotItem(202);
pub const ITEM_WOODEN_SWORD: SlotItem = SlotItem(203);
pub const ITEM_STONE_PICKAXE: SlotItem = SlotItem(210);
pub const ITEM_STONE_AXE: SlotItem = SlotItem(211);
pub const ITEM_STONE_SHOVEL: SlotItem = SlotItem(212);
pub const ITEM_STONE_SWORD: SlotItem = SlotItem(213);
pub const ITEM_IRON_PICKAXE: SlotItem = SlotItem(220);
pub const ITEM_IRON_AXE: SlotItem = SlotItem(221);
pub const ITEM_IRON_SHOVEL: SlotItem = SlotItem(222);
pub const ITEM_IRON_SWORD: SlotItem = SlotItem(223);
pub const ITEM_DIAMOND_PICKAXE: SlotItem = SlotItem(230);
pub const ITEM_DIAMOND_AXE: SlotItem = SlotItem(231);
pub const ITEM_DIAMOND_SHOVEL: SlotItem = SlotItem(232);
pub const ITEM_DIAMOND_SWORD: SlotItem = SlotItem(233);

// ── Armor items ────────────────────────────────────────────────────────────
pub const ITEM_LEATHER: SlotItem = SlotItem(112);
pub const ITEM_LEATHER_HELMET: SlotItem = SlotItem(300);
pub const ITEM_LEATHER_CHESTPLATE: SlotItem = SlotItem(301);
pub const ITEM_LEATHER_LEGGINGS: SlotItem = SlotItem(302);
pub const ITEM_LEATHER_BOOTS: SlotItem = SlotItem(303);
pub const ITEM_IRON_HELMET: SlotItem = SlotItem(310);
pub const ITEM_IRON_CHESTPLATE: SlotItem = SlotItem(311);
pub const ITEM_IRON_LEGGINGS: SlotItem = SlotItem(312);
pub const ITEM_IRON_BOOTS: SlotItem = SlotItem(313);
pub const ITEM_GOLD_HELMET: SlotItem = SlotItem(320);
pub const ITEM_GOLD_CHESTPLATE: SlotItem = SlotItem(321);
pub const ITEM_GOLD_LEGGINGS: SlotItem = SlotItem(322);
pub const ITEM_GOLD_BOOTS: SlotItem = SlotItem(323);
pub const ITEM_DIAMOND_HELMET: SlotItem = SlotItem(330);
pub const ITEM_DIAMOND_CHESTPLATE: SlotItem = SlotItem(331);
pub const ITEM_DIAMOND_LEGGINGS: SlotItem = SlotItem(332);
pub const ITEM_DIAMOND_BOOTS: SlotItem = SlotItem(333);

// ── Building items ─────────────────────────────────────────────────────────
pub const ITEM_OAK_STAIRS: SlotItem = SlotItem(400);
pub const ITEM_COBBLESTONE_STAIRS: SlotItem = SlotItem(401);
pub const ITEM_OAK_SLAB: SlotItem = SlotItem(402);
pub const ITEM_COBBLESTONE_SLAB: SlotItem = SlotItem(403);
pub const ITEM_OAK_FENCE: SlotItem = SlotItem(404);
pub const ITEM_OAK_FENCE_GATE: SlotItem = SlotItem(405);
pub const ITEM_OAK_DOOR: SlotItem = SlotItem(406);
pub const ITEM_OAK_TRAPDOOR: SlotItem = SlotItem(407);
pub const ITEM_LADDER: SlotItem = SlotItem(408);
pub const ITEM_OAK_SIGN: SlotItem = SlotItem(409);
pub const ITEM_WOOL: SlotItem = SlotItem(410);
pub const ITEM_BED: SlotItem = SlotItem(411);

// ── Utility items ──────────────────────────────────────────────────────────
pub const ITEM_BUCKET: SlotItem = SlotItem(500);
pub const ITEM_REDSTONE_DUST: SlotItem = SlotItem(501);
pub const ITEM_COMPASS: SlotItem = SlotItem(502);
pub const ITEM_CLOCK: SlotItem = SlotItem(503);
pub const ITEM_SHEARS: SlotItem = SlotItem(504);
pub const ITEM_FISHING_ROD: SlotItem = SlotItem(505);
pub const ITEM_STRING: SlotItem = SlotItem(506);
pub const ITEM_BOOKSHELF: SlotItem = SlotItem(507);
pub const ITEM_BOOK: SlotItem = SlotItem(508);
pub const ITEM_PAPER: SlotItem = SlotItem(509);
pub const ITEM_SUGAR_CANE: SlotItem = SlotItem(510);
pub const ITEM_GUNPOWDER: SlotItem = SlotItem(511);
pub const ITEM_SAND: SlotItem = SlotItem(512);
pub const ITEM_TNT: SlotItem = SlotItem(513);
pub const ITEM_PUMPKIN: SlotItem = SlotItem(514);
pub const ITEM_JACK_O_LANTERN: SlotItem = SlotItem(515);

// ── Weapon items ───────────────────────────────────────────────────────────
pub const ITEM_BOW: SlotItem = SlotItem(600);
pub const ITEM_ARROW: SlotItem = SlotItem(601);
pub const ITEM_FLINT: SlotItem = SlotItem(602);
pub const ITEM_FEATHER: SlotItem = SlotItem(603);
pub const ITEM_SHIELD: SlotItem = SlotItem(604);

// ── Redstone items ─────────────────────────────────────────────────────────
pub const ITEM_REDSTONE_TORCH: SlotItem = SlotItem(700);
pub const ITEM_REPEATER: SlotItem = SlotItem(701);
pub const ITEM_COMPARATOR: SlotItem = SlotItem(702);
pub const ITEM_QUARTZ: SlotItem = SlotItem(703);
pub const ITEM_PISTON: SlotItem = SlotItem(704);
pub const ITEM_OBSERVER: SlotItem = SlotItem(705);
pub const ITEM_DISPENSER: SlotItem = SlotItem(706);
pub const ITEM_DROPPER: SlotItem = SlotItem(707);
pub const ITEM_HOPPER: SlotItem = SlotItem(708);
pub const ITEM_LEVER: SlotItem = SlotItem(709);
pub const ITEM_STONE_BUTTON: SlotItem = SlotItem(710);
pub const ITEM_STONE: SlotItem = SlotItem(711);

// ── Misc items ─────────────────────────────────────────────────────────────
pub const ITEM_NOTE_BLOCK: SlotItem = SlotItem(800);
pub const ITEM_RAIL: SlotItem = SlotItem(801);
pub const ITEM_PAINTING: SlotItem = SlotItem(802);
pub const ITEM_ITEM_FRAME: SlotItem = SlotItem(803);
pub const ITEM_FLOWER_POT: SlotItem = SlotItem(804);
pub const ITEM_BRICK: SlotItem = SlotItem(805);
