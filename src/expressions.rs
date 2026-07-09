/// Represents a path in a tree and is used for lookups into the yaml [`crate::data_file_parser::DataSet`].
#[derive(Debug, PartialEq, Clone)]
pub struct Path {
    pub segments: Vec<String>,
}

/// Represents a boolean expression, which are used as conditions in templates.
#[derive(Debug, PartialEq, Clone)]
pub struct Expr {
    pub predicate: Predicate,
    pub path: Path,
}

/// Represents a predicate of a boolean expression.
#[derive(Debug, PartialEq, Clone)]
pub enum Predicate {
    Exists,
}

impl Path {
    /// Parses a given string into a [`Path`]. Segments are separated by dots.
    pub fn parse(input: &str) -> Self {
        Path {
            segments: input.split('.').map(|s| s.to_string()).collect(),
        }
    }

    /// Helper function to instantiate a [`Path`] from a vector of segments.
    pub fn from_segments(segments: Vec<&str>) -> Self {
        Path {
            segments: segments.into_iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Helper function to instantiate a [`Path`] from a single segment.
    pub fn from_segment(segment: &str) -> Self {
        Self::from_segments(vec![segment])
    }
}

impl Expr {
    /// Parses the given strings into an [`Expr`].
    pub fn parse(parts: Vec<&str>) -> Result<Self, String> {
        if parts.len() != 2 {
            return Err(format!(
                "Invalid expression syntax - expected a predicate and a path, got: '{:#?}'.",
                parts
            ));
        }
        let predicate = match parts[0] {
            "EXISTS" => Predicate::Exists,
            _ => return Err(format!("Unknown predicate: '{}'.", parts[0])),
        };
        let path = Path::parse(parts[1]);
        Ok(Expr { predicate, path })
    }

    /// Helper function to instantiate an [`Expr`] from a predicate and path segments.
    pub fn from(predicate: Predicate, segments: Vec<&str>) -> Self {
        Expr {
            predicate,
            path: Path::from_segments(segments),
        }
    }
}
