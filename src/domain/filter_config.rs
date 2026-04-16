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
    /// When true, empty tag list means "show all". When false, empty tag list means "show nothing".
    pub no_tag_filter: bool,
}

#[derive(Debug, Clone)]
pub struct TagCategories {
    pub routing_tags: Vec<String>,
    pub mapmatching_tags: Vec<String>,
    pub guidance_tags: Vec<String>,
    pub all_tags: HashSet<String>,
}

impl TagCategories {
    pub fn new(tags: Vec<String>) -> Self {
        let mut routing_tags = Vec::new();
        let mut mapmatching_tags = Vec::new();
        let mut guidance_tags = Vec::new();
        let mut all_tags = HashSet::new();

        for tag in tags {
            all_tags.insert(tag.clone());
            if tag.contains("Planner") || tag.contains("Replan") {
                routing_tags.push(tag);
            } else if tag.contains("Match") || tag.contains("Project") {
                mapmatching_tags.push(tag);
            } else if tag.contains("Guidance") || tag.contains("Warning") {
                guidance_tags.push(tag);
            }
            // everything else is blue by default — no bucket needed
        }

        Self {
            routing_tags,
            mapmatching_tags,
            guidance_tags,
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
/// the four category toggles that can be flipped at runtime in the TUI.
#[derive(Debug, Clone)]
pub struct FilterState {
    pub levels: Vec<&'static str>,
    pub base_tags: Vec<String>,
    pub highlighted_items: Vec<String>,
    pub show_items: Vec<String>,
    pub no_tag_filter: bool,
    /// true = show core navigation messages (progress, tracking, waypoints, …)
    pub navigation: bool,
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
            no_tag_filter: args.no_tag_filter,
            navigation: true,
            guidance: true,
            routing: true,
            mapmatching: true,
        }
    }

    pub fn to_filter_config(&self) -> FilterConfig {
        // Additive model: each toggle owns its tag bucket exclusively.
        // Tags are assigned to exactly one category by pattern, and only
        // tags in enabled categories are passed to the filter. This means
        // the visible set is the union of enabled categories — all off
        // produces an empty tag list, which the filter treats as "show nothing"
        // (contrast with no_tag_filter=true, which means "show everything").
        let mut tags = Vec::new();
        let mut blacklisted_items = Vec::new();

        for tag in &self.base_tags {
            let enabled = if tag.contains("Guidance") || tag.contains("Warning") {
                self.guidance
            } else if tag.contains("Planner") {
                self.routing
            } else if tag.contains("Match") || tag.contains("Project") {
                self.mapmatching
            } else {
                self.navigation
            };
            if enabled {
                tags.push(tag.clone());
            }
        }

        if !self.guidance {
            blacklisted_items.push("guidance".to_string());
            blacklisted_items.push("instruction".to_string());
            blacklisted_items.push("warning".to_string());
        }

        FilterConfig {
            levels: self.levels.clone(),
            tags: TagCategories::new(tags),
            blacklisted_items,
            highlighted_items: self.highlighted_items.clone(),
            show_items: self.show_items.clone(),
            no_tag_filter: self.no_tag_filter,
        }
    }
}

impl FilterConfig {
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
