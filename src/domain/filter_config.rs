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
                steps.push(tag);
            } else if tag.contains("Engine") {
                engines.push(tag);
            } else {
                top_classes.push(tag);
            }
        }

        Self {
            top_classes,
            steps,
            engines,
            all_tags,
        }
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

/// Runtime-mutable filter state. Holds the immutable parts set from CLI args plus
/// the three category toggles that can be flipped at runtime in the TUI.
#[derive(Debug, Clone)]
pub struct FilterState {
    pub levels: Vec<&'static str>,
    pub base_tags: Vec<String>,
    pub highlighted_items: Vec<String>,
    pub show_items: Vec<String>,
    /// true = show guidance messages
    pub guidance: bool,
    /// true = show routing messages
    pub routing: bool,
    /// true = show map-matching messages
    pub mapmatching: bool,
}

impl FilterState {
    pub fn from_args(args: &Args) -> Self {
        let levels = FilterConfig::to_levels(&args.logcat_levels);
        let mut base_tags = if args.no_tag_filter {
            vec![]
        } else {
            FilterConfig::to_tags(&args.tags)
        };
        for tag in &args.add_tag {
            base_tags.push(tag.clone());
        }

        Logger::info_fmt("Base tags:", &[&base_tags]);

        Self {
            levels,
            base_tags,
            highlighted_items: args.highlighted_items.clone(),
            show_items: args.show_items.clone(),
            guidance: !args.no_guidance,
            routing: !args.no_routing,
            mapmatching: !args.no_mapmatching,
        }
    }

    pub fn to_filter_config(&self) -> FilterConfig {
        let mut tags = self.base_tags.clone();
        let mut blacklisted_items = Vec::new();

        if !self.guidance {
            tags.retain(|tag| !tag.contains("Guidance") && !tag.contains("Warning"));
            blacklisted_items.push("guidance".to_string());
            blacklisted_items.push("instruction".to_string());
            blacklisted_items.push("warning".to_string());
        }

        if !self.routing {
            tags.retain(|tag| !tag.contains("Planner"));
        }

        if !self.mapmatching {
            tags.retain(|tag| !tag.contains("Match") && !tag.contains("Project"));
        }

        FilterConfig {
            levels: self.levels.clone(),
            tags: TagCategories::new(tags),
            blacklisted_items,
            highlighted_items: self.highlighted_items.clone(),
            show_items: self.show_items.clone(),
        }
    }
}

impl FilterConfig {
    pub fn parse(args: &Args) -> Self {
        FilterState::from_args(args).to_filter_config()
    }

    pub(crate) fn to_levels(levels_str: &str) -> Vec<&'static str> {
        levels_str
            .split(',')
            .flat_map(|s| match s {
                "I" => vec!["I", "INFO"],
                "D" => vec!["D", "DEBUG"],
                "E" => vec!["E", "ERROR"],
                "W" => vec!["W", "WARN"],
                "T" => vec!["T", "TRACE"],
                _ => vec!["I", "INFO"],
            })
            .collect()
    }

    pub(crate) fn to_tags(tags_str: &str) -> Vec<String> {
        tags_str.split(',').map(|s| s.trim().to_string()).collect()
    }
}
