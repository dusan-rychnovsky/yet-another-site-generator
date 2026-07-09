use crate::data_file_parser::DataSet;
use crate::expressions::{Path, Predicate::Exists};
use crate::template_parser::{TemplateNode, TemplateNode::*, TemplateTree};
use std::path::Path as FsPath;

/// Populates the given [`TemplateTree`] with values from the given [`DataSet`].
pub fn visit(tree: &TemplateTree, data: &DataSet) -> Result<String, String> {
    visit_node(&tree.root, &[data])
}

/// Populates the subtree rooted at the given [`TemplateNode`] with values from the given [`DataSet`].
fn visit_node(node: &TemplateNode, scopes: &[&DataSet]) -> Result<String, String> {
    match node {
        Seq(nodes) => {
            let mut output = String::new();
            for child in nodes {
                let str = visit_node(child, scopes)?;
                output.push_str(&str);
            }
            Ok(output)
        }
        Var(path) => {
            let val = get_str(scopes, path)?;
            let val = replace_asterix(val)?;
            Ok(val)
        }
        Func(name, args) => {
            let output = match name.as_str() {
                "LINK" => eval_link(scopes, args),
                _ => Err(format!("Unknown function: '{}'.", name)),
            }?;
            Ok(output)
        }
        Text(text) => Ok(text.to_string()),
        ForEach(var, path, body) => {
            let current_scope = scopes
                .last()
                .ok_or_else(|| "Data stack is empty.".to_string())?;
            let items = current_scope.list(var, path)?;
            let mut output = String::new();
            for item in &items {
                let mut inner = scopes.to_vec();
                inner.push(item);
                let str = visit_node(body, &inner)?;
                output.push_str(&str);
            }
            Ok(output)
        }
        If(expr, body) => match expr.predicate {
            Exists => {
                if exists(scopes, &expr.path) {
                    visit_node(body, scopes)
                } else {
                    Ok(String::new())
                }
            }
        },
    }
}

fn get_str<'a>(scopes: &[&'a DataSet<'_>], path: &Path) -> Result<&'a str, String> {
    scopes
        .iter()
        .rev()
        .find_map(|scope| scope.get_str(path).transpose())
        .transpose()?
        .ok_or_else(|| {
            format!(
                "Path [{}] is not defined in data file.",
                path.segments.join(".")
            )
        })
}

fn exists(scopes: &[&DataSet], path: &Path) -> bool {
    scopes.iter().any(|scope| scope.exists(path))
}

/// Evaluates the `LINK` function. It expects two path arguments - a source and a target file path -
/// and returns a relative link pointing from the source to the target.
fn eval_link(scopes: &[&DataSet], args: &[Path]) -> Result<String, String> {
    if args.len() != 2 {
        return Err(format!(
            "Function 'LINK' expects 2 arguments, got {}.",
            args.len()
        ));
    }
    let source = get_str(scopes, &args[0])?;
    let target = get_str(scopes, &args[1])?;
    Ok(relative_link(source, target))
}

/// Computes a relative link from the `source` file to the `target` file. Both are treated as file
/// paths; the result is expressed relative to the directory containing `source` and always uses `/`
/// as the separator so that it is a valid href regardless of the host platform.
fn relative_link(source: &str, target: &str) -> String {
    let source_dir = FsPath::new(source)
        .parent()
        .unwrap_or_else(|| FsPath::new(""));
    let source_components: Vec<_> = source_dir.components().collect();
    let target_components: Vec<_> = FsPath::new(target).components().collect();

    let common = source_components
        .iter()
        .zip(target_components.iter())
        .take_while(|(a, b)| a == b)
        .count();

    let mut parts: Vec<String> = Vec::new();
    for _ in common..source_components.len() {
        parts.push("..".to_string());
    }
    for component in &target_components[common..] {
        parts.push(component.as_os_str().to_string_lossy().into_owned());
    }
    parts.join("/")
}

