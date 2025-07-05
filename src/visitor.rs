use crate::template_parser::{TemplateTree, TemplateNode, TemplateNode::*};
use crate::data_file_parser::DataSet;
use crate::expressions::Predicate::Exists;

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
      data.get_str(path).map(String::from)
    },
    Text(text) => {
      Ok(text.to_string())
    },
    ForEach(var, path, body) => {
      let items = data.list(var, path)?;
      let mut output = String::new();
      for item in items {
        let str = visit_node(body, &item)?;
        output.push_str(&str);
      }
      Ok(output)
    },
    If(expr, body) => {
      match expr.predicate {
        Exists => {
          if data.exists(&expr.path) {
            visit_node(body, data)
          } else {
            Ok(String::new())
          }
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_yaml::{Value, Mapping};
  use crate::expressions::{Path, Expr};

  #[test]
  fn visit_simple_text() {
    let data = DataSet::from(&Value::Null);
    let tree = TemplateTree {
      root: Text("Hello, world!"),
    };
    let result = unwrap(visit(&tree, &data));
    assert_eq!("Hello, world!", result);
  }

  #[test]
  fn visit_var_with_simple_path() {
    let data = Value::Mapping(
      Mapping::from_iter(vec![
        (Value::String("name".to_string()), Value::String("Julia".to_string())),
      ])
    );
    let data_set = DataSet::from(&data);
    let tree = TemplateTree {
      root: Seq(vec![
        Box::new(Text("Hello, ")),
        Box::new(Var(Path::from(vec!["name"]))),
        Box::new(Text("!")),
      ]),
    };
    let result = unwrap(visit(&tree, &data_set));
    assert_eq!(result, "Hello, Julia!");
  }

  #[test]
  fn visit_var_fails_if_data_entry_doesnt_exist() {
    let data = Value::Mapping(Mapping::new());
    let data_set = DataSet::from(&data);
    let tree = TemplateTree {
      root: Seq(vec![
        Box::new(Text("Hello, ")),
        Box::new(Var(Path::from(vec!["name"]))),
        Box::new(Text("!")),
      ]),
    };
    let err = visit(&tree, &data_set).unwrap_err();
    assert!(err.contains("Path [name] is not defined in data file."), "Got error: {}", err);
  }

  #[test]
  fn visit_var_fails_if_data_entry_isnt_string() {
    let data = Value::Mapping(
      Mapping::from_iter(vec![
        (Value::String("name".to_string()), Value::Mapping(
          Mapping::from_iter(vec![
            (Value::String("first".to_string()), Value::String("Julia".to_string())),
            (Value::String("last".to_string()), Value::String("Doe".to_string())),
          ])
        )),
      ])
    );
    let data_set = DataSet::from(&data);
    let tree = TemplateTree {
      root: Seq(vec![
        Box::new(Text("Hello, ")),
        Box::new(Var(Path::from(vec!["name"]))),
        Box::new(Text("!")),
      ]),
    };
    let err = visit(&tree, &data_set).unwrap_err();
    assert!(err.contains("Path [name] does not reference a string in data file."), "Got error: {}", err);
  }

  #[test]
  fn visit_var_with_multi_segment_path() {
    let data = Value::Mapping(
      Mapping::from_iter(vec![
        (Value::String("section".to_string()), Value::Mapping(
          Mapping::from_iter(vec![
            (Value::String("title".to_string()), Value::String("Go Basics".to_string())),
          ])
        )),
      ])
    );
    let data_set = DataSet::from(&data);
    let tree = TemplateTree {
      root: Seq(vec![
        Box::new(Text("Section title: ")),
        Box::new(Var(Path::from(vec!["section", "title"]))),
        Box::new(Text(".")),
      ]),
    };
    let result = unwrap(visit(&tree, &data_set));
    assert_eq!(result, "Section title: Go Basics.");
  }

  #[test]
  fn visit_foreach() {
    let data = Value::Mapping(
      Mapping::from_iter(vec![
        (Value::String("section".to_string()), Value::Mapping(
          Mapping::from_iter(vec![
            (Value::String("links".to_string()), Value::Sequence(vec![
              Value::Mapping(Mapping::from_iter(vec![
                (Value::String("href".to_string()), Value::String("Go Basics".to_string())),
              ])),
              Value::Mapping(Mapping::from_iter(vec![
                (Value::String("href".to_string()), Value::String("Advanced Go".to_string())),
              ])),
            ])),
          ])
        )),
      ])
    );
    let data_set = DataSet::from(&data);
    let tree = TemplateTree {
      root: Seq(vec![
        Box::new(ForEach(
          "link",
          Path::from(vec!["section", "links"]),
          Box::new(Seq(vec![
            Box::new(Text("- link: ")),
            Box::new(Var(Path::from(vec!["link", "href"]))),
            Box::new(Text("\n")),
          ]))
        )),
      ]),
    };
    let result = unwrap(visit(&tree, &data_set));
    assert_eq!(result, "\
- link: Go Basics
- link: Advanced Go
");
  }

  #[test]
  fn visit_if_exists() {
    let data = Value::Mapping(
      Mapping::from_iter(vec![
        (Value::String("items".to_string()), Value::Mapping(
          Mapping::from_iter(vec![
            (Value::String("amount".to_string()), Value::String("2".to_string())),
          ])
        )),
      ])
    );
    let data_set = DataSet::from(&data);
    let tree = TemplateTree {
      root: If(
        Expr::from(Exists, vec!["items", "amount"]),
        Box::new(Seq(vec![
          Box::new(Text("We have ")),
          Box::new(Var(Path::from(vec!["items", "amount"]))),
          Box::new(Text(" items left."))
        ]))
      ),
    };
    let result = unwrap(visit(&tree, &data_set));
    assert_eq!(result, "We have 2 items left.");
  }

  #[test]
  fn visit_if_not_exists() {
    let data = Value::Mapping(
      Mapping::from_iter(vec![
        (Value::String("items".to_string()), Value::Mapping(
          Mapping::from_iter(vec![
            (Value::String("count".to_string()), Value::String("2".to_string())),
          ])
        )),
      ])
    );
    let data_set = DataSet::from(&data);
    let tree = TemplateTree {
      root: If(
        Expr::from(Exists, vec!["items", "amount"]),
        Box::new(Seq(vec![
          Box::new(Text("We have ")),
          Box::new(Var(Path::from(vec!["items", "amount"]))),
          Box::new(Text(" items left."))
        ]))
      ),
    };
    let result = unwrap(visit(&tree, &data_set));
    assert_eq!(result, "");
  }

  #[test]
  fn visit_if_collection() {
    let data = Value::Mapping(
      Mapping::from_iter(vec![
        (Value::String("section".to_string()), Value::Mapping(
          Mapping::from_iter(vec![
            (Value::String("subsections".to_string()), Value::Sequence(vec![
              Value::Mapping(Mapping::from_iter(vec![
                (Value::String("title".to_string()), Value::String("Subsection 1".to_string())),
              ])),
              Value::Mapping(Mapping::from_iter(vec![
                (Value::String("title".to_string()), Value::String("Subsection 2".to_string())),
              ])),
            ])),
          ])
        )),
      ])
    );
    let data_set = DataSet::from(&data);
    let tree = TemplateTree {
      root: If(
        Expr::from(Exists, vec!["section", "subsections"]),
        Box::new(Seq(vec![
          Box::new(Text("Subsections:\n")),
          Box::new(ForEach(
            "subsection",
            Path::from(vec!["section", "subsections"]),
            Box::new(Seq(vec![
              Box::new(Text("- ")),
              Box::new(Var(Path::from(vec!["subsection", "title"]))),
              Box::new(Text("\n")),
            ]))
          )),
        ]))
      ),
    };
    let result = unwrap(visit(&tree, &data_set));
    assert_eq!(result, "\
Subsections:
- Subsection 1
- Subsection 2
");
  }

  fn unwrap(result: Result<String, String>) -> String {
    assert!(result.is_ok(), "Error visiting NodeTree: {:?}", result.err());
    result.unwrap()
  }
}
