use std::fs;
use serde::Deserialize;
use serde_yaml;

#[derive (Debug, Deserialize)]
pub struct PageData {
  pub title: String,
  pub crumbs: Vec<CrumbData>,
  pub sections: Vec<SectionData>
}

#[derive (Debug, Deserialize)]
pub struct CrumbData {
  pub text: String,
  pub href: Option<String>
}

#[derive (Debug, Deserialize)]
pub struct SectionData {
  pub title: String,
  pub labels: String,
  pub img: String,
  pub content: Vec<String>,
  pub links: Vec<LinkData>
}

#[derive (Debug, Deserialize)]
pub struct LinkData {
  pub kind: String,
  pub text: String,
  pub href: String
}

pub fn parse(path: &str) -> Result<PageData, Box<dyn std::error::Error>> {
  let content = fs::read_to_string(path)?;
  let doc: serde_yaml::Value = serde_yaml::from_str(&content)?;
  let page_data: PageData = serde_yaml::from_value(doc["page"].clone())?;
  Ok(page_data)
}