/// Replaces asterisks in the given text with HTML <em></em> tags.
fn replace_asterix(text: &str) -> Result<String, &str> {
    let mut result = String::new();
    let mut in_asterix = false;
    for c in text.chars() {
        if c == '*' {
            if !in_asterix {
                result.push_str("<em>");
            } else {
                result.push_str("</em>");
            }
            in_asterix = !in_asterix;
        } else {
            result.push(c);
        }
    }
    if in_asterix {
        Err("Encountered unmatched *.")
    } else {
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_file_parser::Node;
    use crate::expressions::{Expr, Path};
    use serde_yaml::{Mapping, Value};

    #[test]
    fn visit_simple_text() {
        let root = Node::Other;
        let data = DataSet::from(&root);
        let tree = TemplateTree {
            root: Text("Hello, world!".to_string()),
        };
        let result = unwrap(visit(&tree, &data));
        assert_eq!("Hello, world!", result);
    }

    #[test]
    fn visit_var_with_simple_path() {
        let data = Value::Mapping(Mapping::from_iter(vec![(
            Value::String("name".to_string()),
            Value::String("Julia".to_string()),
        )]));
        let root = Node::from_yaml(&data);
        let data_set = DataSet::from(&root);
        let tree = TemplateTree {
            root: Seq(vec![
                Text("Hello, ".to_string()),
                Var(Path::from_segment("name")),
                Text("!".to_string()),
            ]),
        };
        let result = unwrap(visit(&tree, &data_set));
        assert_eq!(result, "Hello, Julia!");
    }

    #[test]
    fn visit_func_link_to_nested_file() {
        let data = Value::Mapping(Mapping::from_iter(vec![
            (
                Value::String("PATH".to_string()),
                Value::String("blog/index.yml".to_string()),
            ),
            (
                Value::String("target".to_string()),
                Value::String("blog/cooking/recipes/oats.yml".to_string()),
            ),
        ]));
        let root = Node::from_yaml(&data);
        let data_set = DataSet::from(&root);
        let tree = TemplateTree {
            root: Func(
                "LINK".to_string(),
                vec![Path::from_segment("PATH"), Path::from_segment("target")],
            ),
        };
        let result = unwrap(visit(&tree, &data_set));
        assert_eq!(result, "cooking/recipes/oats.yml");
    }

    #[test]
    fn visit_func_link_goes_up_directories() {
        let data = Value::Mapping(Mapping::from_iter(vec![
            (
                Value::String("PATH".to_string()),
                Value::String("blog/cooking/recipes/oats.yml".to_string()),
            ),
            (
                Value::String("target".to_string()),
                Value::String("blog/index.yml".to_string()),
            ),
        ]));
        let root = Node::from_yaml(&data);
        let data_set = DataSet::from(&root);
        let tree = TemplateTree {
            root: Func(
                "LINK".to_string(),
                vec![Path::from_segment("PATH"), Path::from_segment("target")],
            ),
        };
        let result = unwrap(visit(&tree, &data_set));
        assert_eq!(result, "../../index.yml");
    }

    #[test]
    fn visit_func_fails_for_unknown_function() {
        let data = Value::Mapping(Mapping::new());
        let root = Node::from_yaml(&data);
        let data_set = DataSet::from(&root);
        let tree = TemplateTree {
            root: Func("bogus".to_string(), Vec::new()),
        };
        let err = visit(&tree, &data_set).unwrap_err();
        assert!(
            err.contains("Unknown function: 'bogus'."),
            "Got error: {}",
            err
        );
    }

    #[test]
    fn visit_func_link_fails_with_wrong_argument_count() {
        let data = Value::Mapping(Mapping::from_iter(vec![(
            Value::String("PATH".to_string()),
            Value::String("blog/index.yml".to_string()),
        )]));
        let root = Node::from_yaml(&data);
        let data_set = DataSet::from(&root);
        let tree = TemplateTree {
            root: Func("LINK".to_string(), vec![Path::from_segment("PATH")]),
        };
        let err = visit(&tree, &data_set).unwrap_err();
        assert!(
            err.contains("Function 'LINK' expects 2 arguments, got 1."),
            "Got error: {}",
            err
        );
    }

    #[test]
    fn visit_var_fails_if_data_entry_doesnt_exist() {
        let data = Value::Mapping(Mapping::new());
        let root = Node::from_yaml(&data);
        let data_set = DataSet::from(&root);
        let tree = TemplateTree {
            root: Seq(vec![
                Text("Hello, ".to_string()),
                Var(Path::from_segment("name")),
                Text("!".to_string()),
            ]),
        };
        let err = visit(&tree, &data_set).unwrap_err();
        assert!(
            err.contains("Path [name] is not defined in data file."),
            "Got error: {}",
            err
        );
    }

    #[test]
    fn visit_var_fails_if_data_entry_isnt_string() {
        let data = Value::Mapping(Mapping::from_iter(vec![(
            Value::String("name".to_string()),
            Value::Mapping(Mapping::from_iter(vec![
                (
                    Value::String("first".to_string()),
                    Value::String("Julia".to_string()),
                ),
                (
                    Value::String("last".to_string()),
                    Value::String("Doe".to_string()),
                ),
            ])),
        )]));
        let root = Node::from_yaml(&data);
        let data_set = DataSet::from(&root);
        let tree = TemplateTree {
            root: Seq(vec![
                Text("Hello, ".to_string()),
                Var(Path::from_segment("name")),
                Text("!".to_string()),
            ]),
        };
        let err = visit(&tree, &data_set).unwrap_err();
        assert!(
            err.contains("Path [name] does not reference a string in data file."),
            "Got error: {}",
            err
        );
    }

    #[test]
    fn visit_var_with_multi_segment_path() {
        let data = Value::Mapping(Mapping::from_iter(vec![(
            Value::String("section".to_string()),
            Value::Mapping(Mapping::from_iter(vec![(
                Value::String("title".to_string()),
                Value::String("Go Basics".to_string()),
            )])),
        )]));
        let root = Node::from_yaml(&data);
        let data_set = DataSet::from(&root);
        let tree = TemplateTree {
            root: Seq(vec![
                Text("Section title: ".to_string()),
                Var(Path::from_segments(vec!["section", "title"])),
                Text(".".to_string()),
            ]),
        };
        let result = unwrap(visit(&tree, &data_set));
        assert_eq!(result, "Section title: Go Basics.");
    }

    #[test]
    fn visit_foreach() {
        let data = Value::Mapping(Mapping::from_iter(vec![(
            Value::String("section".to_string()),
            Value::Mapping(Mapping::from_iter(vec![(
                Value::String("links".to_string()),
                Value::Sequence(vec![
                    Value::Mapping(Mapping::from_iter(vec![(
                        Value::String("href".to_string()),
                        Value::String("Go Basics".to_string()),
                    )])),
                    Value::Mapping(Mapping::from_iter(vec![(
                        Value::String("href".to_string()),
                        Value::String("Advanced Go".to_string()),
                    )])),
                ]),
            )])),
        )]));
        let root = Node::from_yaml(&data);
        let data_set = DataSet::from(&root);
        let tree = TemplateTree {
            root: Seq(vec![ForEach(
                "link".to_string(),
                Path::from_segments(vec!["section", "links"]),
                Box::new(Seq(vec![
                    Text("- link: ".to_string()),
                    Var(Path::from_segments(vec!["link", "href"])),
                    Text("\n".to_string()),
                ])),
            )]),
        };
        let result = unwrap(visit(&tree, &data_set));
        assert_eq!(
            result,
            "\
- link: Go Basics
- link: Advanced Go
"
        );
    }

    #[test]
    fn visit_foreach_can_access_outer_scope() {
        let data = Value::Mapping(Mapping::from_iter(vec![
            (
                Value::String("category".to_string()),
                Value::String("Food".to_string()),
            ),
            (
                Value::String("subcategories".to_string()),
                Value::Sequence(vec![
                    Value::Mapping(Mapping::from_iter(vec![(
                        Value::String("title".to_string()),
                        Value::String("Recipes".to_string()),
                    )])),
                    Value::Mapping(Mapping::from_iter(vec![(
                        Value::String("title".to_string()),
                        Value::String("Techniques".to_string()),
                    )])),
                ]),
            ),
        ]));
        let root = Node::from_yaml(&data);
        let data_set = DataSet::from(&root);
        let tree = TemplateTree {
            root: Seq(vec![
                Text("Categories:\n".to_string()),
                ForEach(
                    "subcategory".to_string(),
                    Path::from_segments(vec!["subcategories"]),
                    Box::new(Seq(vec![
                        Text("- ".to_string()),
                        Var(Path::from_segments(vec!["category"])),
                        Text(": ".to_string()),
                        Var(Path::from_segments(vec!["subcategory", "title"])),
                        Text("\n".to_string()),
                    ])),
                ),
            ]),
        };
        let result = unwrap(visit(&tree, &data_set));
        assert_eq!(
            result,
            "\
Categories:
- Food: Recipes
- Food: Techniques
"
        );
    }

    #[test]
    fn visit_if_exists() {
        let data = Value::Mapping(Mapping::from_iter(vec![(
            Value::String("items".to_string()),
            Value::Mapping(Mapping::from_iter(vec![(
                Value::String("amount".to_string()),
                Value::String("2".to_string()),
            )])),
        )]));
        let root = Node::from_yaml(&data);
        let data_set = DataSet::from(&root);
        let tree = TemplateTree {
            root: If(
                Expr::from(Exists, vec!["items", "amount"]),
                Box::new(Seq(vec![
                    Text("We have ".to_string()),
                    Var(Path::from_segments(vec!["items", "amount"])),
                    Text(" items left.".to_string()),
                ])),
            ),
        };
        let result = unwrap(visit(&tree, &data_set));
        assert_eq!(result, "We have 2 items left.");
    }

    #[test]
    fn visit_if_not_exists() {
        let data = Value::Mapping(Mapping::from_iter(vec![(
            Value::String("items".to_string()),
            Value::Mapping(Mapping::from_iter(vec![(
                Value::String("count".to_string()),
                Value::String("2".to_string()),
            )])),
        )]));
        let root = Node::from_yaml(&data);
        let data_set = DataSet::from(&root);
        let tree = TemplateTree {
            root: If(
                Expr::from(Exists, vec!["items", "amount"]),
                Box::new(Seq(vec![
                    Text("We have ".to_string()),
                    Var(Path::from_segments(vec!["items", "amount"])),
                    Text(" items left.".to_string()),
                ])),
            ),
        };
        let result = unwrap(visit(&tree, &data_set));
        assert_eq!(result, "");
    }

    #[test]
    fn visit_if_collection() {
        let data = Value::Mapping(Mapping::from_iter(vec![(
            Value::String("section".to_string()),
            Value::Mapping(Mapping::from_iter(vec![(
                Value::String("subsections".to_string()),
                Value::Sequence(vec![
                    Value::Mapping(Mapping::from_iter(vec![(
                        Value::String("title".to_string()),
                        Value::String("Subsection 1".to_string()),
                    )])),
                    Value::Mapping(Mapping::from_iter(vec![(
                        Value::String("title".to_string()),
                        Value::String("Subsection 2".to_string()),
                    )])),
                ]),
            )])),
        )]));
        let root = Node::from_yaml(&data);
        let data_set = DataSet::from(&root);
        let tree = TemplateTree {
            root: If(
                Expr::from(Exists, vec!["section", "subsections"]),
                Box::new(Seq(vec![
                    Text("Subsections:\n".to_string()),
                    ForEach(
                        "subsection".to_string(),
                        Path::from_segments(vec!["section", "subsections"]),
                        Box::new(Seq(vec![
                            Text("- ".to_string()),
                            Var(Path::from_segments(vec!["subsection", "title"])),
                            Text("\n".to_string()),
                        ])),
                    ),
                ])),
            ),
        };
        let result = unwrap(visit(&tree, &data_set));
        assert_eq!(
            result,
            "\
Subsections:
- Subsection 1
- Subsection 2
"
        );
    }

    #[test]
    fn visit_var_with_asterix() {
        let data = Value::Mapping(Mapping::from_iter(vec![(
            Value::String("text".to_string()),
            Value::String(
                "\
Podcast o čaji a zemích dálného východu. Formou rozhovorů *Lindy Mannelové* s *Hubertem Hátle*,\
majitelem Dobré čajovny na Václavském náměstí v Praze."
                    .to_string(),
            ),
        )]));
        let root = Node::from_yaml(&data);
        let data_set = DataSet::from(&root);
        let tree = TemplateTree {
            root: Seq(vec![
                Text("<p>".to_string()),
                Var(Path::from_segment("text")),
                Text("</p>".to_string()),
            ]),
        };
        let result = unwrap(visit(&tree, &data_set));
        assert_eq!(
            result,
            "\
<p>Podcast o čaji a zemích dálného východu. Formou rozhovorů <em>Lindy Mannelové</em> s <em>Hubertem Hátle</em>,\
majitelem Dobré čajovny na Václavském náměstí v Praze.</p>"
        );
    }

    fn unwrap(result: Result<String, String>) -> String {
        assert!(
            result.is_ok(),
            "Error visiting NodeTree: {:?}",
            result.err()
        );
        result.unwrap()
    }
}
