use std::collections::HashSet;
use std::fmt;

use crate::domain::filter::RESET_COLOR;

#[derive(Debug, Clone)]
struct Logger {
    verbose: bool,
}

impl Logger {
    fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    fn log(&self, msg: &str) {
        if self.verbose {
            println!("--> {}", msg);
        }
    }

    fn log_fmt(&self, msg: &str, args: &[&dyn fmt::Display]) {
        if self.verbose {
            match args.len() {
                0 => println!("--> {}", msg),
                1 => println!("--> {}", format!("{}", args[0])),
                2 => println!("--> {}", format!("{} {}", args[0], args[1])),
                _ => println!("--> {}", msg),
            }
        }
    }
}

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
    green_rule: HighlightRule,
    logger: Logger,
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
        
        highlighter.logger.log_fmt("Checking message: {}", &[&self.message_lower]);

        // First check for exact phrases in red rules
        for term in &highlighter.red_rule.terms {
            highlighter.logger.log_fmt("Checking red term: '{}'", &[term]);
            if let Some(pos) = self.message_lower.find(term) {
                highlighter.logger.log_fmt("Found term at pos {}", &[&pos]);
                if self.is_complete_match(pos, term.len()) {
                    highlighter.logger.log("Complete match!");
                    matches.push(Match::new(pos, pos + term.len(), highlighter.red_rule.color));
                } else {
                    highlighter.logger.log("Not a complete match");
                }
            } else {
                highlighter.logger.log("Term not found");
            }
        }
        
        // Then check for exact phrases in green rules
        for term in &highlighter.green_rule.terms {
            if let Some(pos) = self.message_lower.find(term) {
                if self.is_complete_match(pos, term.len()) {
                    matches.push(Match::new(pos, pos + term.len(), highlighter.green_rule.color));
                }
            }
        }
        
        // Finally check other rules
        for rule in &highlighter.rules {
            for term in &rule.terms {
                if let Some(pos) = self.message_lower.find(term) {
                    if self.is_complete_match(pos, term.len()) {
                        matches.push(Match::new(pos, pos + term.len(), rule.color));
                    }
                }
            }
        }

        // Sort matches by start position
        matches.sort_by_key(|m| m.start);

        // Resolve overlapping matches
        self.resolve_overlapping_matches(matches, highlighter)
    }

    fn resolve_overlapping_matches(&self, matches: Vec<Match>, highlighter: &MessageHighlighter) -> Vec<Match> {
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
                        if let Some(best_match) = self.find_highest_priority_match(&overlapping, highlighter) {
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

    fn find_highest_priority_match(&self, matches: &[Match], highlighter: &MessageHighlighter) -> Option<Match> {
        matches.iter().max_by_key(|m| {
            match m.color {
                c if c == highlighter.red_rule.color => 2,  // Red has highest priority
                c if c == highlighter.green_rule.color => 1, // Green has second priority
                _ => 0  // Others have lowest priority
            }
        }).cloned()
    }

    fn is_complete_match(&self, pos: usize, word_len: usize) -> bool {
        // Check if match starts at word boundary
        let starts_at_boundary = pos == 0 || {
            let prev_char = self.message_lower.as_bytes()[pos - 1] as char;
            prev_char.is_whitespace() || prev_char.is_ascii_punctuation()
        };

        // Check if match ends at word boundary
        let ends_at_boundary = pos + word_len == self.message_lower.len() || {
            let next_char = self.message_lower.as_bytes()[pos + word_len] as char;
            next_char.is_whitespace() || next_char.is_ascii_punctuation()
        };
        
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
    pub fn new() -> Self {
        // Red highlights for warnings/errors/deviations
        let mut red_items = HashSet::new();
        red_items.insert("unfollowed".to_string());
        red_items.insert("not followed".to_string());
        red_items.insert("deviation".to_string());
        red_items.insert("error".to_string());
        red_items.insert("warning".to_string());

        let red_rule = HighlightRule {
            terms: red_items,
            color: "\x1b[1;31m", // Bold Red
        };

        // Green highlights for positive messages/information
        let mut green_items = HashSet::new();
        green_items.insert("followed".to_string());
        green_items.insert("progress".to_string());
        green_items.insert("success".to_string());
        green_items.insert("planning route".to_string());

        let green_rule = HighlightRule {
            terms: green_items,
            color: "\x1b[1;32m", // Bold Green
        };

        Self {
            rules: Vec::new(),
            red_rule,
            green_rule,
            logger: Logger::new(false),
        }
    }

    pub fn set_verbose(&mut self, verbose: bool) {
        self.logger = Logger::new(verbose);
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
                    self.rules.push(HighlightRule { terms: words, color });
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
