use crate::domain::FilterConfig;
use crate::domain::filter_config::{TagCategories, TagCategory};
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
    levels: Vec<&'static str>,
    tags: TagCategories,
    blacklisted_items: Vec<String>,
    show_items: Vec<String>,
    no_tag_filter: bool,
    message_highlighter: MessageHighlighter,
}

impl LogFilter {
    pub fn new(config: FilterConfig) -> Self {
        let mut builder = create_default_highlighter();

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
            "V" => "\x1b[37m",
            "D" => "\x1b[36m",
            "I" => "\x1b[32m",
            "W" => "\x1b[33m",
            "E" => "\x1b[31m",
            "F" => "\x1b[1;97;41m",
            _ => RESET_COLOR,
        }
    }

    fn get_tag_color(&self, tag: &str) -> &'static str {
        if tag == "AndroidRuntime" {
            return "\x1b[1;31m";
        }
        match self.tags.category_of(tag) {
            TagCategory::Routing => "\x1b[1;31m",
            TagCategory::MapMatching => "\x1b[33m",
            TagCategory::Guidance => "\x1b[35m",
            TagCategory::Navigation => "\x1b[34m",
        }
    }

    fn is_crash_tag(tag: &str) -> bool {
        tag == "AndroidRuntime"
    }

    pub fn is_crash_line(line: &str) -> bool {
        line.contains(" E AndroidRuntime:")
    }

    fn colorize_crash_message(message: &str) -> String {
        let t = message.trim_start();
        if Self::is_crash_exception_line(t) {
            format!("\x1b[1;31m{message}{RESET_COLOR}") // bold red: exception names / causes
        } else if Self::is_crash_framework_frame(t) {
            format!("\x1b[90m{message}{RESET_COLOR}") // dark gray: framework noise
        } else {
            format!("\x1b[31m{message}{RESET_COLOR}") // red: app frames
        }
    }

    fn is_crash_exception_line(trimmed: &str) -> bool {
        trimmed.starts_with("FATAL EXCEPTION")
            || trimmed.starts_with("Caused by:")
            // Java exception: package.ClassName: message — no space before the colon
            || trimmed
                .find(':')
                .map_or(false, |pos| trimmed[..pos].contains('.') && !trimmed[..pos].contains(' '))
    }

    fn is_crash_framework_frame(trimmed: &str) -> bool {
        trimmed.starts_with("...")
            || [
                "at android.",
                "at java.",
                "at kotlin.",
                "at com.android.",
                "at dalvik.",
                "at sun.",
                "at libcore.",
            ]
            .iter()
            .any(|p| trimmed.starts_with(p))
    }

    pub fn matches(&self, line: &str) -> Option<String> {
        if line.trim().is_empty() {
            return None;
        }

        // Raw stack trace lines (no logcat header) — pass through with dim red.
        if Self::looks_like_stack_trace(line) {
            return Some(format!("\x1b[2;31m{line}\x1b[0m"));
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

        if parts.len() <= tag_idx {
            return None;
        }

        // Empty levels list means all levels are off — block everything.
        if self.levels.is_empty() {
            return None;
        }
        let line_level = parts[level_idx];
        if !self
            .levels
            .iter()
            .any(|level| level.eq_ignore_ascii_case(line_level))
        {
            return None;
        }

        // Check tag filter. FATAL lines bypass tag filtering so crashes always show.
        // When no_tag_filter is set, empty tag list means "show all".
        // Otherwise empty tag list means all category toggles are off → show nothing.
        let line_tag = parts[tag_idx].trim_end_matches(':');
        let is_fatal = line_level.eq_ignore_ascii_case("F");
        let is_crash = line_level.eq_ignore_ascii_case("E") && Self::is_crash_tag(line_tag);
        if !self.no_tag_filter && !is_fatal && !is_crash {
            if self.tags.is_empty() || !self.tags.contains_tag(line_tag) {
                return None;
            }
        }

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
                if is_crash {
                    colored_line.push_str(&Self::colorize_crash_message(&message));
                } else {
                    colored_line.push_str(&self.message_highlighter.highlight_message(&message));
                }
                break;
            } else if i < level_idx {
                colored_line.push_str("\x1b[90m");
                colored_line.push_str(part);
                colored_line.push_str(RESET_COLOR);
            } else {
                colored_line.push_str(part);
            }
            colored_line.push(' ');
        }

        Some(colored_line.trim().to_string())
    }

    fn looks_like_stack_trace(line: &str) -> bool {
        let t = line.trim_start();
        t.starts_with("at ") || t.starts_with("Caused by:") || t.starts_with("Suppressed:")
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
                    && Self::looks_like_pid(parts.get(4).copied())
                {
                    return Some(LogFormat::FullWithPidTid);
                }
                Some(LogFormat::Compact)
            } else {
                // No milliseconds: YYYY-MM-DD HH:MM:SS PID TID LEVEL TAG
                if parts.len() >= 6
                    && Self::looks_like_pid(parts.get(2).copied())
                    && Self::looks_like_pid(parts.get(3).copied())
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
        part.map(|p| p.chars().all(|c| c.is_ascii_digit()))
            .unwrap_or(false)
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
        assert!(matches!(
            LogFilter::detect_format(&parts),
            Some(LogFormat::FullWithPidTid)
        ));
    }

    #[test]
    fn detect_full() {
        // YYYY-MM-DD HH:MM:SS PID TID LEVEL TAG msg
        let line = "2024-01-15 10:30:45 1234 5678 I SomeTag: message";
        let parts: Vec<&str> = line.split_whitespace().collect();
        assert!(matches!(
            LogFilter::detect_format(&parts),
            Some(LogFormat::Full)
        ));
    }

    #[test]
    fn detect_short() {
        // MM-DD HH:MM:SS.mmm PID TID LEVEL TAG msg
        let line = "01-15 10:30:45.123 1234 5678 I SomeTag: message";
        let parts: Vec<&str> = line.split_whitespace().collect();
        assert!(matches!(
            LogFilter::detect_format(&parts),
            Some(LogFormat::Short)
        ));
    }

    #[test]
    fn detect_compact() {
        // YYYY-MM-DD HH:MM:SS.mmm+TZ LEVEL TAG: msg
        let line = "2024-01-15 10:30:45.123+0000 I SomeTag: message";
        let parts: Vec<&str> = line.split_whitespace().collect();
        assert!(matches!(
            LogFilter::detect_format(&parts),
            Some(LogFormat::Compact)
        ));
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
        let filter = make_filter(vec!["I"], vec!["Navigation"], vec![], vec![]);
        let line = "2024-01-15 10:30:45 1234 5678 I DefaultNavigation: hello";
        assert!(filter.matches(line).is_some());
    }

    #[test]
    fn matches_passes_matching_tag_case_insensitively() {
        let filter = make_filter(vec!["I"], vec!["guidance"], vec![], vec![]);
        let line = "2024-01-15 10:30:45 1234 5678 I LaneGuidance: hello";
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
        let filter = make_filter(vec!["I"], vec![], vec![], vec!["replan"]);
        let line = "2024-01-15 10:30:45 1234 5678 I SomeTag: replan triggered";
        assert!(filter.matches(line).is_some());
    }

    #[test]
    fn matches_show_item_is_case_insensitive() {
        let filter = make_filter(vec!["I"], vec![], vec![], vec!["error"]);
        let line = "2024-01-15 10:30:45 1234 5678 I SomeTag: ERROR triggered";
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
    fn matches_stack_trace_lines_pass_through() {
        let filter = make_filter(vec![], vec![], vec![], vec![]);
        assert!(
            filter
                .matches("at com.example.Foo.bar(Foo.kt:42)")
                .is_some()
        );
        assert!(
            filter
                .matches("\tat com.example.Foo.bar(Foo.kt:42)")
                .is_some()
        );
        assert!(
            filter
                .matches("Caused by: java.lang.NullPointerException")
                .is_some()
        );
        assert!(filter.matches("--------- beginning of main").is_none());
    }

    #[test]
    fn matches_stack_trace_lines_are_dim_red() {
        let filter = make_filter(vec![], vec![], vec![], vec![]);
        let result = filter.matches("at com.example.Foo.bar(Foo.kt:42)").unwrap();
        assert!(result.contains("\x1b[2;31m"));
        assert!(result.contains("\x1b[0m"));
    }

    #[test]
    fn matches_fatal_level_uses_background_red() {
        let filter = make_filter(vec!["F"], vec![], vec![], vec![]);
        let line = "2024-01-15 10:30:45 1234 5678 F SomeTag: crash";
        let result = filter.matches(line).unwrap();
        assert!(result.contains("\x1b[1;97;41m"));
    }
}
