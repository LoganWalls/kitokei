use anyhow::Result;
use std::path::Path;

macro_rules! gated_match {
    ($input:ident, $( ($lang:ident, $feature:expr, $body:expr) ),* $(,)?)  => {
        match $input {
            $(
                #[cfg(feature = $feature)]
                Self::$lang => Ok(($body)),
            )*
            #[cfg(not(feature = "all-languages"))]
            lang => {
                Err(anyhow::anyhow!(
                    "This copy of kitokei was not built with '{}' support",
                    lang.name()
                ))
            },
        }
    };
}

macro_rules! queries_for {
    ($lang:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/external/zed/queries/",
            $lang,
            "/highlights.scm"
        ))
    };
}

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
        gated_match!(
            self,
            (Python, "python", tree_sitter_python::language()),
            (Rust, "rust", tree_sitter_rust::language()),
        )
    }

    pub fn queries(&self) -> Result<&'static str> {
        gated_match!(
            self,
            (Python, "python", queries_for!("python")),
            (Rust, "rust", queries_for!("rust")),
        )
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
