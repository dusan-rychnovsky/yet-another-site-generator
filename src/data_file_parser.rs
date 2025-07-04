use crate::expressions::Path;
use std::fs;
use serde_yaml;

pub struct DataSet {
  pub data: serde_yaml::Value,
}

pub fn parse(path: &str) -> Result<DataSet, Box<dyn std::error::Error>> {
  let content = fs::read_to_string(path)?;
  parse_content(&content)
}

pub fn parse_content(input :&str) -> Result<DataSet, Box<dyn std::error::Error>> {
  let value: serde_yaml::Value = serde_yaml::from_str(input)?;
  Ok(DataSet { data: value })
}

impl DataSet {
  pub fn get_value(&self, path: &Path) -> Result<&str, String> {
    let value = Self::locate(&self, path);
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

  pub fn list(&self, context: &str, path: &Path) -> Result<Vec<DataSet>, String> {
    let value = Self::locate(&self, path);
    match value {
      Some(value) => {
        match value.as_sequence() {
          Some(seq) => Ok(seq.iter().map(|v| Self::push(context, v)).collect()),
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
    Self::locate(&self, path).is_some()
  }

  fn locate(&self, path: &Path) -> Option<&serde_yaml::Value> {
    path.segments.iter().fold(Some(&self.data), |acc, segment| {
      acc.and_then(|v| v.get(segment))
    })
  }

  fn push(str: &str, value: &serde_yaml::Value) -> DataSet {
    DataSet {
      data: serde_yaml::Value::Mapping(
        serde_yaml::Mapping::from_iter(vec![
          (serde_yaml::Value::String(str.to_string()), value.clone()), // TODO: can I get rid of clones?
        ])
      )
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
    let result = parse_content(content);
    assert!(result.is_ok(), "Expected to parse content successfully. Error: {:?}", result.err());

    let doc = result.unwrap().data;
    assert_eq!(doc["page"]["title"], "Hra Go");
    assert_eq!(doc["page"]["crumbs"][0]["text"], "Domů");
    assert_eq!(doc["page"]["sections"][0]["title"], "Go klub Můstek");
    assert_eq!(doc["page"]["sections"][0]["labels"], "CZ. Klub.");
    assert_eq!(doc["page"]["sections"][1]["title"], "Go Magic");
    assert_eq!(doc["page"]["sections"][1]["labels"], "ENG. YouTube.");
  }
}
