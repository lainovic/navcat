use std::collections::HashSet;

use ratatui::style::{Color, Modifier, Style};

use crate::domain::message_highlighter::MessageHighlighter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HighlightPriority {
    Custom = 0,
    Green = 1,
    Yellow = 2,
    Red = 3,
}

impl HighlightPriority {
    pub fn style(self) -> Style {
        match self {
            Self::Red => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            Self::Green => Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
            Self::Yellow => Style::default().fg(Color::Yellow),
            Self::Custom => Style::default().bg(Color::Yellow),
        }
    }
}

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
