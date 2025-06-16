use std::collections::HashSet;

use crate::domain::filter::RESET_COLOR;
use crate::domain::highlight_builder::{BG_YELLOW, GREEN_COLOR, RED_COLOR, YELLOW_COLOR};
use crate::shared::logger::Logger;

#[derive(Debug, Clone)]
struct HighlightRule {
    terms: HashSet<String>,
    color: &'static str,
}

#[derive(Debug, Clone)]
struct Match {
    start: usize,
    end: usize,
    color: &'static str,
}

impl Match {
    fn new(start: usize, end: usize, color: &'static str) -> Self {
        Self { start, end, color }
    }
}

#[derive(Debug, Clone)]
pub struct MessageHighlighter {
    rules: Vec<HighlightRule>,
    red_rule: HighlightRule,
    yellow_rule: HighlightRule,
    green_rule: HighlightRule,
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

        // First check for exact phrases in red rules
        for term in &highlighter.red_rule.terms {
            if let Some(pos) = self.message_lower.find(term) {
                Logger::debug_fmt("Found red term at pos {}", &[&pos]);
                if self.is_complete_match(pos, term.len()) {
                    Logger::debug("Complete match!");
                    matches.push(Match::new(
                        pos,
                        pos + term.len(),
                        highlighter.red_rule.color,
                    ));
                } else {
                    Logger::debug("Not a complete match");
                }
            } else {
                Logger::debug("Term not found");
            }
        }

        // First check for exact phrases in yellow rules
        for term in &highlighter.yellow_rule.terms {
            if let Some(pos) = self.message_lower.find(term) {
                Logger::debug_fmt("Found yellow term at pos {}", &[&pos]);
                if self.is_complete_match(pos, term.len()) {
                    Logger::debug("Complete match!");
                    matches.push(Match::new(
                        pos,
                        pos + term.len(),
                        highlighter.yellow_rule.color,
                    ));
                } else {
                    Logger::debug("Not a complete match");
                }
            } else {
                Logger::debug("Term not found");
            }
        }

        // Then check for exact phrases in green rules
        for term in &highlighter.green_rule.terms {
            if let Some(pos) = self.message_lower.find(term) {
                Logger::debug_fmt("Found green term at pos {}", &[&pos]);
                if self.is_complete_match(pos, term.len()) {
                    Logger::debug("Complete match!");
                    matches.push(Match::new(
                        pos,
                        pos + term.len(),
                        highlighter.green_rule.color,
                    ));
                } else {
                    Logger::debug("Not a complete match");
                }
            } else {
                Logger::debug("Term not found");
            }
        }

        // Finally check other rules
        for rule in &highlighter.rules {
            for term in &rule.terms {
                if let Some(pos) = self.message_lower.find(term) {
                    Logger::debug_fmt("Found custom term at pos {}", &[&pos]);
                    if self.is_complete_match(pos, term.len()) {
                        Logger::debug("Complete match!");
                        matches.push(Match::new(pos, pos + term.len(), rule.color));
                    } else {
                        Logger::debug("Not a complete match");
                    }
                } else {
                    Logger::debug("Term not found");
                }
            }
        }

        // Sort matches by start position
        matches.sort_by_key(|m| m.start);

