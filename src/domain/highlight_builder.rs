use std::collections::HashSet;

use crate::domain::message_highlighter::MessageHighlighter;

pub const RED_COLOR: &str = "\x1b[1;31m";    // Bold Red
pub const GREEN_COLOR: &str = "\x1b[1;32m";  // Bold Green
pub const YELLOW_COLOR: &str = "\x1b[33m";   // Yellow
pub const BG_YELLOW: &str = "\x1b[43m";      // Background Yellow

pub struct HighlightBuilder {
    red_words: HashSet<String>,
    green_words: HashSet<String>,
    yellow_words: HashSet<String>,
    custom_words: HashSet<String>,
}

impl HighlightBuilder {
    pub fn new() -> Self {
        Self {
            red_words: HashSet::new(),
            green_words: HashSet::new(),
            yellow_words: HashSet::new(),
            custom_words: HashSet::new(),
        }
    }

    pub fn add_red_word(mut self, word: &str) -> Self {
        self.red_words.insert(word.to_lowercase());
        self
    }

    pub fn add_green_word(mut self, word: &str) -> Self {
        self.green_words.insert(word.to_lowercase());
        self
    }

    pub fn add_yellow_word(mut self, word: &str) -> Self {
        self.yellow_words.insert(word.to_lowercase());
        self
    }

    pub fn add_custom_word(mut self, word: &str) -> Self {
        self.custom_words.insert(word.to_lowercase());
        self
    }

    pub fn add_red_words(mut self, words: &[&str]) -> Self {
        for word in words {
            self.red_words.insert(word.to_lowercase());
        }
        self
    }

    pub fn add_green_words(mut self, words: &[&str]) -> Self {
        for word in words {
            self.green_words.insert(word.to_lowercase());
        }
        self
    }

    pub fn add_yellow_words(mut self, words: &[&str]) -> Self {
        for word in words {
            self.yellow_words.insert(word.to_lowercase());
        }
        self
    }

    pub fn add_custom_words(mut self, words: &[&str]) -> Self {
        for word in words {
            self.custom_words.insert(word.to_lowercase());
        }
        self
    }

    pub fn build(self) -> MessageHighlighter {
        MessageHighlighter::new(
            self.red_words,
            self.green_words,
            self.yellow_words,
            self.custom_words,
        )
    }
}
