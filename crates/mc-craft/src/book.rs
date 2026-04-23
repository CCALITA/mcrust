//! Book and quill writing system.
//!
//! Implements writable books (book and quill) and signed written books
//! with page navigation, text editing, signing, and copy generation tracking.

/// Maximum number of pages in a book.
pub const MAX_PAGES: usize = 100;

/// Maximum characters per page.
pub const MAX_CHARS_PER_PAGE: usize = 256;

/// Maximum length of a book title.
pub const MAX_TITLE_LENGTH: usize = 32;

/// Maximum generation value that can still be copied (original=0, copy=1, copy_of_copy=2).
const MAX_COPYABLE_GENERATION: u8 = 1;

/// A writable book (book and quill) with editable pages.
#[derive(Debug, Clone)]
pub struct BookAndQuill {
    pub pages: Vec<String>,
    pub current_page: usize,
}

impl BookAndQuill {
    /// Creates a new book and quill with a single empty page.
    pub fn new() -> Self {
        Self {
            pages: vec![String::new()],
            current_page: 0,
        }
    }

    /// Adds a new empty page at the end. Returns `true` if the page was added,
    /// `false` if the book is already at the maximum page count.
    pub fn add_page(&mut self) -> bool {
        if self.pages.len() >= MAX_PAGES {
            return false;
        }
        self.pages.push(String::new());
        true
    }

    /// Sets the text of the given page, truncating to [`MAX_CHARS_PER_PAGE`] characters.
    /// Does nothing if the page index is out of range.
    pub fn set_page_text(&mut self, page: usize, text: String) {
        if let Some(slot) = self.pages.get_mut(page) {
            if text.len() <= MAX_CHARS_PER_PAGE {
                *slot = text;
            } else {
                *slot = truncate_to_char_boundary(&text, MAX_CHARS_PER_PAGE);
            }
        }
    }

    /// Advances to the next page if one exists.
    pub fn next_page(&mut self) {
        if self.current_page + 1 < self.pages.len() {
            self.current_page += 1;
        }
    }

    /// Goes back to the previous page if one exists.
    pub fn prev_page(&mut self) {
        if self.current_page > 0 {
            self.current_page -= 1;
        }
    }

    /// Returns the text of the current page.
    pub fn current_text(&self) -> &str {
        &self.pages[self.current_page]
    }
}

impl Default for BookAndQuill {
    fn default() -> Self {
        Self::new()
    }
}

/// A signed, read-only book with title, author, and generation tracking.
#[derive(Debug, Clone)]
pub struct WrittenBook {
    pub title: String,
    pub author: String,
    pub pages: Vec<String>,
    pub generation: u8,
}

/// Signs a [`BookAndQuill`], converting it into a [`WrittenBook`].
///
/// The title is truncated to [`MAX_TITLE_LENGTH`] characters. The resulting
/// book has generation 0 (original).
pub fn sign_book(quill: BookAndQuill, title: String, author: String) -> WrittenBook {
    let truncated_title = if title.len() <= MAX_TITLE_LENGTH {
        title
    } else {
        truncate_to_char_boundary(&title, MAX_TITLE_LENGTH)
    };

    WrittenBook {
        title: truncated_title,
        author,
        pages: quill.pages,
        generation: 0,
    }
}

/// Creates a copy of a [`WrittenBook`], incrementing its generation.
///
/// Returns `None` if the book's generation is 2 or higher (copy of a copy of a copy
/// is not allowed).
pub fn copy_book(book: &WrittenBook) -> Option<WrittenBook> {
    if book.generation > MAX_COPYABLE_GENERATION {
        return None;
    }

    Some(WrittenBook {
        title: book.title.clone(),
        author: book.author.clone(),
        pages: book.pages.clone(),
        generation: book.generation + 1,
    })
}

