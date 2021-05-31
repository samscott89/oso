use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Hash)]
pub enum SourceInfo {
    // From the parser
    Parser {
        /// Index into the source map stored in the knowledge base
        src_id: u64,

        /// Location of the term within the source map
        left: usize,
        right: usize,
    },

    /// Created as a temporary variable
    TemporaryVariable,

    /// From an FFI call
    Ffi,

    /// Created for a test
    Test,
}

impl SourceInfo {
    pub fn ffi() -> Self {
        Self::Ffi
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Source {
    pub filename: Option<String>,
    pub src: String,
}

pub struct Sources {
    /// Map from source ID to `Source`.
    pub sources: HashMap<u64, Source>,
    /// Map from filename to source ID
    pub files: HashMap<String, u64>,
}

impl Default for Sources {
    fn default() -> Self {
        let mut sources = HashMap::new();
        sources.insert(
            0,
            Source {
                filename: None,
                src: "<Unknown>".to_string(),
            },
        );
        Self {
            sources,
            files: Default::default(),
        }
    }
}

impl Sources {
    pub fn add_source(&mut self, source: Source, id: u64) {
        if let Some(ref f) = source.filename {
            self.files.insert(f.clone(), id);
        }
        self.sources.insert(id, source);
    }

    pub fn get_source(&self, src_id: u64) -> Option<Source> {
        self.sources.get(&src_id).cloned()
    }
}
