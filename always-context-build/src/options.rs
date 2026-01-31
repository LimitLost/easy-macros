#[derive(Debug, Default, Clone)]
pub struct BuildOptions {
    skip: Vec<Skip>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Skip {
    Macros,
    MacrosShort,
    EasySql,
    Es,
}

impl BuildOptions {
    /// Create a new `BuildOptions` with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the skip options for generated `#[always_context]` attributes
    pub fn skip(mut self, skips: Vec<Skip>) -> Self {
        self.skip = skips;
        self
    }

    /// Generate the attribute string based on configured options
    /// Returns the content that goes inside `#[always_context(...)]`
    pub(crate) fn generate_attribute_options(&self) -> String {
        if self.skip.is_empty() {
            return String::new();
        }

        let mut options = Vec::new();

        // Convert Skip variants to their string representation
        for skip in &self.skip {
            let option_str = match skip {
                Skip::Macros => "macros",
                Skip::MacrosShort => "!",
                Skip::EasySql => "easy_sql",
                Skip::Es => "es",
            };
            options.push(option_str);
        }

        format!("skip({})", options.join(", "))
    }
}
