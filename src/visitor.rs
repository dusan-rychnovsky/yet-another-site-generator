use crate::template_parser::{TemplateTree, TemplateNode, TemplateNode::*};
use crate::expressions::Path;
use serde_yaml;

pub fn visit(tree: &TemplateTree, data: &serde_yaml::Value) -> Result<String, String> {
  visit_node(&tree.root, data)
}

fn visit_node(node: &TemplateNode, data: &serde_yaml::Value) -> Result<String, String> {
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
      match data.get(path.segments[0]) {
        Some(value) => {
          match value.as_str() {
            Some(s) => Ok(s.to_string()),
            None => Err(
              format!("Var [{}] is not a string.", path.segments.join("."))
            ),
          }
        },
        None => Err(
          format!("Var [{}] is not defined.", path.segments.join("."))
        ),
      }
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
    let data = serde_yaml::Value::Null;
    let tree = TemplateTree {
      root: Text("Hello, world!"),
    };
    let result = unwrap(visit(&tree, &data));
    assert_eq!("Hello, world!", result);
  }

  #[test]
  fn visit_var_with_simple_path() {
    let data = serde_yaml::Value::Mapping(
      serde_yaml::Mapping::from_iter(vec![
        (serde_yaml::Value::String("name".to_string()), serde_yaml::Value::String("Julia".to_string())),
      ])
    );
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
    let data = serde_yaml::Value::Mapping(serde_yaml::Mapping::new());
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
    let data = serde_yaml::Value::Mapping(
      serde_yaml::Mapping::from_iter(vec![
        (serde_yaml::Value::String("name".to_string()), serde_yaml::Value::Mapping(
          serde_yaml::Mapping::from_iter(vec![
            (serde_yaml::Value::String("first".to_string()), serde_yaml::Value::String("Julia".to_string())),
            (serde_yaml::Value::String("last".to_string()), serde_yaml::Value::String("Doe".to_string())),
          ])
        )),
      ])
    );
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

  fn unwrap(result: Result<String, String>) -> String {
    assert!(result.is_ok(), "Error visiting NodeTree: {:?}", result.err());
    result.unwrap()
  }
}
