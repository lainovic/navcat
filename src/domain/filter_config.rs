use std::collections::HashSet;

use crate::application::cli::Args;
use crate::shared::logger::Logger;

#[derive(Debug)]
pub struct FilterConfig {
    pub levels: Vec<&'static str>,
    pub tags: TagCategories,
    pub blacklisted_items: Vec<String>,
    pub highlighted_items: Vec<String>,
    pub show_items: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TagCategories {
    pub top_classes: Vec<String>,
    pub steps: Vec<String>,
    pub engines: Vec<String>,
    pub all_tags: HashSet<String>,
}

impl TagCategories {
    pub fn new(tags: Vec<String>) -> Self {
        let mut top_classes = Vec::new();
        let mut steps = Vec::new();
        let mut engines = Vec::new();
        let mut all_tags = HashSet::new();

        for tag in tags {
            all_tags.insert(tag.clone());
            if tag.contains("Step") {
                Self::add_tag(tag, &mut steps);
            } else if tag.contains("Engine") {
                Self::add_tag(tag, &mut engines);
            } else {
                Self::add_tag(tag, &mut top_classes);
            }
        }

        Self {
            top_classes,
            steps,
            engines,
            all_tags,
        }
    }

    fn add_tag(tag: String, collection: &mut Vec<String>) {
        collection.push(tag);
    }

    pub fn contains_tag(&self, tag: &str) -> bool {
        let tag_lower = tag.to_lowercase();
        Logger::debug_fmt("Checking tag:", &[&tag_lower]);
        Logger::debug_fmt("Available tags:", &[&self.all_tags]);
        let result = self
            .all_tags
            .iter()
            .any(|t| tag_lower.contains(&t.to_lowercase()));
        Logger::debug_fmt("Match result:", &[&result]);
        result
    }
}

impl FilterConfig {
    pub fn parse(args: &Args) -> Self {
        let levels = Self::to_levels(&args.logcat_levels);
        let mut tags = Self::to_tags(&args.tags);
        let mut blacklisted_items = Vec::new();
        let mut highlighted_items = Vec::new();
        let mut show_items = Vec::new();

        // Add any additional tags from the command line
        for tag in &args.add_tag {
            Logger::info_fmt("Adding tag:", &[&tag]);
            tags.push(tag.clone());
        }

        Logger::debug_fmt("All tags before filtering:", &[&tags]);

        if !args.guidance {
            tags = tags
                .into_iter()
                .filter(|tag| !tag.contains("Guidance") && !tag.contains("Warning"))
                .collect();
            blacklisted_items.push("guidance".to_string());
            blacklisted_items.push("instruction".to_string());
            blacklisted_items.push("warning".to_string());
        }

        Logger::debug_fmt("All tags after guidance filter:", &[&tags]);

        if !args.routing {
            tags = tags
                .into_iter()
                .filter(|tag| !tag.contains("Planner"))
                .collect();
        }

        Logger::debug_fmt("All tags after routing filter:", &[&tags]);

        if !args.mapmatching {
            tags = tags
                .into_iter()
                .filter(|tag| !tag.contains("Match") && !tag.contains("Project"))
                .collect();
        }

        Logger::debug_fmt("Final tags:", &[&tags]);

        if !args.highlighted_items.is_empty() {
            highlighted_items = args
                .highlighted_items
                .split(",")
                .map(|s| s.trim().to_string())
                .collect()
        }

        if !args.show_items.is_empty() {
            show_items = args
                .show_items
                .split(",")
                .map(|s| s.trim().to_string())
                .collect()
        }

        Self {
            levels,
            tags: TagCategories::new(tags),
            blacklisted_items,
            highlighted_items,
            show_items,
        }
    }

    fn to_levels(levels_str: &str) -> Vec<&'static str> {
        return levels_str
            .split(',')
            .map(|s| match s {
                "I" => vec!["I", "INFO"],
                "D" => vec!["D", "DEBUG"],
                "E" => vec!["E", "ERROR"],
                "W" => vec!["W", "WARN"],
                "T" => vec!["T", "TRACE"],
                _ => vec!["I", "INFO"],
            })
            .flatten()
            .collect();
    }

    fn to_tags(tags_str: &str) -> Vec<String> {
        return tags_str.split(',').map(|s| s.trim().to_string()).collect();
    }
}
