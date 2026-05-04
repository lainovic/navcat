use std::collections::HashMap;

use crate::application::cli::Args;
use crate::shared::logger::Logger;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagCategory {
    Navigation,
    Guidance,
    Routing,
    MapMatching,
}

impl TagCategory {
    pub fn classify(tag: &str) -> Self {
        let tag = tag.to_ascii_lowercase();
        if tag.contains("planner") || tag.contains("replan") {
            Self::Routing
        } else if tag.contains("match") || tag.contains("project") {
            Self::MapMatching
        } else if tag.contains("guidance") || tag.contains("warning") {
            Self::Guidance
        } else {
            Self::Navigation
        }
    }
}

#[derive(Debug, Clone)]
pub struct LevelState {
    pub verbose: bool,
    pub debug: bool,
    pub info: bool,
    pub warn: bool,
    pub error: bool,
    pub fatal: bool,
}

impl LevelState {
    pub fn default_levels() -> Self {
        Self {
            verbose: false,
            debug: true,
            info: true,
            warn: true,
            error: true,
            fatal: true,
        }
    }

    pub fn all_off() -> Self {
        Self {
            verbose: false,
            debug: false,
            info: false,
            warn: false,
            error: false,
            fatal: false,
        }
    }

    pub fn from_str(s: &str) -> Self {
        let mut ls = Self {
            verbose: false,
            debug: false,
            info: false,
            warn: false,
            error: false,
            fatal: false,
        };
        for part in s.split(',') {
            match part.trim() {
                "V" | "VERBOSE" => ls.verbose = true,
                "D" | "DEBUG" => ls.debug = true,
                "I" | "INFO" => ls.info = true,
                "W" | "WARN" => ls.warn = true,
                "E" | "ERROR" => ls.error = true,
                "F" | "FATAL" => ls.fatal = true,
                _ => {}
            }
        }
        ls
    }

    pub fn to_levels(&self) -> Vec<&'static str> {
        let mut v = Vec::new();
        if self.verbose {
            v.push("V");
        }
        if self.debug {
            v.push("D");
        }
        if self.info {
            v.push("I");
        }
        if self.warn {
            v.push("W");
        }
        if self.error {
            v.push("E");
        }
        if self.fatal {
            v.push("F");
        }
        v
    }
}

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
    tags: HashMap<String, TagCategory>,
}

impl TagCategories {
    pub fn new(tags: Vec<String>) -> Self {
        Self {
            tags: tags
                .into_iter()
                .map(|t| {
                    let cat = TagCategory::classify(&t);
                    (t, cat)
                })
                .collect(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.tags.is_empty()
    }

    pub fn contains_tag(&self, tag: &str) -> bool {
        let tag_lower = tag.to_ascii_lowercase();
        self.tags
            .keys()
            .any(|t| tag_lower.contains(&t.to_ascii_lowercase()))
    }

    /// Returns the category of the first stored tag that is a substring of `tag`,
    /// checked in priority order: Routing > MapMatching > Guidance > Navigation.
    pub fn category_of(&self, tag: &str) -> TagCategory {
        let tag_lower = tag.to_ascii_lowercase();
        for &priority in &[
            TagCategory::Routing,
            TagCategory::MapMatching,
            TagCategory::Guidance,
        ] {
            if self
                .tags
                .iter()
                .any(|(t, &cat)| cat == priority && tag_lower.contains(&t.to_ascii_lowercase()))
            {
                return priority;
            }
        }
        TagCategory::Navigation
    }
}

/// Runtime-mutable filter state. Holds the immutable parts set from CLI args plus
/// the four category toggles that can be flipped at runtime in the TUI.
#[derive(Debug, Clone)]
pub struct FilterState {
    pub level_state: LevelState,
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
        let level_state = LevelState::from_str(&args.logcat_levels);
        let mut base_tags = if args.no_tag_filter {
            vec![]
        } else {
            FilterConfig::to_tags(&args.tags)
        };
        base_tags.extend(
            args.add_tag
                .iter()
                .map(String::as_str)
                .map(str::trim)
                .filter(|tag| !tag.is_empty())
                .map(ToOwned::to_owned),
        );

        Logger::info_fmt("Base tags:", &[&base_tags]);

        Self {
            level_state,
            base_tags,
            highlighted_items: args.highlighted_items.clone(),
            show_items: args
                .show_items
                .iter()
                .map(|s| s.to_ascii_lowercase())
                .collect(),
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
            let enabled = match TagCategory::classify(tag) {
                TagCategory::Navigation => self.navigation,
                TagCategory::Guidance => self.guidance,
                TagCategory::Routing => self.routing,
                TagCategory::MapMatching => self.mapmatching,
            };
            if enabled {
                tags.push(tag.clone());
            }
        }

        if !self.guidance {
            blacklisted_items.extend(GUIDANCE_BLACKLIST.iter().map(|&s| s.to_string()));
        }

        FilterConfig {
            levels: self.level_state.to_levels(),
            tags: TagCategories::new(tags),
            blacklisted_items,
            highlighted_items: self.highlighted_items.clone(),
            show_items: self.show_items.clone(),
            no_tag_filter: self.no_tag_filter,
        }
    }
}

const GUIDANCE_BLACKLIST: &[&str] = &["guidance", "instruction", "warning"];

impl FilterConfig {
    pub(crate) fn to_tags(tags_str: &str) -> Vec<String> {
        tags_str
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(ToOwned::to_owned)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_replan_as_routing() {
        assert_eq!(TagCategory::classify("ReplanEngine"), TagCategory::Routing);
    }

    #[test]
    fn classify_project_as_mapmatching() {
        assert_eq!(
            TagCategory::classify("ProjectionStep"),
            TagCategory::MapMatching
        );
    }

    #[test]
    fn classify_guidance_as_guidance() {
        assert_eq!(TagCategory::classify("LaneGuidance"), TagCategory::Guidance);
    }

    #[test]
    fn classify_unknown_as_navigation() {
        assert_eq!(
            TagCategory::classify("DefaultRouteTrackingEngine"),
            TagCategory::Navigation
        );
    }

    #[test]
    fn classify_lowercase_tags() {
        assert_eq!(TagCategory::classify("replan"), TagCategory::Routing);
        assert_eq!(TagCategory::classify("guidance"), TagCategory::Guidance);
    }

    #[test]
    fn to_tags_drops_empty_entries() {
        assert_eq!(FilterConfig::to_tags("foo, ,bar,,"), vec!["foo", "bar"]);
    }

    #[test]
    fn from_args_drops_empty_add_tags_and_lowercases_show_items() {
        let args = Args {
            file: None,
            logcat_levels: "I".to_string(),
            tags: "foo".to_string(),
            add_tag: vec!["".to_string(), " Bar ".to_string()],
            no_tag_filter: false,
            serial: None,
            debug_level: crate::application::cli::VerbosityLevel::None,
            highlighted_items: vec![],
            show_items: vec!["Error".to_string()],
            completions: None,
            version: false,
        };

        let state = FilterState::from_args(&args);

        assert_eq!(state.base_tags, vec!["foo", "Bar"]);
        assert_eq!(state.show_items, vec!["error"]);
    }
}
