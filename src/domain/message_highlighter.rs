use std::collections::HashSet;

use ratatui::text::Span;

use crate::domain::highlight_builder::HighlightPriority;
use crate::shared::logger::Logger;

#[derive(Debug, Clone)]
struct HighlightRule {
    terms: HashSet<String>,
    priority: HighlightPriority,
}

#[derive(Debug, Clone)]
struct Match {
    start: usize,
    end: usize,
    priority: HighlightPriority,
}

impl Match {
    fn new(start: usize, end: usize, priority: HighlightPriority) -> Self {
        Self {
            start,
            end,
            priority,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MessageHighlighter {
    rules: Vec<HighlightRule>,
}

struct MessageProcessor<'a> {
    message: &'a str,
    message_lower: String,
}

impl<'a> MessageProcessor<'a> {
    fn new(message: &'a str) -> Self {
        Self {
            message,
            message_lower: message.to_lowercase(),
        }
    }

    fn find_matches(&self, highlighter: &MessageHighlighter) -> Vec<Match> {
        let mut matches = Vec::new();

        Logger::debug_fmt("Checking message: {}", &[&self.message_lower]);

        for rule in &highlighter.rules {
            for term in &rule.terms {
                for (pos, _) in self.message_lower.match_indices(term.as_str()) {
                    Logger::debug_fmt("Found term at pos {}", &[&pos]);
                    if self.is_complete_match(pos, term.len()) {
                        Logger::debug("Complete match!");
                        matches.push(Match::new(pos, pos + term.len(), rule.priority));
                    } else {
                        Logger::debug("Not a complete match");
                    }
                }
            }
        }

        matches.sort_by_key(|m| m.start);
        self.resolve_overlapping_matches(matches)
    }

    fn resolve_overlapping_matches(&self, matches: Vec<Match>) -> Vec<Match> {
        let mut resolved = Vec::new();
        let mut current: Option<Match> = None;
        let mut overlapping: Vec<Match> = Vec::new();

        for m in matches {
            match current {
                None => {
                    current = Some(m.clone());
                    overlapping.push(m);
                }
                Some(ref curr) => {
                    if m.start <= curr.end {
                        overlapping.push(m);
                    } else {
                        if let Some(best) = self.find_highest_priority_match(&overlapping) {
                            resolved.push(best);
                        }
                        current = Some(m.clone());
                        overlapping = vec![m];
                    }
                }
            }
        }

        if !overlapping.is_empty() {
            if let Some(best) = self.find_highest_priority_match(&overlapping) {
                resolved.push(best);
            }
        }

        resolved
    }

    fn find_highest_priority_match(&self, matches: &[Match]) -> Option<Match> {
        matches.iter().max_by_key(|m| m.priority).cloned()
    }

    fn is_complete_match(&self, pos: usize, word_len: usize) -> bool {
        let is_word_boundary = |c: char| {
            c.is_whitespace() || c.is_ascii_punctuation() || c == '(' || c == ')' || c == '='
        };

        let starts_at_boundary = pos == 0
            || self.message_lower[..pos]
                .chars()
                .next_back()
                .is_none_or(is_word_boundary);

        let ends_at_boundary = pos + word_len == self.message_lower.len()
            || self.message_lower[pos + word_len..]
                .chars()
                .next()
                .is_none_or(is_word_boundary);

        starts_at_boundary && ends_at_boundary
    }

    fn build_spans(&self, matches: Vec<Match>) -> Vec<Span<'static>> {
        let mut spans = Vec::new();
        let mut last_end = 0;

        for m in matches {
            if m.start > last_end {
                spans.push(Span::raw(self.message[last_end..m.start].to_owned()));
            }
            spans.push(Span::styled(
                self.message[m.start..m.end].to_owned(),
                m.priority.style(),
            ));
            last_end = m.end;
        }

        if last_end < self.message.len() {
            spans.push(Span::raw(self.message[last_end..].to_owned()));
        }

        spans
    }
}

impl MessageHighlighter {
    pub fn new(
        red_words: HashSet<String>,
        green_words: HashSet<String>,
        yellow_words: HashSet<String>,
        custom_words: HashSet<String>,
    ) -> Self {
        Self {
            rules: vec![
                HighlightRule {
                    terms: red_words,
                    priority: HighlightPriority::Red,
                },
                HighlightRule {
                    terms: yellow_words,
                    priority: HighlightPriority::Yellow,
                },
                HighlightRule {
                    terms: green_words,
                    priority: HighlightPriority::Green,
                },
                HighlightRule {
                    terms: custom_words,
                    priority: HighlightPriority::Custom,
                },
            ],
        }
    }

