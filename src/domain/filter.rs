use crate::domain::filter_config::TagCategories;
use crate::domain::message_highlighter::MessageHighlighter;

use super::filter_config::FilterConfig;

pub const RESET_COLOR: &str = "\x1b[0m";

#[derive(Clone, Debug)]
pub struct LogFilter {
    pub levels: Vec<&'static str>,
    pub tags: TagCategories,
    pub blacklisted_items: Vec<&'static str>,
    message_highlighter: MessageHighlighter,
}

impl LogFilter {
    pub fn new(config: FilterConfig) -> Self {
        Self { 
            levels: config.levels, 
            tags: config.tags,
            blacklisted_items: config.blacklisted_items,
            message_highlighter: MessageHighlighter::new(),
        }
    }

    fn get_level_color(level: &str) -> &'static str {
        match level {
            "E" => "\x1b[31m", // Red for ERROR
            "W" => "\x1b[33m", // Yellow for WARN
            "I" => "\x1b[32m", // Green for INFO
            "D" => "\x1b[36m", // Cyan for DEBUG
            "T" => "\x1b[35m", // Magenta for TRACE
            _ => "\x1b[0m",    // Default color for others
        }
    }

    fn get_tag_color(&self, tag: &str) -> &'static str {
        if self.tags.top_classes.iter().any(|t| t.eq_ignore_ascii_case(tag)) {
            "\x1b[34m" // Blue for top classes
        } else if self.tags.steps.iter().any(|t| t.eq_ignore_ascii_case(tag)) {
            "\x1b[35m" // Magenta for steps
        } else if self.tags.engines.iter().any(|t| t.eq_ignore_ascii_case(tag)) {
            "\x1b[36m" // Cyan for engines
        } else {
            "\x1b[33m" // Yellow for other tags
        }
    }

    pub fn matches(&self, line: &str) -> Option<String> {
        // Skip empty lines
        if line.trim().is_empty() {
            return None;
        }

        let line_lower = line.to_ascii_lowercase();
        if self.blacklisted_items.iter().any(|word| line_lower.contains(word)) {
            return None;
        }

        // Parse the log line
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 5 {
            return None;
        }

        // Detect format and get level/tag indices
        let (level_idx, tag_idx) = if parts[0].contains('-') && parts[0].len() == 10 {
            // YYYY-MM-DD format
            if parts[1].contains('.') {
                // Has milliseconds and a time-zone
                (5, 6) // Format: YYYY-MM-DD HH:MM:SS.mmm +TZ PID TID LEVEL TAG
            } else {
                (4, 5) // Format: YYYY-MM-DD HH:MM:SS PID TID LEVEL TAG
            }
        } else {
            // MM-DD format
            (4, 5) // Format: MM-DD HH:MM:SS PID TID LEVEL TAG
        };

        // Ensure we have enough parts for the detected format
        if parts.len() <= tag_idx {
            return None;
        }

        // Check if any level matches, if levels are set.
        if !self.levels.is_empty() {
            let line_level = parts[level_idx];
            if !self
                .levels
                .iter()
                .any(|level| level.eq_ignore_ascii_case(line_level))
            {
                return None;
            }
        }

        // Check if any tag matches, if tags are set.
        let line_tag = parts[tag_idx].trim_end_matches(':');
        if !self.tags.contains_tag(line_tag) {
            return None;
        }

        // Colorize.
        let mut colored_line = String::new();
        for (i, part) in parts.iter().enumerate() {
            if i == level_idx {
                colored_line.push_str(Self::get_level_color(part));
                colored_line.push_str(part);
                colored_line.push_str(RESET_COLOR);
            } else if i == tag_idx {
                let tag = part.trim_end_matches(':');
                colored_line.push_str(self.get_tag_color(tag));
                colored_line.push_str(part);
                colored_line.push_str(RESET_COLOR);
            } else if i > tag_idx {
                let message = parts[tag_idx + 1..].join(" ");
                colored_line.push_str(&self.message_highlighter.highlight_message(&message));
                break; // Skip remaining parts since we've joined them
            } else {
                colored_line.push_str(part);
            }
            colored_line.push(' ');
        }

        Some(colored_line.trim().to_string())
    }

    pub fn add_highlight_word(&mut self, word: String, color: &'static str) {
        self.message_highlighter.add_highlight_word(word, color);
    }

    pub fn remove_highlight_word(&mut self, word: &str) {
        self.message_highlighter.remove_highlight_word(word);
    }

    pub fn set_verbose(&mut self, verbose: bool) {
        self.message_highlighter.set_verbose(verbose);
    }
}
