use yasg::template_tokenizer::{self, TemplateToken::*};
use yasg::expressions::{Path, Expr, Predicate::*};
use std::fs;

#[test]
fn tokenize_template_example_html() {

  let content = fs::read_to_string("tests/data/example-template.html")
    .unwrap_or_else(|e| panic!("Failed to read template file: {}", e));
  let tokens = template_tokenizer::tokenize(&content)
    .unwrap_or_else(|e| panic!("Failed to tokenize template file: {}", e));

  assert_eq!(17, tokens.len());

  assert_eq!(
    Text("\
<!DOCTYPE html>
<html lang=\"en\">
  <head>
    <title>"),
    tokens[0]
  );

  assert_eq!(Var(Path::from_segment("title")), tokens[1]);

  assert_eq!(
    Text("\
</title>
  </head>
  <body>
    <h1>"),
    tokens[2]
  );

  assert_eq!(Var(Path::from_segment("title")), tokens[3]);

  assert_eq!(
    Text("\
</h1>
    <p>This is a testing page.</p>
    "),
    tokens[4]
  );

  assert_eq!(If(Expr::from(Exists, vec!["backpack", "items"])), tokens[5]);

  assert_eq!(
    Text("
      <h2>Items in Backpack:</h2>
      <ul>
        "),
    tokens[6]
  );

  assert_eq!(For("item", Path::from_segments(vec!["backpack", "items"])), tokens[7]);

  assert_eq!(
    Text("
          <li>"),
    tokens[8]
  );

  assert_eq!(Var(Path::from_segments(vec!["item", "name"])), tokens[9]);
  assert_eq!(Text(" - weight: "), tokens[10]);
  assert_eq!(Var(Path::from_segments(vec!["item", "weight"])), tokens[11]);
  assert_eq!(Text("</li>\n        "), tokens[12]);
  assert_eq!(EndFor("item"), tokens[13]);

  assert_eq!(
    Text("
      </ul>
    "),
    tokens[14]);

  assert_eq!(EndIf, tokens[15]);

  assert_eq!(
    Text("
  </body>
</html>
"),
    tokens[16]
  );
}
