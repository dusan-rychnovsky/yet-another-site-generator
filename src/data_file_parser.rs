use crate::expressions::Path;
use std::collections::HashMap;

/// Represents a node in the data tree.
#[derive(Debug, Clone)]
pub enum Node {
    /// A scalar string value.
    Str(String),
    /// An ordered sequence of nodes.
    Seq(Vec<Node>),
    /// A mapping of keys to nodes.
    Map(HashMap<String, Node>),
    /// A value that is present but is not a string (e.g. number, bool, null).
    Other,
}

/// Represents a dataset backed by a [`Node`] tree.
#[derive(Debug)]
pub struct DataSet<'a> {
    /// Context is used to offset paths in the represented tree, in cases when
    /// the [`DataSet`] is referenced in the context of a variable in the template,
    /// such as within foreach loops. Otherwise, it is an empty string.
    pub context: &'a str,
    /// Root node of the data tree.
    pub root: &'a Node,
}

/// Parses the given yaml content into a yaml tree, which can then be converted into a [`Node`] tree using [`Node::from_yaml`].
pub fn parse(input: &str) -> Result<Node, serde_yaml::Error> {
    Ok(Node::from_yaml(&serde_yaml::from_str(input)?))
}

impl Node {
    /// Converts a borrowed yaml value into a [`Node`].
    pub fn from_yaml(value: &serde_yaml::Value) -> Self {
        match value {
            serde_yaml::Value::String(s) => Node::Str(s.clone()),
            serde_yaml::Value::Sequence(seq) => {
                Node::Seq(seq.iter().map(Node::from_yaml).collect())
            }
            serde_yaml::Value::Mapping(map) => Node::Map(
                map.iter()
                    .filter_map(|(k, v)| k.as_str().map(|k| (k.to_string(), Node::from_yaml(v))))
                    .collect(),
            ),
            _ => Node::Other,
        }
    }
}

impl<'a> DataSet<'a> {
    /// Creates a new [`DataSet`] with empty [`DataSet::context`].
    pub fn from(root: &'a Node) -> Self {
        DataSet { context: "", root }
    }

    /// Gets a string value from the represented tree at the given path.
    /// Returns an error if the path is not defined in the tree
    /// or if it does not reference a string.
    pub fn get_str(&self, path: &Path) -> Result<Option<&str>, String> {
        let value = Self::locate(self, path);
        match value {
            Some(Node::Str(value)) => Ok(Some(value.as_str())),
            Some(_) => Err(format!(
                "Path [{}] does not reference a string in data file.",
                path.segments.join(".")
            )),
            None => Ok(None),
        }
    }

    /// Lists all child [`DataSet`]s which are located at the given path in the represented tree.
    /// Returns an error if the path is not defined in the tree
    /// or if it does not reference a sequence.
    ///
    /// # Arguments
    /// * `context` - context of the child datasets.
    pub fn list(&self, context: &'a str, path: &Path) -> Result<Vec<DataSet<'a>>, String> {
        let value = self.locate(path);
        match value {
            Some(Node::Seq(seq)) => Ok(seq.iter().map(|v| DataSet { context, root: v }).collect()),
            Some(_) => Err(format!(
                "Path [{}] does not reference a sequence in data file.",
                path.segments.join(".")
            )),
            None => Err(format!(
                "Path [{}] is not defined in data file.",
                path.segments.join(".")
            )),
        }
    }

    /// Checks if a node exists in the represented tree at the given path.
    pub fn exists(&self, path: &Path) -> bool {
        Self::locate(self, path).is_some()
    }

    /// Locates a node in the represented tree by the given path.
    fn locate(&self, path: &Path) -> Option<&'a Node> {
        // offset path by dataset context, if exists
        if !self.context.is_empty() {
            if !path.segments.is_empty() && self.context == path.segments[0] {
                let new_dataset = DataSet {
                    context: "",
                    root: self.root,
                };
                let new_path = Path {
                    segments: path.segments[1..].to_vec(),
                };
                new_dataset.locate(&new_path)
            } else {
                None
            }
        } else {
            path.segments
                .iter()
                .try_fold(self.root, |acc, segment| match acc {
                    Node::Map(map) => map.get(segment.as_str()),
                    _ => None,
                })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_content_handles_simple_data_file() {
        let content = "\
page:
  title: Hra Go
  crumbs:
    - href: \"/\"
      text: Domů
    - text: Zdroje
    - text: Go
  sections:
    - title: Go klub Můstek
      labels: CZ. Klub.
    - title: Go Magic
      labels: ENG. YouTube.
";

        let result = parse(content);
        assert!(
            result.is_ok(),
            "Expected to parse content successfully. Error: {:?}",
            result.err()
        );

        let doc = result.unwrap();
        let data_set = DataSet::from(&doc);

        assert_eq!(
            data_set.get_str(&Path::parse("page.title")).unwrap(),
            Some("Hra Go")
        );

        let crumbs = data_set.list("", &Path::parse("page.crumbs")).unwrap();
        assert_eq!(
            crumbs[0].get_str(&Path::parse("text")).unwrap(),
            Some("Domů")
        );

        let sections = data_set.list("", &Path::parse("page.sections")).unwrap();
        assert_eq!(
            sections[0].get_str(&Path::parse("title")).unwrap(),
            Some("Go klub Můstek")
        );
        assert_eq!(
            sections[0].get_str(&Path::parse("labels")).unwrap(),
            Some("CZ. Klub.")
        );
        assert_eq!(
            sections[1].get_str(&Path::parse("title")).unwrap(),
            Some("Go Magic")
        );
        assert_eq!(
            sections[1].get_str(&Path::parse("labels")).unwrap(),
            Some("ENG. YouTube.")
        );
    }
}
