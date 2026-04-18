/// Maximum number of pages a book can contain.
const MAX_PAGES: usize = 100;
/// Maximum number of characters per page.
const MAX_PAGE_CHARS: usize = 256;

/// Represents a book that can be placed on a lectern.
#[derive(Debug, Clone, PartialEq)]
pub struct BookData {
    pub title: String,
    pub author: String,
    pub pages: Vec<String>,
    pub signed: bool,
}

impl BookData {
    /// Creates a new empty book with the given title and author.
    pub fn new(title: impl Into<String>, author: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            author: author.into(),
            pages: Vec::new(),
            signed: false,
        }
    }

    /// Adds a page to the book. Returns `true` if the page was added successfully,
    /// or `false` if the book already has the maximum number of pages or the text
    /// exceeds the character limit.
    pub fn add_page(&mut self, text: &str) -> bool {
        if self.pages.len() >= MAX_PAGES {
            return false;
        }
        if text.len() > MAX_PAGE_CHARS {
            return false;
        }
        self.pages.push(text.to_string());
        true
    }

    /// Returns the text of the page at the given index, or `None` if out of bounds.
    pub fn get_page(&self, idx: usize) -> Option<&str> {
        self.pages.get(idx).map(|s| s.as_str())
    }

    /// Returns the number of pages in the book.
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }
}

/// Represents the state of a lectern block in the world.
#[derive(Debug, Clone, Default)]
pub struct LecternState {
    pub book: Option<BookData>,
    pub page: u8,
}

impl LecternState {
    /// Places a book on the lectern, resetting the current page to 0.
    pub fn place_book(&mut self, book: BookData) {
        self.book = Some(book);
        self.page = 0;
    }

    /// Takes the book from the lectern, returning it if one was present.
    pub fn take_book(&mut self) -> Option<BookData> {
        self.page = 0;
        self.book.take()
    }

    /// Turns the page forward or backward. Clamps to valid page range.
    pub fn turn_page(&mut self, forward: bool) {
        if let Some(ref book) = self.book {
            let total = book.page_count();
            if total == 0 {
                return;
            }
            if forward {
                if (self.page as usize) < total.saturating_sub(1) {
                    self.page += 1;
                }
            } else if self.page > 0 {
                self.page -= 1;
            }
        }
    }

    /// Returns the current page index.
    pub fn current_page(&self) -> u8 {
        self.page
    }
}

