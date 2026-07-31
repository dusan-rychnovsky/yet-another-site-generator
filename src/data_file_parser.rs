use crate::expressions::Path;
use std::collections::HashMap;
use std::rc::Rc;

/// Represents a node in the data tree.
#[derive(Debug, Clone)]
pub enum Node {
    /// A scalar string value.
    Str(String),
    /// An ordered sequence of nodes.
    Seq(Vec<Rc<Node>>),
    /// A mapping of keys to nodes.
    Map(HashMap<String, Rc<Node>>),
    /// A value that is present but is not a string (e.g. number, bool, null).
    Other,
}

/// Parses the given YAML file content into a [`Node`] tree.
pub fn parse(input: &str) -> Result<Rc<Node>, serde_yaml::Error> {
    Ok(Node::from_yaml(&serde_yaml::from_str(input)?))
}

impl Node {
    /// Converts a Serde YAML value into a [`Node`].
    pub fn from_yaml(value: &serde_yaml::Value) -> Rc<Self> {
        Rc::new(match value {
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
        })
    }

    pub fn from_yaml_text(text: &str) -> Result<Rc<Self>, String> {
        serde_yaml::from_str(text)
            .map_err(|e| format!("Failed to parse YAML: {}", e))
            .map(|value| Node::from_yaml(&value))
    }

    pub fn get_map_child(&self, key: &str) -> Option<&Node> {
        match self {
            Node::Map(map) => map.get(key).map(Rc::as_ref),
            _ => None,
        }
    }

    pub fn get_seq(&self) -> Option<&[Rc<Node>]> {
        match self {
            Node::Seq(seq) => Some(seq.as_slice()),
            _ => None,
        }
    }

    pub fn get_text(&self) -> Option<&str> {
        match self {
            Node::Str(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
struct Scope {
    /// Context is used to offset paths in the represented tree, in cases when
    /// the parent [`DataSet`] is referenced in the context of a variable in the template,
    /// such as within foreach loops. Otherwise, it is an empty string.
    context: String,
    root: Rc<Node>,
}

impl Scope {
    /// Locates a node in the represented tree by the given path.
    fn locate(&self, path: &Path) -> Option<&Node> {
        let segments = if self.context.is_empty() {
            path.segments.as_slice()
        } else if path.segments.first() == Some(&self.context) {
            &path.segments[1..]
        } else {
            return None;
        };

        segments
            .iter()
            .try_fold(self.root.as_ref(), |node, segment| {
                node.get_map_child(segment)
            })
    }
}

/// Represents a dataset backed by a [`Node`] tree.
#[derive(Debug)]
pub struct DataSet {
    scopes: Vec<Scope>,
}

impl DataSet {
    /// Creates a new [`DataSet`] with an empty context.
    pub fn from_tree(root: Rc<Node>) -> Self {
        DataSet {
            scopes: vec![Scope {
                context: String::new(),
                root,
            }],
        }
    }

    pub fn push(&self, context: &str, root: &Rc<Node>) -> DataSet {
        let mut scopes = self.scopes.to_vec();
        scopes.push(Scope {
            context: context.to_string(),
            root: Rc::clone(root),
        });
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
    pub fn list(&self, context: &str, path: &Path) -> Result<Vec<DataSet>, String> {
        let value = self.locate(path);
        match value {
            Some(Node::Seq(seq)) => Ok(seq.iter().map(|node| self.push(context, node)).collect()),
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
        let data_set = DataSet::from_tree(doc);

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
