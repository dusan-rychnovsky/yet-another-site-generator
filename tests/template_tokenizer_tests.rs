use yasg::template_tokenizer::{self, TemplateToken, TemplateToken::*};
use yasg::expressions::{Path, Expr, Predicate::*};
use std::fs;

#[test]
fn tokenize_template_go_html() {
    let content = fs::read_to_string("tests/data/go-template.html")
      .unwrap_or_else(|e| panic!("Failed to read template file: {}", e));

    let tokens = template_tokenizer::tokenize(&content);
    assert!(tokens.is_ok(), "Expected to tokenize template file successfully. Error: {:?}", tokens.err());
    let tokens = tokens.unwrap();

    assert_eq!(53, tokens.len());

    assert_token_text(
      &tokens[0],
      "<!DOCTYPE html>\n<html lang=\"cs\">",
      "<title>Mé oblíbené zdroje informací na téma: "
    );
    assert_eq!(Var(Path::from(vec!["title"])), tokens[1]);
    assert_token_text(
      &tokens[2],
      "</title>\n  <link ",
      "<ol class=\"breadcrumb mb-0\">\n        "
    );
    assert_eq!(For("crumb", Path::from(vec!["crumbs"])), tokens[3]);
    assert_eq!(Text("\n          <li class=\"breadcrumb-item\"><a href=\""), tokens[4]);
    assert_eq!(Var(Path::from(vec!["crumb", "href"])), tokens[5]);
    assert_eq!(Text("\">"), tokens[6]);
    assert_eq!(Var(Path::from(vec!["crumb", "text"])), tokens[7]);
    assert_eq!(Text("</a></li>\n        "), tokens[8]);
    assert_eq!(EndFor("crumb"), tokens[9]);
    // ...
    assert_eq!(If(Expr::from(Exists, vec!["section", "subsections"])), tokens[29]);
    assert_eq!(Text("\n                  <ul>\n                    "), tokens[30]);
    assert_eq!(For("subsection", Path::from(vec!["section", "subsections"])), tokens[31]);
    assert_eq!(Text("\n                      <li class=\"mb-1\">\n                        <em>"), tokens[32]);
    assert_eq!(Var(Path::from(vec!["subsection", "title"])), tokens[33]);
    assert_eq!(Text("</em> - "), tokens[34]);
    assert_eq!(Var(Path::from(vec!["subsection", "content"])), tokens[35]);
    assert_eq!(Text("\n                      </li>\n                    "), tokens[36]);
    assert_eq!(EndFor("subsection"), tokens[37]);
    assert_eq!(Text("\n                  </ul>\n                "), tokens[38]);
    assert_eq!(EndIf, tokens[39]);
    // ...
    assert_token_text(
      &tokens[52],
      "\n      </div>\n    </div>",
      "</body>\n</html>\n"
    );
}

fn assert_token_text(token: &TemplateToken, expected_start: &str, expected_end: &str) {
  match token {
    Text(s) => {
      assert!(s.starts_with(expected_start), "unexpected text: '{}'", s);
      assert!(s.ends_with(expected_end), "unexpected text: '{}'", s);
    }
    other => panic!("unexpected token: {:?}", other),
  }
}