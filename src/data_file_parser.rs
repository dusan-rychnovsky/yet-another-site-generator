use crate::expressions::Path;
use serde_yaml;

#[derive(Debug)]
pub struct DataSet<'a> {
  pub context: &'a str,
  pub root: &'a serde_yaml::Value
}

pub fn parse(input :&str) -> Result<serde_yaml::Value, serde_yaml::Error> {
  serde_yaml::from_str(input)
}

impl<'a> DataSet<'a> {
  pub fn from(root: &'a serde_yaml::Value) -> Self {
    DataSet { context: "", root }
  }

  pub fn get_str(&self, path: &Path) -> Result<&str, String> {
    let value = Self::locate(self, path);
    match value {
      Some(value) => {
        match value.as_str() {
          Some(value) => Ok(value),
          None => Err(
            format!("Path [{}] does not reference a string in data file.", path.segments.join("."))
          ),
        }
      },
      None => Err(
        format!("Path [{}] is not defined in data file.", path.segments.join("."))
      ),
    }
  }

  pub fn list(&self, context: &'a str, path: &Path) -> Result<Vec<DataSet<'a>>, String> {
    let value = self.locate(path);
    match value {
      Some(value) => {
        match value.as_sequence() {
          Some(seq) => Ok(
            seq.iter()
              .map(|v| DataSet {
                context,
                root: v,
              })
              .collect()
          ),
          None => Err(
            format!("Path [{}] does not reference a sequence in data file.", path.segments.join("."))
          ),
        }
      },
      None => Err(
        format!("Path [{}] is not defined in data file.", path.segments.join("."))
      ),
    }
  }

  pub fn exists(&self, path: &Path) -> bool {
    Self::locate(self, path).is_some()
  }

  fn locate(&self, path: &Path) -> Option<&'a serde_yaml::Value> {
    if !self.context.is_empty() {
      if !path.segments.is_empty() && self.context == path.segments[0] {
        let new_dataset = DataSet {
          context: "",
          root: self.root
        };
        let new_path = Path {
          segments: path.segments[1..].to_vec()
        };
        new_dataset.locate(&new_path)
      }
      else {
        None
      }
    }
    else {
      path.segments.iter().try_fold(self.root, |acc, segment| {
        acc.get(segment)
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
    assert!(result.is_ok(), "Expected to parse content successfully. Error: {:?}", result.err());

    let doc = result.unwrap();
    assert_eq!(doc["page"]["title"], "Hra Go");
    assert_eq!(doc["page"]["crumbs"][0]["text"], "Domů");
    assert_eq!(doc["page"]["sections"][0]["title"], "Go klub Můstek");
    assert_eq!(doc["page"]["sections"][0]["labels"], "CZ. Klub.");
    assert_eq!(doc["page"]["sections"][1]["title"], "Go Magic");
    assert_eq!(doc["page"]["sections"][1]["labels"], "ENG. YouTube.");
  }
}
