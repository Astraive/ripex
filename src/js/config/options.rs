use crate::js::syntax::SyntaxFeatures;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceType {
    Script,
    Module,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EcmaVersion {
    Es5,
    Es2015,
    Es2016,
    Es2017,
    Es2018,
    Es2019,
    Es2020,
    Es2021,
    Es2022,
    Es2023,
    Es2024,
    Es2025,
    Latest,
}

impl EcmaVersion {
    pub fn latest() -> Self {
        EcmaVersion::Latest
    }
}

#[derive(Debug, Clone)]
pub struct ParserOptions {
    pub source_type: SourceType,
    pub features: SyntaxFeatures,
    pub capture_comments: bool,
    pub ecma_version: EcmaVersion,
}

impl Default for ParserOptions {
    fn default() -> Self {
        ParserOptions {
            source_type: SourceType::Script,
            features: SyntaxFeatures::new(),
            capture_comments: true,
            ecma_version: EcmaVersion::latest(),
        }
    }
}

impl ParserOptions {
    pub fn module() -> Self {
        ParserOptions {
            source_type: SourceType::Module,
            ..Default::default()
        }
    }

    pub fn script() -> Self {
        ParserOptions::default()
    }

    pub fn is_module(&self) -> bool {
        self.source_type == SourceType::Module
    }
}
