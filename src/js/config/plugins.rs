use super::options::ParserOptions;

#[derive(Debug, Clone, Default)]
pub struct ParserPlugins {
    pub jsx: bool,
    pub typescript: bool,
    pub decorators: bool,
    pub import_attributes: bool,
}

impl ParserPlugins {
    pub fn apply(&self, options: &mut ParserOptions) {
        let mut features = options.features;
        features.jsx |= self.jsx;
        features.typescript |= self.typescript;
        features.decorators |= self.decorators;
        features.import_attributes |= self.import_attributes;
        options.features = features;
    }

    pub fn all_js() -> Self {
        ParserPlugins {
            jsx: true,
            ..Default::default()
        }
    }

    pub fn all_ts() -> Self {
        ParserPlugins {
            jsx: true,
            typescript: true,
            decorators: true,
            ..Default::default()
        }
    }

    pub fn typescript() -> Self {
        ParserPlugins {
            typescript: true,
            decorators: true,
            ..Default::default()
        }
    }
}
