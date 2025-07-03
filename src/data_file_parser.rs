use std::fs;
use serde_yaml;

pub struct DataSet {
  pub data: serde_yaml::Value,
}

pub fn parse(path: &str) -> Result<DataSet, Box<dyn std::error::Error>> {
  let content = fs::read_to_string(path)?;
  parse_content(&content)
}

fn parse_content(input :&str) -> Result<DataSet, Box<dyn std::error::Error>> {
  let value: serde_yaml::Value = serde_yaml::from_str(input)?;
  Ok(DataSet { data: value })
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
