use skim::prelude::*;

pub struct CustomItem {
    /// Group of item. The smaller the better score.
    pub group: u16,

    /// Set to true to always show this entry in selector.
    pub persist: bool,

    /// The actual string that is provided.
    pub inner: String,
}

impl SkimItem for CustomItem {
    fn text(&self) -> Cow<str> {
        Cow::Borrowed(&self.inner)
    }
}

struct CustomEngine {
    inner: Box<dyn MatchEngine>,
}

impl MatchEngine for CustomEngine {
    fn match_item(&self, item: Arc<dyn SkimItem>) -> Option<MatchResult> {
        // If item is set as persistent, don't need to score them.
        if let Some(item) = (*item).as_any().downcast_ref::<CustomItem>() {
            if item.persist {
                return Some(MatchResult {
                    rank: [item.group as i32, 0, 0, 0],
                    matched_range: MatchRange::Chars(Vec::new()),
                });
            }
        }

        // Score text, then apply the group score.
        let mut result = self.inner.match_item(item.clone());
        if let Some(item) = (*item).as_any().downcast_ref::<CustomItem>() {
            if let Some(result) = &mut result {
                result.rank[0] += item.group as i32;
            }
        }

        result
    }
}

impl std::fmt::Display for CustomEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(CustomEngine: {})", self.inner)
    }
}

pub struct CustomEngineFactory {
    inner: Box<dyn MatchEngineFactory>,
}

impl CustomEngineFactory {
    pub fn new() -> Self {
        let fuzzy_engine_factory = ExactOrFuzzyEngineFactory::builder().build();
        Self {
            inner: Box::new(AndOrEngineFactory::new(fuzzy_engine_factory)),
        }
    }
}

impl MatchEngineFactory for CustomEngineFactory {
    fn create_engine_with_case(&self, query: &str, case: CaseMatching) -> Box<dyn MatchEngine> {
        let engine = self.inner.create_engine_with_case(query, case);
        Box::new(CustomEngine { inner: engine })
    }
}
