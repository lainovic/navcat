use std::collections::HashSet;

use crate::domain::message_highlighter::MessageHighlighter;

pub const RED_COLOR: &str = "\x1b[1;31m"; // Bold Red
pub const GREEN_COLOR: &str = "\x1b[1;32m"; // Bold Green
pub const YELLOW_COLOR: &str = "\x1b[33m"; // Yellow
pub const BG_YELLOW: &str = "\x1b[43m"; // Background Yellow

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

pub fn create_default_highlighter() -> HighlightBuilder {
    HighlightBuilder::new()
        // Red highlights for warnings/errors/deviations
        .add_red_words(&[
            "error",
            "removed",
            "unfollowed",
            "not followed",
            "unvisited",
            "deviation",
            "off-road",
            "off-route",
        ])
        // Green highlights for positive messages/information
        .add_green_words(&[
            "success",
            "created",
            "added",
            "followed",
            "following",
            "visited",
            "planned",
            "arrived",
            "departed",
            "started",
            "starting",
            "resumed",
            "resuming",
            "stopped",
            "stopping",
        ])
        // Yellow highlights for navigation and map matching events
        .add_yellow_words(&[
            "warning",
            "updated",
            "changed",
            "segment",
            "old",
            "new",
            "map matching",
            "projected",
            "matchlocation",
            "matched",
            "replan",
            "should replan",
            "refresh",
            "back to route",
            "replanning",
            "language change",
            "increment",
            "progress",
            "current location",
            "distancealongroute",
            "traffic jam",
            "instruction",
            "guidance",
            "lane guidance",
            "lane level guidance",
            "route",
            "waypoint",
            "planning route",
        ])
}
