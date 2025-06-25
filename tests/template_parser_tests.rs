use yet_another_site_generator::template_parser;
use yet_another_site_generator::template_parser::TemplateNode;

#[test]
fn parse_loads_and_parses_template_file() {
  let template_tree = template_parser::parse("tests/data/template-go.html");

  assert!(template_tree.is_ok(), "Expected to parse template file successfully. Error: {:?}", template_tree.err());
  let template_tree = template_tree.unwrap();

  assert_eq!(template_tree.root, TemplateNode::Seq(Vec::new()));
}
