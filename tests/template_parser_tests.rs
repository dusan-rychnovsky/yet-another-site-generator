use yet_another_site_generator::template_parser;
use yet_another_site_generator::template_parser::TemplateNode;
use std::fs;

#[test]
#[ignore]
fn parse_loads_and_parses_template_file() {
  let content = fs::read_to_string("tests/data/go-template.html")
    .unwrap_or_else(|e| panic!("Failed to read template file: {}", e));
  let template_tree = template_parser::parse(&content);

  assert!(template_tree.is_ok(), "Expected to parse template file successfully. Error: {:?}", template_tree.err());
  let template_tree = template_tree.unwrap();

  assert_eq!(template_tree.root, TemplateNode::Seq(Vec::new()));
}
