use std::collections::HashSet;

#[derive(Debug, Clone)]
struct HighlightRule {
    words: HashSet<String>,
    color: &'static str,
}

#[derive(Debug, Clone)]
pub struct MessageHighlighter {
    rules: Vec<HighlightRule>,
    red_rule: HighlightRule,
    green_rule: HighlightRule,
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
            words: red_items,
            color: "\x1b[1;31m", // Bold Red
        };

        // Green highlights for positive messages/information
        let mut green_items = HashSet::new();
        green_items.insert("followed".to_string());
        green_items.insert("progress".to_string());
        green_items.insert("success".to_string());

        let green_rule = HighlightRule {
            words: green_items,
            color: "\x1b[1;32m", // Bold Green
        };

        Self {
            rules: Vec::new(),
            red_rule,
            green_rule,
        }
    }

    pub fn highlight_message(&self, message: &str) -> String {
        let mut highlighted = String::new();
        let words: Vec<&str> = message.split_whitespace().collect();

        for word in words {
            let word_lower = word.to_lowercase();
            let mut found_rule = false;

            // Check red words first
            if self.red_rule.words.iter().any(|w| word_lower.contains(w)) {
                highlighted.push_str(self.red_rule.color);
                highlighted.push_str(word);
                highlighted.push_str("\x1b[0m");
                found_rule = true;
            }
            // Then check green words
            else if self
                .green_rule
                .words
                .iter()
                .any(|w| word_lower.contains(w))
            {
                highlighted.push_str(self.green_rule.color);
                highlighted.push_str(word);
                highlighted.push_str("\x1b[0m");
                found_rule = true;
            }
            // Finally check other rules
            else {
                for rule in &self.rules {
                    if rule.words.iter().any(|w| word_lower.contains(w)) {
                        highlighted.push_str(rule.color);
                        highlighted.push_str(word);
                        highlighted.push_str("\x1b[0m");
                        found_rule = true;
                        break;
                    }
                }
            }

            if !found_rule {
                highlighted.push_str(word);
            }
            highlighted.push(' ');
        }

        highlighted.trim().to_string()
    }

    pub fn add_highlight_word(&mut self, word: String, color: &'static str) {
        let word_lower = word.to_lowercase();

        match color {
            "\x1b[1;31m" => self.red_rule.words.insert(word_lower),
            "\x1b[1;32m" => self.green_rule.words.insert(word_lower),
            _ => {
                // Find or create a rule for other colors
                if let Some(rule) = self.rules.iter_mut().find(|r| r.color == color) {
                    rule.words.insert(word_lower)
                } else {
                    let mut words = HashSet::new();
                    words.insert(word_lower);
                    self.rules.push(HighlightRule { words, color });
                    true
                }
            }
        };
    }

    pub fn remove_highlight_word(&mut self, word: &str) {
        let word_lower = word.to_lowercase();
        self.red_rule.words.remove(&word_lower);
        self.green_rule.words.remove(&word_lower);
        for rule in &mut self.rules {
            rule.words.remove(&word_lower);
        }
    }
}
