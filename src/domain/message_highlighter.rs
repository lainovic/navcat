use std::collections::HashSet;

use crate::domain::filter::RESET_COLOR;
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
                    _ => 0,                                    // Others have lowest priority
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
    pub fn new(highlighted_items: Vec<String>) -> Self {
        // Red highlights for warnings/errors/deviations
        let mut red_items = HashSet::new();

        red_items.insert("error".to_string());

        red_items.insert("old".to_string());
        red_items.insert("removed".to_string());

        red_items.insert("unfollowed".to_string());
        red_items.insert("not followed".to_string());
        red_items.insert("unvisited".to_string());
        red_items.insert("deviation".to_string());
        red_items.insert("off-road".to_string());

        let red_rule = HighlightRule {
            terms: red_items,
            color: "\x1b[1;31m", // Bold Red
        };

        // Green highlights for positive messages/information
        let mut green_items = HashSet::new();

        green_items.insert("success".to_string());

        green_items.insert("added".to_string());

        // tracking engine
        green_items.insert("following".to_string());
        green_items.insert("followed".to_string());
        green_items.insert("visited".to_string());

        // route planner
        green_items.insert("planned".to_string());

        let green_rule = HighlightRule {
            terms: green_items,
            color: "\x1b[1;32m", // Bold Green
        };

        // Yellow highlights for navigation and map matching events
        let mut yellow_items = HashSet::new();

        yellow_items.insert("warning".to_string());
        yellow_items.insert("updated".to_string());
        yellow_items.insert("changed".to_string());

        yellow_items.insert("segment".to_string());

        // map-matcher
        yellow_items.insert("map matching".to_string());
        yellow_items.insert("projected".to_string());
        yellow_items.insert("matchlocation".to_string());
        yellow_items.insert("matched".to_string());

        // replan actions
        yellow_items.insert("should replan".to_string());
        yellow_items.insert("refresh".to_string());
        yellow_items.insert("back to route".to_string());
        yellow_items.insert("continuous replanning".to_string());
        yellow_items.insert("full replanning".to_string());
        yellow_items.insert("language change".to_string());
        yellow_items.insert("increment".to_string());

        // progress engine
        yellow_items.insert("progress".to_string());
        yellow_items.insert("current location".to_string());
        yellow_items.insert("distancealongroute".to_string());

        // guidance engine
        yellow_items.insert("traffic jam".to_string());
        yellow_items.insert("guidance".to_string());
        yellow_items.insert("instruction".to_string());

        // route planner
        yellow_items.insert("planning route".to_string());
        yellow_items.insert("route".to_string());

        let yellow_rule = HighlightRule {
            terms: yellow_items,
            color: "\x1b[33m", // Yellow
        };

        let rule_for_highlights = HighlightRule {
            terms: highlighted_items.into_iter().collect(),
            color: "\x1b[43m", // Background Yellow
        };

        Self {
            rules: vec![rule_for_highlights],
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

    pub fn add_highlight_word(&mut self, word: String, color: &'static str) {
        let word_lower = word.to_lowercase();

        match color {
            "\x1b[1;31m" => self.red_rule.terms.insert(word_lower),
            "\x1b[1;32m" => self.green_rule.terms.insert(word_lower),
            _ => {
                // Find or create a rule for other colors
                if let Some(rule) = self.rules.iter_mut().find(|r| r.color == color) {
                    rule.terms.insert(word_lower)
                } else {
                    let mut words = HashSet::new();
                    words.insert(word_lower);
                    self.rules.push(HighlightRule {
                        terms: words,
                        color,
                    });
                    true
                }
            }
        };
    }

    pub fn remove_highlight_word(&mut self, word: &str) {
        let word_lower = word.to_lowercase();
        self.red_rule.terms.remove(&word_lower);
        self.green_rule.terms.remove(&word_lower);
        for rule in &mut self.rules {
            rule.terms.remove(&word_lower);
        }
    }
}
