use std::fs;
use serde_yaml;

pub fn parse(path: &str) -> Result<serde_yaml::Value, Box<dyn std::error::Error>> {
  let content = fs::read_to_string(path)?;
  parse_content(&content)
}

fn parse_content(input :&str) -> Result<serde_yaml::Value, Box<dyn std::error::Error>> {
  let doc: serde_yaml::Value = serde_yaml::from_str(input)?;
  Ok(doc)
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

    let doc = result.unwrap();
    assert_eq!(doc["page"]["title"], "Hra Go");
    assert_eq!(doc["page"]["crumbs"][0]["text"], "Domů");
    assert_eq!(doc["page"]["sections"][0]["title"], "Go klub Můstek");
    assert_eq!(doc["page"]["sections"][0]["labels"], "CZ. Klub.");
    assert_eq!(doc["page"]["sections"][1]["title"], "Go Magic");
    assert_eq!(doc["page"]["sections"][1]["labels"], "ENG. YouTube.");
  }
}
