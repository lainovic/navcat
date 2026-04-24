use crate::domain::FilterConfig;
use crate::domain::filter_config::TagCategories;
use crate::domain::highlight_builder::create_default_highlighter;
use crate::domain::message_highlighter::MessageHighlighter;

pub const RESET_COLOR: &str = "\x1b[0m";

#[derive(Debug)]
enum LogFormat {
    FullWithPidTid, // YYYY-MM-DD HH:MM:SS.mmm +TZ PID TID LEVEL TAG
    Full,           // YYYY-MM-DD HH:MM:SS PID TID LEVEL TAG
    Compact,        // YYYY-MM-DD HH:MM:SS.mmm+TZ LEVEL TAG: MESSAGE
    Short,          // MM-DD HH:MM:SS PID TID LEVEL TAG
}

#[derive(Clone, Debug)]
pub struct LogFilter {
    pub levels: Vec<&'static str>,
    pub tags: TagCategories,
    pub blacklisted_items: Vec<String>,
    pub show_items: Vec<String>,
    pub no_tag_filter: bool,
    message_highlighter: MessageHighlighter,
}

impl LogFilter {
    pub fn new(config: FilterConfig) -> Self {
        let mut builder = create_default_highlighter();

        // Add any custom highlighted items from the config
        if !config.highlighted_items.is_empty() {
            builder = builder.add_custom_words(
                &config
                    .highlighted_items
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>(),
            );
        }

        Self {
            levels: config.levels,
            tags: config.tags,
            blacklisted_items: config.blacklisted_items,
            show_items: config.show_items,
            no_tag_filter: config.no_tag_filter,
            message_highlighter: builder.build(),
        }
    }

