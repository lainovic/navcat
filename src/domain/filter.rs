use crate::domain::filter_config::TagCategories;
use crate::domain::message_highlighter::MessageHighlighter;
use crate::domain::filter_config::create_default_highlighter;

use super::filter_config::FilterConfig;

pub const RESET_COLOR: &str = "\x1b[0m";

#[derive(Debug)]
enum LogFormat {
    FullWithMillis, // YYYY-MM-DD HH:MM:SS.mmm +TZ PID TID LEVEL TAG
    Full,           // YYYY-MM-DD HH:MM:SS PID TID LEVEL TAG
    Short,          // MM-DD HH:MM:SS PID TID LEVEL TAG
}

#[derive(Clone, Debug)]
pub struct LogFilter {
    pub levels: Vec<&'static str>,
    pub tags: TagCategories,
    pub blacklisted_items: Vec<String>,
    pub show_items: Vec<String>,
    message_highlighter: MessageHighlighter,
}

impl LogFilter {
    pub fn new(config: FilterConfig) -> Self {
        let mut builder = create_default_highlighter();
        
        // Add any custom highlighted items from the config
        if !config.highlighted_items.is_empty() {
            builder = builder.add_custom_words(&config.highlighted_items.iter().map(|s| s.as_str()).collect::<Vec<_>>());
        }

        Self {
            levels: config.levels,
            tags: config.tags,
            blacklisted_items: config.blacklisted_items,
            show_items: config.show_items,
            message_highlighter: builder.build(),
        }
    }

    fn get_level_color(level: &str) -> &'static str {
        match level {
            "E" => "\x1b[31m", // Red for ERROR
            "W" => "\x1b[33m", // Yellow for WARN
            "I" => "\x1b[32m", // Green for INFO
            "D" => "\x1b[36m", // Cyan for DEBUG
            "T" => "\x1b[35m", // Magenta for TRACE
            _ => RESET_COLOR,    // Default color for others
        }
    }

    fn get_tag_color(&self, tag: &str) -> &'static str {
        if self.tags.top_classes.iter().any(|t| tag.contains(t)) {
            "\x1b[34m" // Blue for top classes
        } else if self.tags.steps.iter().any(|t| tag.contains(t)) {
            "\x1b[35m" // Magenta for steps
        } else if self.tags.engines.iter().any(|t| tag.contains(t)) {
            "\x1b[36m" // Cyan for engines
        } else {
            "\x1b[1;31m" // Red for other tags
        }
    }

    pub fn matches(&self, line: &str) -> Option<String> {
        // Skip empty lines
        if line.trim().is_empty() {
            return None;
        }

        let line_lower = line.to_ascii_lowercase();

        if !self.show_items.is_empty()
            && !self.show_items.iter().any(|word| line_lower.contains(word))
        {
            return None;
        }

        if self
            .blacklisted_items
            .iter()
            .any(|word| line_lower.contains(word))
        {
            return None;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        let (level_idx, tag_idx) = Self::get_level_and_tag_indices(&parts);

        // Ensure we have enough parts for the detected format.
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

    fn get_level_and_tag_indices(parts: &Vec<&str>) -> (usize, usize) {
        return match Self::detect_format(&parts) {
            LogFormat::FullWithMillis => (5, 6),
            LogFormat::Full => (4, 5),
            LogFormat::Short => (4, 5),
        };
    }

    fn detect_format(parts: &Vec<&str>) -> LogFormat {
        if parts[0].contains('-') && parts[0].len() == 10 {
            // YYYY-MM-DD
            if parts[1].contains('.') {
                // HH:MM:SS.mmm +TZ
                LogFormat::FullWithMillis
            } else {
                // HH:MM:SS
                LogFormat::Full
            }
        } else {
            // MM-DD
            LogFormat::Short
        }
    }

    pub fn add_highlight_word(&mut self, word: String, color: &'static str) {
        self.message_highlighter.add_highlight_word(word, color);
    }

    pub fn remove_highlight_word(&mut self, word: &str) {
        self.message_highlighter.remove_highlight_word(word);
    }
}
