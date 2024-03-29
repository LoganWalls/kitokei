use anyhow::Result;
use std::path::Path;

pub enum Language {
    Python,
    Rust,
}

impl Language {
    pub fn name(&self) -> &str {
        match self {
            Self::Python => "python",
            Self::Rust => "rust",
        }
    }

    pub fn tree_sitter_language(&self) -> Result<tree_sitter::Language> {
        match self {
            #[cfg(feature = "python")]
            Self::Python => Ok(tree_sitter_python::language()),
            #[cfg(feature = "rust")]
            Self::Rust => Ok(tree_sitter_rust::language()),
            #[cfg(not(feature = "all-languages"))]
            lang => {
                Err(anyhow::anyhow!(
                    "Feature flag for '{}' not enabled",
                    lang.name()
                ))
            }
        }
    }

    pub fn queries(&self) -> Result<&'static str> {
        match self {
            #[cfg(feature = "python")]
            Self::Python => {
                Ok(include_str!("../external/nvim-treesitter/queries/python/highlights.scm"))
            }
            #[cfg(feature = "rust")]
            Self::Rust => {
                Ok(include_str!("../external/nvim-treesitter/queries/rust/highlights.scm"))
            }
            #[cfg(not(feature = "all-languages"))]
            lang => {
                Err(anyhow::anyhow!(
                    "Feature flag for '{}' not enabled",
                    lang.name()
                ))
            }
        }
    }
}

impl TryFrom<&Path> for Language {
    type Error = anyhow::Error;
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let extension = path
            .extension()
            .ok_or_else(|| anyhow::anyhow!("{path:?} has no file extension"))?
            .to_str()
            .expect("str compatible extension");
        match extension {
            "py" => Ok(Self::Python),
            "rs" => Ok(Self::Rust),
            _ => Err(anyhow::anyhow!("Unsupported extension: {}", extension)),
        }
    }
}