    pub fn highlight_message(&self, message: &str) -> Vec<Span<'static>> {
        let processor = MessageProcessor::new(message);
        let matches = processor.find_matches(self);
        if matches.is_empty() {
            return vec![Span::raw(message.to_owned())];
        }
        processor.build_spans(matches)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::highlight_builder::HighlightPriority;

    fn make_highlighter(
        red: &[&str],
        green: &[&str],
        yellow: &[&str],
        custom: &[&str],
    ) -> MessageHighlighter {
        let to_set = |words: &[&str]| words.iter().map(|w| w.to_lowercase()).collect();
        MessageHighlighter::new(to_set(red), to_set(green), to_set(yellow), to_set(custom))
    }

    fn has_style(spans: &[Span], priority: HighlightPriority) -> bool {
        spans.iter().any(|s| s.style == priority.style())
    }

    fn count_style(spans: &[Span], priority: HighlightPriority) -> usize {
        spans.iter().filter(|s| s.style == priority.style()).count()
    }

    #[test]
    fn red_beats_yellow_on_overlap() {
        let h = make_highlighter(&["error"], &[], &["error"], &[]);
        let result = h.highlight_message("error occurred");
        assert!(has_style(&result, HighlightPriority::Red));
        assert!(!has_style(&result, HighlightPriority::Yellow));
    }

    #[test]
    fn red_beats_green_on_overlap() {
        let h = make_highlighter(&["started"], &["started"], &[], &[]);
        let result = h.highlight_message("started successfully");
        assert!(has_style(&result, HighlightPriority::Red));
        assert!(!has_style(&result, HighlightPriority::Green));
    }

    #[test]
    fn yellow_beats_green_on_overlap() {
        let h = make_highlighter(&[], &["progress"], &["progress"], &[]);
        let result = h.highlight_message("progress update");
        assert!(has_style(&result, HighlightPriority::Yellow));
        assert!(!has_style(&result, HighlightPriority::Green));
    }

    #[test]
    fn builtin_beats_custom_on_overlap() {
        let h = make_highlighter(&["error"], &[], &[], &["error"]);
        let result = h.highlight_message("error occurred");
        assert!(has_style(&result, HighlightPriority::Red));
        assert!(!has_style(&result, HighlightPriority::Custom));
    }

    #[test]
    fn all_occurrences_highlighted() {
        let h = make_highlighter(&["error"], &[], &[], &[]);
        let result = h.highlight_message("error then another error here");
        assert_eq!(count_style(&result, HighlightPriority::Red), 2);
    }

    #[test]
    fn single_occurrence_highlighted_once() {
        let h = make_highlighter(&["error"], &[], &[], &[]);
        let result = h.highlight_message("just one error here");
        assert_eq!(count_style(&result, HighlightPriority::Red), 1);
    }

    #[test]
    fn partial_word_not_highlighted() {
        let h = make_highlighter(&["old"], &[], &[], &[]);
        let result = h.highlight_message("unfolded map");
        assert!(!has_style(&result, HighlightPriority::Red));
    }

    #[test]
    fn exact_word_is_highlighted() {
        let h = make_highlighter(&["old"], &[], &[], &[]);
        let result = h.highlight_message("the old route");
        assert!(has_style(&result, HighlightPriority::Red));
    }
}