        // Resolve overlapping matches
        self.resolve_overlapping_matches(matches, highlighter)
    }

    fn resolve_overlapping_matches(
        &self,
        matches: Vec<Match>,
        highlighter: &MessageHighlighter,
    ) -> Vec<Match> {
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
                        // Still overlapping with current group
                        overlapping.push(m);
                    } else {
                        // No longer overlapping, resolve current group
                        if let Some(best_match) =
                            self.find_highest_priority_match(&overlapping, highlighter)
                        {
                            resolved.push(best_match);
                        }
                        // Start new group
                        current = Some(m.clone());
                        overlapping = vec![m];
                    }
                }
            }
        }

        // Resolve final group
        if !overlapping.is_empty() {
            if let Some(best_match) = self.find_highest_priority_match(&overlapping, highlighter) {
                resolved.push(best_match);
            }
        }

        resolved
    }

    fn find_highest_priority_match(
        &self,
        matches: &[Match],
        highlighter: &MessageHighlighter,
    ) -> Option<Match> {
        matches
            .iter()
            .max_by_key(|m| {
                match m.color {
                    c if c == highlighter.red_rule.color => 3, // Red has highest priority
                    c if c == highlighter.yellow_rule.color => 2, // Yellow has second priority
                    c if c == highlighter.green_rule.color => 1, // Green has third priority
                    _ => 4, // custom colors have the higest priority
                }
            })
            .cloned()
    }

    fn is_complete_match(&self, pos: usize, word_len: usize) -> bool {
        let is_word_boundary = |c: char| {
            c.is_whitespace() || c.is_ascii_punctuation() || c == '(' || c == ')' || c == '='
        };

        // Check if match starts at word boundary
        let starts_at_boundary =
            pos == 0 || is_word_boundary(self.message_lower.as_bytes()[pos - 1] as char);

        // Check if match ends at word boundary
        let ends_at_boundary = pos + word_len == self.message_lower.len()
            || is_word_boundary(self.message_lower.as_bytes()[pos + word_len] as char);

        starts_at_boundary && ends_at_boundary
    }

    fn build_highlighted(&self, matches: Vec<Match>) -> String {
        let mut highlighted = String::new();
        let mut last_end = 0;

        for m in matches {
            // Add uncolored text before the match
            if m.start > last_end {
                highlighted.push_str(&self.message[last_end..m.start]);
            }
            // Add colored match
            highlighted.push_str(m.color);
            highlighted.push_str(&self.message[m.start..m.end]);
            highlighted.push_str(RESET_COLOR);
            last_end = m.end;
        }

        // Add any remaining uncolored text
        if last_end < self.message.len() {
            highlighted.push_str(&self.message[last_end..]);
        }

        highlighted
    }
}

impl MessageHighlighter {
    pub fn new(
        red_words: HashSet<String>,
        green_words: HashSet<String>,
        yellow_words: HashSet<String>,
        custom_words: HashSet<String>,
    ) -> Self {
        let red_rule = HighlightRule {
            terms: red_words,
            color: RED_COLOR,
        };

        let green_rule = HighlightRule {
            terms: green_words,
            color: GREEN_COLOR,
        };

        let yellow_rule = HighlightRule {
            terms: yellow_words,
            color: YELLOW_COLOR,
        };

        let custom_rule = HighlightRule {
            terms: custom_words,
            color: BG_YELLOW,
        };

        Self {
            rules: vec![custom_rule],
            red_rule,
            yellow_rule,
            green_rule,
        }
    }

    pub fn highlight_message(&self, message: &str) -> String {
        let processor = MessageProcessor::new(message);
        let matches = processor.find_matches(self);
        processor.build_highlighted(matches)
    }

    // TODO: use or remove
    // pub fn add_highlight_word(&mut self, word: String, color: &'static str) {
    //     let word_lower = word.to_lowercase();

    //     match color {
    //         c if c == RED_COLOR => self.red_rule.terms.insert(word_lower),
    //         c if c == GREEN_COLOR => self.green_rule.terms.insert(word_lower),
    //         c if c == YELLOW_COLOR => self.yellow_rule.terms.insert(word_lower),
    //         _ => {
    //             // Find or create a rule for other colors
    //             if let Some(rule) = self.rules.iter_mut().find(|r| r.color == color) {
    //                 rule.terms.insert(word_lower)
    //             } else {
    //                 let mut words = HashSet::new();
    //                 words.insert(word_lower);
    //                 self.rules.push(HighlightRule {
    //                     terms: words,
    //                     color,
    //                 });
    //                 true
    //             }
    //         }
    //     };
    // }

    // pub fn remove_highlight_word(&mut self, word: &str) {
    //     let word_lower = word.to_lowercase();
    //     self.red_rule.terms.remove(&word_lower);
    //     self.green_rule.terms.remove(&word_lower);
    //     self.yellow_rule.terms.remove(&word_lower);
    //     for rule in &mut self.rules {
    //         rule.terms.remove(&word_lower);
    //     }
    // }
}
