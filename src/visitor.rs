use crate::template_parser::{TemplateTree, TemplateNode, TemplateNode::*};
use crate::data_file_parser::DataSet;
use crate::expressions::Path;
use serde_yaml::{Value, Mapping};

pub fn visit(tree: &TemplateTree, data: &DataSet) -> Result<String, String> {
  visit_node(&tree.root, data)
}

fn visit_node(node: &TemplateNode, data: &DataSet) -> Result<String, String> {
  match node {
    Seq(nodes) => {
      let mut output = String::new();
      for child in nodes {
        let str = visit_node(child, data)?;
        output.push_str(&str);
      }
      Ok(output)
    },
    Var(path) => {
      data.get_value(path).map(String::from)
    },
    Text(text) => {
      Ok(text.to_string())
    },
    _ => Ok(String::from("not implemented yet")),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn visit_simple_text() {
    let data = DataSet { data: Value::Null };
    let tree = TemplateTree {
      root: Text("Hello, world!"),
    };
    let result = unwrap(visit(&tree, &data));
    assert_eq!("Hello, world!", result);
  }

  #[test]
  fn visit_var_with_simple_path() {
    let data = DataSet {
      data: Value::Mapping(
        Mapping::from_iter(vec![
          (Value::String("name".to_string()), Value::String("Julia".to_string())),
        ])
      )
    };
    let tree = TemplateTree {
      root: Seq(vec![
        Box::new(Text("Hello, ")),
        Box::new(Var(Path::from(vec!["name"]))),
        Box::new(Text("!")),
      ]),
    };
    let result = unwrap(visit(&tree, &data));
    assert_eq!(result, "Hello, Julia!");
  }

  #[test]
  fn visit_var_fails_if_data_entry_doesnt_exist() {
    let data = DataSet { data: Value::Mapping(Mapping::new()) };
    let tree = TemplateTree {
      root: Seq(vec![
        Box::new(Text("Hello, ")),
        Box::new(Var(Path::from(vec!["name"]))),
        Box::new(Text("!")),
      ]),
    };
    let err = visit(&tree, &data).unwrap_err();
    assert!(err.contains("Var [name] is not defined."), "Got error: {}", err);
  }

  #[test]
  fn visit_var_fails_if_data_entry_isnt_string() {
    let data = DataSet {
      data: Value::Mapping(
        Mapping::from_iter(vec![
          (Value::String("name".to_string()), Value::Mapping(
            Mapping::from_iter(vec![
              (Value::String("first".to_string()), Value::String("Julia".to_string())),
              (Value::String("last".to_string()), Value::String("Doe".to_string())),
            ])
          )),
        ])
      )
    };
    let tree = TemplateTree {
      root: Seq(vec![
        Box::new(Text("Hello, ")),
        Box::new(Var(Path::from(vec!["name"]))),
        Box::new(Text("!")),
      ]),
    };
    let err = visit(&tree, &data).unwrap_err();
    assert!(err.contains("Var [name] is not a string."), "Got error: {}", err);
  }

  #[test]
  fn visit_var_with_multi_segment_path() {
    let data = DataSet {
      data: Value::Mapping(
        Mapping::from_iter(vec![
          (Value::String("section".to_string()), Value::Mapping(
            Mapping::from_iter(vec![
              (Value::String("title".to_string()), Value::String("Go Basics".to_string())),
            ])
          )),
        ])
      )
    };
    let tree = TemplateTree {
      root: Seq(vec![
        Box::new(Text("Section title: ")),
        Box::new(Var(Path::from(vec!["section", "title"]))),
        Box::new(Text(".")),
      ]),
    };
    let result = unwrap(visit(&tree, &data));
    assert_eq!(result, "Section title: Go Basics.");
  }

  fn unwrap(result: Result<String, String>) -> String {
    assert!(result.is_ok(), "Error visiting NodeTree: {:?}", result.err());
    result.unwrap()
  }
}