/// Truncates a string to at most `max_chars` characters, respecting char boundaries.
fn truncate_to_char_boundary(s: &str, max_chars: usize) -> String {
    s.chars().take(max_chars).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_book_has_one_empty_page() {
        let book = BookAndQuill::new();
        assert_eq!(book.pages.len(), 1);
        assert_eq!(book.current_page, 0);
        assert_eq!(book.current_text(), "");
    }

    #[test]
    fn add_page_up_to_max() {
        let mut book = BookAndQuill::new();
        // Already has 1 page, add 99 more
        for _ in 0..99 {
            assert!(book.add_page());
        }
        assert_eq!(book.pages.len(), MAX_PAGES);
        // 101st page should fail
        assert!(!book.add_page());
        assert_eq!(book.pages.len(), MAX_PAGES);
    }

    #[test]
    fn set_page_text_normal() {
        let mut book = BookAndQuill::new();
        book.set_page_text(0, "Hello, world!".to_string());
        assert_eq!(book.current_text(), "Hello, world!");
    }

    #[test]
    fn set_page_text_truncates_to_max_chars() {
        let mut book = BookAndQuill::new();
        let long_text = "a".repeat(300);
        book.set_page_text(0, long_text);
        assert_eq!(book.current_text().len(), MAX_CHARS_PER_PAGE);
    }

    #[test]
    fn set_page_text_out_of_range_does_nothing() {
        let mut book = BookAndQuill::new();
        book.set_page_text(5, "ghost".to_string());
        assert_eq!(book.pages.len(), 1);
        assert_eq!(book.current_text(), "");
    }

    #[test]
    fn next_and_prev_page_navigation() {
        let mut book = BookAndQuill::new();
        book.add_page();
        book.add_page();
        book.set_page_text(0, "Page 1".to_string());
        book.set_page_text(1, "Page 2".to_string());
        book.set_page_text(2, "Page 3".to_string());

        assert_eq!(book.current_text(), "Page 1");

        book.next_page();
        assert_eq!(book.current_text(), "Page 2");

        book.next_page();
        assert_eq!(book.current_text(), "Page 3");

        // Already at last page, should stay
        book.next_page();
        assert_eq!(book.current_text(), "Page 3");

        book.prev_page();
        assert_eq!(book.current_text(), "Page 2");

        book.prev_page();
        assert_eq!(book.current_text(), "Page 1");

        // Already at first page, should stay
        book.prev_page();
        assert_eq!(book.current_text(), "Page 1");
    }

    #[test]
    fn sign_book_creates_original() {
        let mut quill = BookAndQuill::new();
        quill.set_page_text(0, "Once upon a time...".to_string());
        quill.add_page();
        quill.set_page_text(1, "The end.".to_string());

        let written = sign_book(quill, "My Story".to_string(), "Author".to_string());
        assert_eq!(written.title, "My Story");
        assert_eq!(written.author, "Author");
        assert_eq!(written.generation, 0);
        assert_eq!(written.pages.len(), 2);
        assert_eq!(written.pages[0], "Once upon a time...");
        assert_eq!(written.pages[1], "The end.");
    }

    #[test]
    fn sign_book_truncates_long_title() {
        let quill = BookAndQuill::new();
        let long_title = "a".repeat(50);
        let written = sign_book(quill, long_title, "Author".to_string());
        assert_eq!(written.title.len(), MAX_TITLE_LENGTH);
    }

    #[test]
    fn copy_book_increments_generation() {
        let original = WrittenBook {
            title: "Test".to_string(),
            author: "Author".to_string(),
            pages: vec!["Page 1".to_string()],
            generation: 0,
        };

        // Original (gen 0) -> Copy (gen 1)
        let copy = copy_book(&original).expect("should copy original");
        assert_eq!(copy.generation, 1);
        assert_eq!(copy.title, "Test");
        assert_eq!(copy.author, "Author");
        assert_eq!(copy.pages, vec!["Page 1".to_string()]);

        // Copy (gen 1) -> Copy of copy (gen 2)
        let copy_of_copy = copy_book(&copy).expect("should copy a copy");
        assert_eq!(copy_of_copy.generation, 2);

        // Copy of copy (gen 2) -> None (cannot copy further)
        let too_many = copy_book(&copy_of_copy);
        assert!(too_many.is_none());
    }

    #[test]
    fn truncate_respects_multibyte_chars() {
        let mut book = BookAndQuill::new();
        // Each emoji is multiple bytes but 1 char
        let emoji_text = "😀".repeat(300);
        book.set_page_text(0, emoji_text);
        assert_eq!(book.current_text().chars().count(), MAX_CHARS_PER_PAGE);
    }

    #[test]
    fn default_trait_matches_new() {
        let default_book = BookAndQuill::default();
        let new_book = BookAndQuill::new();
        assert_eq!(default_book.pages, new_book.pages);
        assert_eq!(default_book.current_page, new_book.current_page);
    }
}
