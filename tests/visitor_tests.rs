use yasg::template_parser;
use yasg::data_file_parser::DataSet;
use yasg::visitor;
use std::fs;

#[test]
fn visit_populates_template_file_with_data_file() {
  let template_content = fs::read_to_string("tests/data/example-template.html")
    .unwrap_or_else(|e| panic!("Failed to read template file: '{}'.", e));
  let template_tokens = yasg::template_tokenizer::tokenize(&template_content)
    .unwrap_or_else(|e| panic!("Failed to tokenize template file: '{}'.", e));
  let template = template_parser::parse(&template_tokens)
    .unwrap_or_else(|e| panic!("Failed to parse template file: '{}'.", e));

  let data_content = fs::read_to_string("tests/data/example-data.yml")
    .unwrap_or_else(|e| panic!("Failed to read data file: '{}'.", e));
  let data = yasg::data_file_parser::parse(&data_content)
    .unwrap_or_else(|e| panic!("Failed to parse data file: '{}'.", e));
  let data_set = DataSet::from(&data);

  let output = visitor::visit(&template, &data_set)
    .unwrap_or_else(|e| panic!("Failed to populate data file: '{}'.", e));

  assert_eq!("\
<!DOCTYPE html>
<html lang=\"en\">
  <head>
    <title>Hello World!</title>
  </head>
  <body>
    <h1>Hello World!</h1>
    <p>This is a testing page.</p>
    
      <h2>Items in Backpack:</h2>
      <ul>
        
          <li>sleeping bag - weight: 1.5kg</li>
        
          <li>tent - weight: 2.0kg</li>
        
          <li>water bottle - weight: 0.5kg</li>
        
      </ul>
    
  </body>
</html>
",
  output);
}
