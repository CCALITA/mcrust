//! Lectern block: holds a written book and broadcasts the current page via redstone.
//!
//! A lectern can hold a single book of pages. Players can navigate pages, and the
//! current page is exposed as a redstone signal. Empty lecterns emit no signal.
//! Villager AI uses [`librarian_attraction_block_id`] to detect lecterns.

/// Placeholder block id for lecterns. Used by villager AI for librarian POI detection.
pub const LECTERN_BLOCK_ID: u16 = 200;

/// Returns the block id villagers use to detect lecterns when seeking a librarian
/// workstation.
pub fn librarian_attraction_block_id() -> u16 {
    LECTERN_BLOCK_ID
}

/// A lectern block state. Holds an optional written book with paged contents.
#[derive(Debug, Clone)]
pub struct Lectern {
    pub has_book: bool,
    pub pages: Vec<String>,
    pub current_page: usize,
}

impl Default for Lectern {
    fn default() -> Self {
        Self::new()
    }
}

impl Lectern {
    /// Construct an empty lectern with no book.
    pub fn new() -> Self {
        Self {
            has_book: false,
            pages: Vec::new(),
            current_page: 0,
        }
    }

    /// Place a book on the lectern. Returns false if the lectern already holds a
    /// book or if `pages` is empty.
    pub fn place_book(&mut self, pages: Vec<String>) -> bool {
        if self.has_book || pages.is_empty() {
            return false;
        }
        self.pages = pages;
        self.current_page = 0;
        self.has_book = true;
        true
    }

    /// Take the book off the lectern, returning its pages and resetting state.
    pub fn take_book(&mut self) -> Option<Vec<String>> {
        if !self.has_book {
            return None;
        }
        let pages = std::mem::take(&mut self.pages);
        self.current_page = 0;
        self.has_book = false;
        Some(pages)
    }

    /// Advance to the next page, clamped at the last page.
    pub fn next_page(&mut self) {
        if !self.has_book || self.pages.is_empty() {
            return;
        }
        let last = self.pages.len() - 1;
        if self.current_page < last {
            self.current_page += 1;
        } else {
            self.current_page = last;
        }
    }

    /// Move to the previous page, clamped at 0.
    pub fn prev_page(&mut self) {
        if self.current_page > 0 {
            self.current_page -= 1;
        }
    }

    /// Compute the redstone signal strength for a comparator reading this lectern.
    /// Returns 0 when no book is present, otherwise scales the current page across
    /// the range 1..=15.
    pub fn redstone_signal(&self) -> u8 {
        if !self.has_book || self.pages.is_empty() {
            return 0;
        }
        let total = self.pages.len();
        let scaled = ((self.current_page + 1) * 15) / total;
        scaled.clamp(1, 15) as u8
    }

    /// Return the text of the page currently being viewed, or an empty slice when
    /// the lectern is empty.
    pub fn page_text(&self) -> &str {
        if !self.has_book {
            return "";
        }
        self.pages
            .get(self.current_page)
            .map(String::as_str)
            .unwrap_or("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_pages() -> Vec<String> {
        vec!["one".into(), "two".into(), "three".into()]
    }

    #[test]
    fn new_lectern_is_empty() {
        let l = Lectern::new();
        assert!(!l.has_book);
        assert_eq!(l.current_page, 0);
        assert!(l.pages.is_empty());
        assert_eq!(l.redstone_signal(), 0);
        assert_eq!(l.page_text(), "");
    }

    #[test]
    fn place_then_take_cycle() {
        let mut l = Lectern::new();
        assert!(l.place_book(sample_pages()));
        assert!(l.has_book);
        assert_eq!(l.page_text(), "one");

        let taken = l.take_book().expect("book should come back");
        assert_eq!(taken, sample_pages());
        assert!(!l.has_book);
        assert!(l.pages.is_empty());
        assert_eq!(l.current_page, 0);
    }

    #[test]
    fn place_book_rejects_when_occupied() {
        let mut l = Lectern::new();
        assert!(l.place_book(sample_pages()));
        assert!(!l.place_book(vec!["other".into()]));
        // Original book preserved
        assert_eq!(l.pages, sample_pages());
    }

    #[test]
    fn place_book_rejects_empty_pages() {
        let mut l = Lectern::new();
        assert!(!l.place_book(Vec::new()));
        assert!(!l.has_book);
    }

    #[test]
    fn take_book_on_empty_returns_none() {
        let mut l = Lectern::new();
        assert!(l.take_book().is_none());
    }

    #[test]
    fn next_page_clamps_to_last() {
        let mut l = Lectern::new();
        l.place_book(sample_pages());
        l.next_page();
        assert_eq!(l.current_page, 1);
        l.next_page();
        assert_eq!(l.current_page, 2);
        l.next_page(); // already at last
        assert_eq!(l.current_page, 2);
        assert_eq!(l.page_text(), "three");
    }

    #[test]
    fn prev_page_clamps_to_zero() {
        let mut l = Lectern::new();
        l.place_book(sample_pages());
        l.prev_page();
        assert_eq!(l.current_page, 0);
        l.next_page();
        l.prev_page();
        assert_eq!(l.current_page, 0);
    }

    #[test]
    fn next_page_noop_when_empty() {
        let mut l = Lectern::new();
        l.next_page();
        assert_eq!(l.current_page, 0);
    }

    #[test]
    fn redstone_signal_scales_across_pages() {
        let mut l = Lectern::new();
        l.place_book(sample_pages()); // 3 pages
        // page 0 -> ((0+1)*15)/3 = 5
        assert_eq!(l.redstone_signal(), 5);
        l.next_page();
        // page 1 -> ((1+1)*15)/3 = 10
        assert_eq!(l.redstone_signal(), 10);
        l.next_page();
        // page 2 -> ((2+1)*15)/3 = 15
        assert_eq!(l.redstone_signal(), 15);
    }

    #[test]
    fn redstone_signal_minimum_is_one_with_book() {
        let mut l = Lectern::new();
        // 30 pages: page 0 -> ((0+1)*15)/30 = 0, clamped up to 1
        let pages: Vec<String> = (0..30).map(|i| format!("p{i}")).collect();
        l.place_book(pages);
        assert_eq!(l.redstone_signal(), 1);
    }

    #[test]
    fn redstone_signal_zero_when_empty() {
        let l = Lectern::new();
        assert_eq!(l.redstone_signal(), 0);
    }

    #[test]
    fn page_text_returns_current_page() {
        let mut l = Lectern::new();
        l.place_book(sample_pages());
        assert_eq!(l.page_text(), "one");
        l.next_page();
        assert_eq!(l.page_text(), "two");
    }

    #[test]
    fn librarian_attraction_id_is_lectern_block_id() {
        assert_eq!(librarian_attraction_block_id(), LECTERN_BLOCK_ID);
        assert_eq!(LECTERN_BLOCK_ID, 200);
    }
}
