use yasg::template_parser;
use yasg::template_parser::TemplateNode;
use yasg::expressions::{Path, Predicate::*};
use std::fs;

#[test]
fn parse_loads_and_parses_template_file() {
  let content = fs::read_to_string("tests/data/example-template.html")
    .unwrap_or_else(|e| panic!("Failed to read template file: '{}'.", e));
  let tokens = yasg::template_tokenizer::tokenize(&content)
    .unwrap_or_else(|e| panic!("Failed to tokenize template file: '{}'.", e));
  let template = template_parser::parse(&tokens)
    .unwrap_or_else(|e| panic!("Failed to parse template file: '{}'.", e));

  if let TemplateNode::Seq(nodes) = &template.root {
    assert_eq!(nodes.len(), 7);
    assert_eq!(nodes[0], Box::new(TemplateNode::Text("<!DOCTYPE html>\n<html lang=\"en\">\n  <head>\n    <title>")));
    assert_eq!(nodes[1], Box::new(TemplateNode::Var(Path::from_segment("title"))));
    assert_eq!(nodes[2], Box::new(TemplateNode::Text("</title>\n  </head>\n  <body>\n    <h1>")));
    assert_eq!(nodes[3], Box::new(TemplateNode::Var(Path::from_segment("title"))));
    assert_eq!(nodes[4], Box::new(TemplateNode::Text("</h1>\n    <p>This is a testing page.</p>\n    ")));

    match nodes[5].as_ref() {
      TemplateNode::If(condition, body) => {
        assert_eq!(condition.predicate, Exists);
        assert_eq!(condition.path.segments, vec!["backpack", "items"]);

        match body.as_ref() {
          TemplateNode::Seq(if_nodes) => {
            assert_eq!(if_nodes.len(), 3);
            assert_eq!(if_nodes[0], Box::new(TemplateNode::Text("\n      <h2>Items in Backpack:</h2>\n      <ul>\n        ")));

            match if_nodes[1].as_ref() {
              TemplateNode::ForEach(var, path, for_body) => {
                assert_eq!(var, &"item");
                assert_eq!(path.segments, vec!["backpack", "items"]);

                match for_body.as_ref() {
                  TemplateNode::Seq(for_nodes) => {
                    assert_eq!(for_nodes.len(), 5);
                    assert_eq!(for_nodes[0], Box::new(TemplateNode::Text("\n          <li>")));
                    assert_eq!(for_nodes[1], Box::new(TemplateNode::Var(Path::from_segments(vec!["item", "name"]))));
                    assert_eq!(for_nodes[2], Box::new(TemplateNode::Text(" - weight: ")));
                    assert_eq!(for_nodes[3], Box::new(TemplateNode::Var(Path::from_segments(vec!["item", "weight"]))));
                    assert_eq!(for_nodes[4], Box::new(TemplateNode::Text("</li>\n        ")));
                  }
                  _ => {
                    panic!("Expected foreach body to be a Seq node.");
                  }
                }
              }
              _ => {
                panic!("Expected nodes[1] inside if to be a foreach node.");
              }
            }

            assert_eq!(if_nodes[2], Box::new(TemplateNode::Text("\n      </ul>\n    ")));
          }
          _ => {
            panic!("Expected if body to be a Seq node.");
          }
        }
      }
      _ => {
        panic!("Expected nodes[5] to be an If node.");
      }
    }

    assert_eq!(nodes[6], Box::new(TemplateNode::Text("\n  </body>\n</html>\n")));
  }
  else {
    panic!("Expected template root to be a Seq node.");
  }
}
