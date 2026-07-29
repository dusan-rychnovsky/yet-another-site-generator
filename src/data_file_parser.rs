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

#[derive(Debug, Clone)]
struct Scope<'a> {
    /// Context is used to offset paths in the represented tree, in cases when
    /// the parent [`DataSet`] is referenced in the context of a variable in the template,
    /// such as within foreach loops. Otherwise, it is an empty string.
    context: &'a str,
    root: &'a Node,
}

impl<'a> Scope<'a> {
    /// Locates a node in the represented tree by the given path.
    fn locate(&self, path: &Path) -> Option<&'a Node> {
        // offset path by dataset context, if exists
        if !self.context.is_empty() {
            if !path.segments.is_empty() && self.context == path.segments[0] {
                let new_scope = Scope {
                    context: "",
                    root: self.root,
                };
                let new_path = Path {
                    segments: path.segments[1..].to_vec(),
                };
                new_scope.locate(&new_path)
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

/// Represents a dataset backed by a [`Node`] tree.
#[derive(Debug)]
pub struct DataSet<'a> {
    scopes: Vec<Scope<'a>>,
}

impl<'a> DataSet<'a> {
    /// Creates a new [`DataSet`] with empty [`DataSet::context`].
    pub fn from(root: &'a Node) -> Self {
        DataSet {
            scopes: vec![Scope { context: "", root }],
        }
    }

    pub fn push(&self, context: &'a str, root: &'a Node) -> DataSet<'a> {
        let mut scopes = self.scopes.to_vec();
        scopes.push(Scope { context, root });
        DataSet { scopes }
    }

    /// Gets a string value from the represented tree at the given path.
    /// Returns an error if the path is not defined in the tree
    /// or if it does not reference a string.
    pub fn get_str(&self, path: &Path) -> Result<&str, String> {
        let value = Self::locate(self, path);
        match value {
            Some(Node::Str(value)) => Ok(value.as_str()),
            Some(_) => Err(format!(
                "Path [{}] does not reference a string in data file.",
                path.segments.join(".")
            )),
            None => Err(format!(
                "Path [{}] is not defined in data file.",
                path.segments.join(".")
            )),
        }
    }

    /// Lists all child [`DataSet`]s which are located at the given path in the represented tree.
    /// Returns an error if the path is not defined in the tree
    /// or if it does not reference a sequence.
    ///
    /// # Arguments
    /// * `context` - context of the child datasets.
    pub fn list<'b>(&'b self, context: &'b str, path: &Path) -> Result<Vec<DataSet<'b>>, String> {
        let value = self.locate(path);
        match value {
            Some(Node::Seq(seq)) => Ok(seq.iter().map(|v| self.push(context, v)).collect()),
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

    fn locate(&self, path: &Path) -> Option<&Node> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.locate(path))
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
            "Hra Go"
        );

        let crumbs = data_set.list("", &Path::parse("page.crumbs")).unwrap();
        assert_eq!(crumbs[0].get_str(&Path::parse("text")).unwrap(), "Domů");

        let sections = data_set.list("", &Path::parse("page.sections")).unwrap();
        assert_eq!(
            sections[0].get_str(&Path::parse("title")).unwrap(),
            "Go klub Můstek"
        );
        assert_eq!(
            sections[0].get_str(&Path::parse("labels")).unwrap(),
            "CZ. Klub."
        );
        assert_eq!(
            sections[1].get_str(&Path::parse("title")).unwrap(),
            "Go Magic"
        );
        assert_eq!(
            sections[1].get_str(&Path::parse("labels")).unwrap(),
            "ENG. YouTube."
        );
    }
}