    fn get_level_color(level: &str) -> &'static str {
        match level {
            "V" => "\x1b[37m",   // White for VERBOSE
            "D" => "\x1b[36m",   // Cyan for DEBUG
            "I" => "\x1b[32m",   // Green for INFO
            "W" => "\x1b[33m",   // Yellow for WARN
            "E" => "\x1b[31m",   // Red for ERROR
            "F" => "\x1b[1;31m", // Bold red for FATAL
            _ => RESET_COLOR,
        }
    }

    fn get_tag_color(&self, tag: &str) -> &'static str {
        if self.tags.routing_tags.iter().any(|t| tag.contains(t)) {
            "\x1b[1;31m" // Bold red for routing (planners, replan)
        } else if self.tags.mapmatching_tags.iter().any(|t| tag.contains(t)) {
            "\x1b[33m" // Yellow for map-matching
        } else if self.tags.guidance_tags.iter().any(|t| tag.contains(t)) {
            "\x1b[35m" // Magenta for guidance
        } else {
            "\x1b[34m" // Blue for everything else
        }
    }

    pub fn matches(&self, line: &str) -> Option<String> {
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

        let (level_idx, tag_idx) = Self::get_level_and_tag_indices(&parts)?;

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

        // Check tag filter. When no_tag_filter is set, empty tag list means "show all".
        // Otherwise empty tag list means all category toggles are off → show nothing.
        let line_tag = parts[tag_idx].trim_end_matches(':');
        if !self.no_tag_filter {
            if self.tags.all_tags.is_empty() || !self.tags.contains_tag(line_tag) {
                return None;
            }
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

    fn get_level_and_tag_indices(parts: &[&str]) -> Option<(usize, usize)> {
        match Self::detect_format(parts)? {
            LogFormat::FullWithPidTid => Some((5, 6)),
            LogFormat::Full => Some((4, 5)),
            LogFormat::Short => Some((4, 5)),
            LogFormat::Compact => Some((2, 3)),
        }
    }

    fn detect_format(parts: &[&str]) -> Option<LogFormat> {
        if parts.len() < 3 {
            return None;
        }

        if parts[0].contains('-') && parts[0].len() == 10 {
            // YYYY-MM-DD format
            if parts[1].contains('.') {
                // Has milliseconds: YYYY-MM-DD HH:MM:SS.mmm +TZ PID TID LEVEL TAG
                if parts.len() >= 7
                    && Self::looks_like_pid(parts.get(3).copied())
                    && Self::looks_like_tid(parts.get(4).copied())
                {
                    return Some(LogFormat::FullWithPidTid);
                }
                Some(LogFormat::Compact)
            } else {
                // No milliseconds: YYYY-MM-DD HH:MM:SS PID TID LEVEL TAG
                if parts.len() >= 6
                    && Self::looks_like_pid(parts.get(2).copied())
                    && Self::looks_like_tid(parts.get(3).copied())
                {
                    return Some(LogFormat::Full);
                }
                Some(LogFormat::Compact)
            }
        } else if parts[0].len() == 5 && parts[0].as_bytes()[2] == b'-' {
            // MM-DD format
            Some(LogFormat::Short)
        } else {
            None
        }
    }

    fn looks_like_pid(part: Option<&str>) -> bool {
        part.map(|p| p.chars().all(|c| c.is_ascii_digit())).unwrap_or(false)
    }

    fn looks_like_tid(part: Option<&str>) -> bool {
        part.map(|p| p.chars().all(|c| c.is_ascii_digit())).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::filter_config::{FilterConfig, TagCategories};

    fn make_filter(
        levels: Vec<&'static str>,
        tags: Vec<&'static str>,
        blacklist: Vec<&'static str>,
        show: Vec<&'static str>,
    ) -> LogFilter {
        let tag_strings: Vec<String> = tags.into_iter().map(String::from).collect();
        let no_tag_filter = tag_strings.is_empty();
        LogFilter::new(FilterConfig {
            levels,
            tags: TagCategories::new(tag_strings),
            blacklisted_items: blacklist.into_iter().map(String::from).collect(),
            highlighted_items: vec![],
            show_items: show.into_iter().map(String::from).collect(),
            no_tag_filter,
        })
    }

    // --- detect_format ---

    #[test]
    fn detect_full_with_pid_tid() {
        // YYYY-MM-DD HH:MM:SS.mmm +TZ PID TID LEVEL TAG msg
        let line = "2024-01-15 10:30:45.123 +0000 1234 5678 I SomeTag: message";
        let parts: Vec<&str> = line.split_whitespace().collect();
        assert!(matches!(LogFilter::detect_format(&parts), Some(LogFormat::FullWithPidTid)));
    }

    #[test]
    fn detect_full() {
        // YYYY-MM-DD HH:MM:SS PID TID LEVEL TAG msg
        let line = "2024-01-15 10:30:45 1234 5678 I SomeTag: message";
        let parts: Vec<&str> = line.split_whitespace().collect();
        assert!(matches!(LogFilter::detect_format(&parts), Some(LogFormat::Full)));
    }

    #[test]
    fn detect_short() {
        // MM-DD HH:MM:SS.mmm PID TID LEVEL TAG msg
        let line = "01-15 10:30:45.123 1234 5678 I SomeTag: message";
        let parts: Vec<&str> = line.split_whitespace().collect();
        assert!(matches!(LogFilter::detect_format(&parts), Some(LogFormat::Short)));
    }

    #[test]
    fn detect_compact() {
        // YYYY-MM-DD HH:MM:SS.mmm+TZ LEVEL TAG: msg
        let line = "2024-01-15 10:30:45.123+0000 I SomeTag: message";
        let parts: Vec<&str> = line.split_whitespace().collect();
        assert!(matches!(LogFilter::detect_format(&parts), Some(LogFormat::Compact)));
    }

    #[test]
    fn detect_too_few_parts_returns_none() {
        let parts = vec!["2024-01-15"];
        assert!(LogFilter::detect_format(&parts).is_none());
    }

    #[test]
    fn detect_stacktrace_returns_none() {
        let line = "at com.example.Foo.bar(Foo.kt:42)";
        let parts: Vec<&str> = line.split_whitespace().collect();
        assert!(LogFilter::detect_format(&parts).is_none());
    }

    #[test]
    fn detect_logcat_header_returns_none() {
        let line = "--------- beginning of main";
        let parts: Vec<&str> = line.split_whitespace().collect();
        assert!(LogFilter::detect_format(&parts).is_none());
    }

    // --- matches: level filtering ---

    #[test]
    fn matches_passes_correct_level() {
        let filter = make_filter(vec!["I"], vec![], vec![], vec![]);
        let line = "2024-01-15 10:30:45 1234 5678 I SomeTag: hello";
        assert!(filter.matches(line).is_some());
    }

    #[test]
    fn matches_rejects_wrong_level() {
        let filter = make_filter(vec!["E"], vec![], vec![], vec![]);
        let line = "2024-01-15 10:30:45 1234 5678 I SomeTag: hello";
        assert!(filter.matches(line).is_none());
    }

    // --- matches: tag filtering ---

    #[test]
    fn matches_passes_matching_tag() {
        let filter = make_filter(vec![], vec!["Navigation"], vec![], vec![]);
        let line = "2024-01-15 10:30:45 1234 5678 I DefaultNavigation: hello";
        assert!(filter.matches(line).is_some());
    }

    #[test]
    fn matches_rejects_non_matching_tag() {
        let filter = make_filter(vec![], vec!["Navigation"], vec![], vec![]);
        let line = "2024-01-15 10:30:45 1234 5678 I SomeOtherTag: hello";
        assert!(filter.matches(line).is_none());
    }

    #[test]
    fn matches_passes_all_tags_when_tag_list_empty() {
        let filter = make_filter(vec!["I"], vec![], vec![], vec![]);
        let line = "2024-01-15 10:30:45 1234 5678 I AnythingAtAll: hello";
        assert!(filter.matches(line).is_some());
    }

    // --- matches: blacklist ---

    #[test]
    fn matches_rejects_blacklisted_word() {
        let filter = make_filter(vec![], vec![], vec!["guidance"], vec![]);
        let line = "2024-01-15 10:30:45 1234 5678 I SomeTag: guidance update";
        assert!(filter.matches(line).is_none());
    }

    #[test]
    fn matches_blacklist_is_case_insensitive() {
        let filter = make_filter(vec![], vec![], vec!["guidance"], vec![]);
        let line = "2024-01-15 10:30:45 1234 5678 I SomeTag: GUIDANCE update";
        assert!(filter.matches(line).is_none());
    }

    // --- matches: show-items ---

    #[test]
    fn matches_passes_line_containing_show_item() {
        let filter = make_filter(vec![], vec![], vec![], vec!["replan"]);
        let line = "2024-01-15 10:30:45 1234 5678 I SomeTag: replan triggered";
        assert!(filter.matches(line).is_some());
    }

    #[test]
    fn matches_rejects_line_missing_show_item() {
        let filter = make_filter(vec![], vec![], vec![], vec!["replan"]);
        let line = "2024-01-15 10:30:45 1234 5678 I SomeTag: normal progress update";
        assert!(filter.matches(line).is_none());
    }

    // --- matches: misc ---

    #[test]
    fn matches_empty_line_returns_none() {
        let filter = make_filter(vec![], vec![], vec![], vec![]);
        assert!(filter.matches("").is_none());
        assert!(filter.matches("   ").is_none());
    }

    #[test]
    fn matches_unrecognized_format_returns_none() {
        let filter = make_filter(vec![], vec![], vec![], vec![]);
        assert!(filter.matches("at com.example.Foo.bar(Foo.kt:42)").is_none());
        assert!(filter.matches("--------- beginning of main").is_none());
    }
}
