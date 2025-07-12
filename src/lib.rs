use data_file_parser::DataSet;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub mod data_file_parser;
pub mod template_tokenizer;
pub mod template_parser;
pub mod expressions;
pub mod visitor;

pub fn process_single_file(data_file_path: &str, template_file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
  let data_content = fs::read_to_string(data_file_path)
    .map_err(|e| format!("Failed to read data file content. File: '{}'. Error: '{}'.", data_file_path, e))?;
  let data = data_file_parser::parse(&data_content)
    .map_err(|e| format!("Failed to parse data file content. File: '{}'. Error: '{}'.", data_file_path, e))?;
  let data_set = DataSet::from(&data);

  process_file(template_file_path, &data_set)
}

pub fn process_recursive(src_dir_path: &str, dst_dir_path: &str) -> Result<(), Box<dyn std::error::Error>> {
  check_dir_exists(src_dir_path)?;
  check_dir_exists(dst_dir_path)?;

  for entry in WalkDir::new(src_dir_path)
    .into_iter()
    .filter_map(|e| e.ok())
    .filter(|e| e.file_type().is_file())
    .filter(|e| e.path().extension().map_or(false, |ext| ext == "yml"))
    {
      let data_file_path = entry.path();
      let data_file_content = fs::read_to_string(data_file_path)?;
      let data = data_file_parser::parse(&data_file_content)?;
      let data_set = DataSet::from(&data);

      let template_file_path = data_set.get_str(&expressions::Path::from(vec!["template"]))?;
      let parent_path = data_file_path.parent().unwrap();
      let template_file_path = parent_path.join(template_file_path);

      let output = process_file(template_file_path.to_str().unwrap(), &data_set)?;

      let relative_path = data_file_path.strip_prefix(src_dir_path)
        .map_err(|e| format!("Failed to resolve data file relative path: {}", e))?;
      let output_path = Path::new(dst_dir_path)
        .join(relative_path)
        .with_extension("html");

      if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
      }

      fs::write(&output_path, output)?;
      println!("Generated: {:?}", output_path);
  }

  Ok(())
}

fn check_dir_exists(path: &str) -> Result<(), String> {
  let path = Path::new(path);
  if !path.exists() {
    return Err(format!("Failed to load directory. Dir: '{}'. Error: 'Path does not exist.'.", path.display()));
  }
  if !path.is_dir() {
    return Err(format!("Failed to load directory. Dir: '{}'. Error: 'Path is not a directory.'.", path.display()));
  }
  Ok(())
}

fn process_file(template_file_path: &str, data_set: &DataSet) -> Result<String, Box<dyn std::error::Error>> {
  let template_file_content = fs::read_to_string(template_file_path)
    .map_err(|e| format!("Failed to read template file content. File: '{}'. Error: '{}'.", template_file_path, e))?;
  let template_tokens = template_tokenizer::tokenize(&template_file_content)
    .map_err(|e| format!("Failed to parse template file content. File: '{}'. Error: '{}'.", template_file_path, e))?;
  let template_tree = template_parser::parse(&template_tokens)?;

  let result = visitor::visit(&template_tree, data_set)?;
  Ok(result)
}
