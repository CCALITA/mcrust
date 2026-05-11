//! Chiseled bookshelf storage block with 6 slots for book items.

/// A chiseled bookshelf that stores up to 6 books.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChiseledBookshelf {
    /// Six slots, each optionally holding a book item ID.
    pub slots: [Option<u16>; 6],
    /// Index of the last slot interacted with (0..5).
    pub last_interacted: u8,
}

impl ChiseledBookshelf {
    /// Creates an empty chiseled bookshelf.
    #[must_use]
    pub fn new() -> Self {
        Self {
            slots: [None; 6],
            last_interacted: 0,
        }
    }
}

/// Valid book item IDs.
const BOOK_ITEMS: &[u16] = &[
    770,  // book
    771,  // written_book
    772,  // writable_book
    773,  // enchanted_book
    774,  // knowledge_book
];

/// Attempts to add a book to the specified slot. Returns true if successful.
#[must_use]
pub fn add_book(shelf: &mut ChiseledBookshelf, slot: u8, book_id: u16) -> bool {
    if slot >= 6 || !is_valid_book_item(book_id) {
        return false;
    }
    if shelf.slots[slot as usize].is_some() {
        return false;
    }
    shelf.slots[slot as usize] = Some(book_id);
    shelf.last_interacted = slot;
    true
}

/// Removes a book from the specified slot. Returns the book item ID if present.
#[must_use]
pub fn remove_book(shelf: &mut ChiseledBookshelf, slot: u8) -> Option<u16> {
    if slot >= 6 {
        return None;
    }
    let book = shelf.slots[slot as usize].take();
    if book.is_some() {
        shelf.last_interacted = slot;
    }
    book
}

/// Returns the comparator output signal based on the last interacted slot.
/// Output is `last_interacted + 1` if that slot is occupied, otherwise 0.
#[must_use]
pub fn bookshelf_comparator_output(shelf: &ChiseledBookshelf) -> u8 {
    if shelf.slots[shelf.last_interacted as usize].is_some() {
        shelf.last_interacted + 1
    } else {
        0
    }
}

/// Returns whether the given item ID is a valid book for a chiseled bookshelf.
#[must_use]
pub fn is_valid_book_item(item_id: u16) -> bool {
    BOOK_ITEMS.contains(&item_id)
}

/// Returns the number of occupied slots.
#[must_use]
pub fn occupied_slot_count(shelf: &ChiseledBookshelf) -> u8 {
    shelf.slots.iter().filter(|s| s.is_some()).count() as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_bookshelf_is_empty() {
        let shelf = ChiseledBookshelf::new();
        assert_eq!(occupied_slot_count(&shelf), 0);
        assert_eq!(shelf.last_interacted, 0);
    }

    #[test]
    fn test_add_book_valid() {
        let mut shelf = ChiseledBookshelf::new();
        assert!(add_book(&mut shelf, 0, 770));
        assert_eq!(shelf.slots[0], Some(770));
        assert_eq!(shelf.last_interacted, 0);
    }

    #[test]
    fn test_add_book_invalid_item() {
        let mut shelf = ChiseledBookshelf::new();
        assert!(!add_book(&mut shelf, 0, 999));
        assert_eq!(shelf.slots[0], None);
    }

    #[test]
    fn test_add_book_slot_occupied() {
        let mut shelf = ChiseledBookshelf::new();
        add_book(&mut shelf, 2, 771);
        assert!(!add_book(&mut shelf, 2, 772));
        assert_eq!(shelf.slots[2], Some(771));
    }

    #[test]
    fn test_add_book_invalid_slot() {
        let mut shelf = ChiseledBookshelf::new();
        assert!(!add_book(&mut shelf, 6, 770));
    }

    #[test]
    fn test_remove_book() {
        let mut shelf = ChiseledBookshelf::new();
        add_book(&mut shelf, 3, 773);
        let removed = remove_book(&mut shelf, 3);
        assert_eq!(removed, Some(773));
        assert_eq!(shelf.slots[3], None);
        assert_eq!(shelf.last_interacted, 3);
    }

    #[test]
    fn test_remove_book_empty_slot() {
        let mut shelf = ChiseledBookshelf::new();
        assert_eq!(remove_book(&mut shelf, 0), None);
    }

    #[test]
    fn test_remove_book_invalid_slot() {
        let mut shelf = ChiseledBookshelf::new();
        assert_eq!(remove_book(&mut shelf, 7), None);
    }

    #[test]
    fn test_comparator_output_occupied() {
        let mut shelf = ChiseledBookshelf::new();
        add_book(&mut shelf, 4, 774);
        assert_eq!(bookshelf_comparator_output(&shelf), 5);
    }

    #[test]
    fn test_comparator_output_empty() {
        let shelf = ChiseledBookshelf::new();
        assert_eq!(bookshelf_comparator_output(&shelf), 0);
    }

    #[test]
    fn test_is_valid_book_item() {
        assert!(is_valid_book_item(770));
        assert!(is_valid_book_item(774));
        assert!(!is_valid_book_item(100));
    }

    #[test]
    fn test_occupied_slot_count() {
        let mut shelf = ChiseledBookshelf::new();
        add_book(&mut shelf, 0, 770);
        add_book(&mut shelf, 2, 771);
        add_book(&mut shelf, 5, 772);
        assert_eq!(occupied_slot_count(&shelf), 3);
    }
}
