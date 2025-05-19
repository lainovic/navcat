use std::collections::HashSet;

#[derive(Debug)]
pub struct FilterConfig {
    pub levels: Vec<&'static str>,
    pub tags: TagCategories,
    pub blacklisted_items: Vec<&'static str>,
}

#[derive(Debug, Clone)]
pub struct TagCategories {
    pub top_classes: Vec<String>,
    pub steps: Vec<String>,
    pub engines: Vec<String>,
    all_tags: HashSet<String>,
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
        self.all_tags.contains(tag)
    }
}

impl FilterConfig {
    pub fn parse(
        levels_str: &str,
        tags_str: &str,
        include_guidance: bool,
        include_routing: bool,
    ) -> Self {
        let levels = Self::to_levels(levels_str);
        let tags = Self::to_tags(tags_str);

        let mut blacklisted_items = Vec::new();
        if !include_guidance {
            blacklisted_items.push("guidance");
            blacklisted_items.push("instruction");
            blacklisted_items.push("warning");
        }

        if !include_routing {
            blacklisted_items.push("router");
        }

        Self {
            levels,
            tags: TagCategories::new(tags),
            blacklisted_items,
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
