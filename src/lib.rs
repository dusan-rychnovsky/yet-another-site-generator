use data_file_parser::DataSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use walkdir::WalkDir;

pub mod data_file_parser;
pub mod template_tokenizer;
pub mod template_parser;
pub mod expressions;
pub mod visitor;

pub fn populate_all_files(src_dir_path: &str, dst_dir_path: &str) -> Result<(), Box<dyn std::error::Error>> {
  check_dir_exists(src_dir_path)?;
  check_dir_exists(dst_dir_path)?;

  for entry in WalkDir::new(src_dir_path)
    .into_iter()
    .filter_map(|e| e.ok())
    .filter(|e| e.file_type().is_file())
    .filter(|e| e.path().extension().map_or(false, |ext| ext == "yml"))
    {
      let data_file_path = entry.path();
      let output = populate_file(data_file_path.to_str().unwrap(), None)?;
      let (output_path, output_dir_path) = construct_output_path(data_file_path, src_dir_path, dst_dir_path)?;
      fs::create_dir_all(output_dir_path)?;
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

fn construct_output_path(data_file_path: &Path, src_dir_path: &str, dst_dir_path: &str) -> Result<(PathBuf, PathBuf), String> {
  let relative_data_file_path = data_file_path.strip_prefix(src_dir_path)
    .map_err(|e| format!("Failed to resolve relative data file path. Error: '{}'.", e))?;
  let output_path = Path::new(dst_dir_path)
    .join(relative_data_file_path)
    .with_extension("html");
  let output_dir_path = output_path.parent()
    .ok_or_else(|| format!("Failed to resolve parent directory path. File path: '{}'.", output_path.display()))?
    .to_path_buf();
  Ok((output_path, output_dir_path))
}

pub fn populate_file(data_file_path: &str, template_file_path: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
  let data_content = fs::read_to_string(data_file_path)
    .map_err(|e| format!("Failed to read data file content. File: '{}'. Error: '{}'.", data_file_path, e))?;
  let data = data_file_parser::parse(&data_content)
    .map_err(|e| format!("Failed to parse data file content. File: '{}'. Error: '{}'.", data_file_path, e))?;
  let data_set = DataSet::from(&data);

  let template_file_path = look_up_template_file_path(&data_set, data_file_path, template_file_path)?;
  let template_file_content = fs::read_to_string(&template_file_path)
    .map_err(|e| format!("Failed to populate data file. File: '{}'. Failed to read template file content. File: '{}'. Error: '{}'.", data_file_path, template_file_path, e))?;
  let template_tokens = template_tokenizer::tokenize(&template_file_content)
    .map_err(|e| format!("Failed to populate data file. File: '{}'. Failed to parse template file content. File: '{}'. Error: '{}'.", data_file_path, template_file_path, e))?;
  let template_tree = template_parser::parse(&template_tokens)
    .map_err(|e| format!("Failed to populate data file. File: '{}'. Failed to parse template file content. File: '{}'. Error: '{}'.", data_file_path, template_file_path, e))?;

  let result = visitor::visit(&template_tree, &data_set)
    .map_err(|e| format!("Failed to populate data file. File: '{}'. Error: '{}'.", data_file_path, e))?;

  Ok(result)
}

fn look_up_template_file_path(data_set: &DataSet, data_file_path: &str, template_file_path: Option<&str>) -> Result<String, String> {
  let template_file_path = if let Some(template_file_path) = template_file_path {
    template_file_path.to_string()
  }
  else {
    let template_file_path = data_set.get_str(&expressions::Path::from_segment("template"))
      .map_err(|e| format!("Failed to parse data file content. File: '{}'. Error: '{}'.", data_file_path, e))?;
    let parent_path = Path::new(data_file_path).parent().unwrap();
    parent_path.join(template_file_path).to_string_lossy().to_string()
  };
  Ok(template_file_path)
}
