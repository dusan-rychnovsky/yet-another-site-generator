use std::fs;

pub mod data_file_parser;
pub mod template_parser;
pub mod template_tokenizer;
pub mod expressions;
pub mod visitor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args: Vec<String> = std::env::args().collect();
  let template_path = &args[1];
  let data_path = &args[2];

  let template_content = fs::read_to_string(template_path)?;
  let template_tokens = template_tokenizer::tokenize(&template_content)?;
  // println!("tokens: {:#?}", template_tokens);
  let template_tree = template_parser::parse_tokens(&template_tokens)?;
  // println!("tree: {:#?}", template_tree);

  let data_content = fs::read_to_string(data_path)?;
  let data_set = data_file_parser::parse_content(&data_content)?;
  // println!("data: {:#?}", data_set);

  let result = visitor::visit(&template_tree, &data_set)?;
  println!("{}", result);

  Ok(())
}