/// Calculates the redstone signal strength for a lectern.
///
/// Returns 0 if there is no book. Otherwise returns `1 + page * 14 / max(1, total - 1)`.
pub fn lectern_redstone(has_book: bool, page: u8, total: u8) -> u8 {
    if !has_book {
        return 0;
    }
    let denom = (total as u16).saturating_sub(1).max(1);
    let signal = 1 + (page as u16) * 14 / denom;
    signal as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- BookData tests ---

    #[test]
    fn book_new_creates_empty_book() {
        let book = BookData::new("Title", "Author");
        assert_eq!(book.title, "Title");
        assert_eq!(book.author, "Author");
        assert_eq!(book.page_count(), 0);
        assert!(!book.signed);
    }

    #[test]
    fn book_add_page_within_limits() {
        let mut book = BookData::new("T", "A");
        assert!(book.add_page("Hello world"));
        assert_eq!(book.page_count(), 1);
        assert_eq!(book.get_page(0), Some("Hello world"));
    }

    #[test]
    fn book_get_page_out_of_bounds() {
        let book = BookData::new("T", "A");
        assert_eq!(book.get_page(0), None);
        assert_eq!(book.get_page(100), None);
    }

    #[test]
    fn book_rejects_page_over_char_limit() {
        let mut book = BookData::new("T", "A");
        let long_text = "x".repeat(MAX_PAGE_CHARS + 1);
        assert!(!book.add_page(&long_text));
        assert_eq!(book.page_count(), 0);
    }

    #[test]
    fn book_accepts_page_at_char_limit() {
        let mut book = BookData::new("T", "A");
        let exact_text = "x".repeat(MAX_PAGE_CHARS);
        assert!(book.add_page(&exact_text));
        assert_eq!(book.page_count(), 1);
    }

    #[test]
    fn book_rejects_page_over_max_pages() {
        let mut book = BookData::new("T", "A");
        for i in 0..MAX_PAGES {
            assert!(book.add_page(&format!("Page {i}")));
        }
        assert_eq!(book.page_count(), MAX_PAGES);
        assert!(!book.add_page("One more"));
        assert_eq!(book.page_count(), MAX_PAGES);
    }

    // --- LecternState page navigation tests ---

    #[test]
    fn lectern_page_navigation_forward() {
        let mut lectern = LecternState::default();
        let mut book = BookData::new("T", "A");
        book.add_page("p0");
        book.add_page("p1");
        book.add_page("p2");
        lectern.place_book(book);

        assert_eq!(lectern.current_page(), 0);
        lectern.turn_page(true);
        assert_eq!(lectern.current_page(), 1);
        lectern.turn_page(true);
        assert_eq!(lectern.current_page(), 2);
        // Should not go past the last page
        lectern.turn_page(true);
        assert_eq!(lectern.current_page(), 2);
    }

    #[test]
    fn lectern_page_navigation_backward() {
        let mut lectern = LecternState::default();
        let mut book = BookData::new("T", "A");
        book.add_page("p0");
        book.add_page("p1");
        lectern.place_book(book);

        lectern.turn_page(true); // go to page 1
        assert_eq!(lectern.current_page(), 1);
        lectern.turn_page(false);
        assert_eq!(lectern.current_page(), 0);
        // Should not go below 0
        lectern.turn_page(false);
        assert_eq!(lectern.current_page(), 0);
    }

    #[test]
    fn lectern_turn_page_without_book_is_noop() {
        let mut lectern = LecternState::default();
        lectern.turn_page(true);
        assert_eq!(lectern.current_page(), 0);
    }

    // --- Place / Take tests ---

    #[test]
    fn lectern_place_and_take_book() {
        let mut lectern = LecternState::default();
        let book = BookData::new("My Book", "Me");
        lectern.place_book(book.clone());

        assert!(lectern.book.is_some());
        let taken = lectern.take_book();
        assert_eq!(taken, Some(book));
        assert!(lectern.book.is_none());
        assert_eq!(lectern.current_page(), 0);
    }

    #[test]
    fn lectern_take_book_when_empty() {
        let mut lectern = LecternState::default();
        assert_eq!(lectern.take_book(), None);
    }

    #[test]
    fn lectern_place_book_resets_page() {
        let mut lectern = LecternState::default();
        let mut book = BookData::new("T", "A");
        book.add_page("p0");
        book.add_page("p1");
        lectern.place_book(book);
        lectern.turn_page(true);
        assert_eq!(lectern.current_page(), 1);

        // Place a new book — page should reset
        let book2 = BookData::new("T2", "A2");
        lectern.place_book(book2);
        assert_eq!(lectern.current_page(), 0);
    }

    // --- Redstone signal tests ---

    #[test]
    fn redstone_no_book_returns_zero() {
        assert_eq!(lectern_redstone(false, 0, 0), 0);
        assert_eq!(lectern_redstone(false, 5, 10), 0);
    }

    #[test]
    fn redstone_single_page_book() {
        // total=1 => denom=max(1,0)=1 => 1 + 0*14/1 = 1
        assert_eq!(lectern_redstone(true, 0, 1), 1);
    }

    #[test]
    fn redstone_first_page() {
        // page=0, total=15 => 1 + 0*14/14 = 1
        assert_eq!(lectern_redstone(true, 0, 15), 1);
    }

    #[test]
    fn redstone_last_page() {
        // page=14, total=15 => 1 + 14*14/14 = 1 + 14 = 15
        assert_eq!(lectern_redstone(true, 14, 15), 15);
    }

    #[test]
    fn redstone_middle_page() {
        // page=7, total=15 => 1 + 7*14/14 = 1 + 7 = 8
        assert_eq!(lectern_redstone(true, 7, 15), 8);
    }

    #[test]
    fn redstone_two_page_book() {
        // page=0 => 1 + 0*14/1 = 1
        assert_eq!(lectern_redstone(true, 0, 2), 1);
        // page=1 => 1 + 1*14/1 = 15
        assert_eq!(lectern_redstone(true, 1, 2), 15);
    }
}
