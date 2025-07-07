use data_file_parser::DataSet;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub mod data_file_parser;
pub mod template_parser;
pub mod template_tokenizer;
pub mod expressions;
pub mod visitor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args: Vec<String> = std::env::args().collect();

  if args.len() < 2 {
    print_usage_and_quit(&args);
  }

  // recursive mode
  if args[1] == "-r" {
    if args.len() != 4 {
      print_usage_and_quit(&args);
    }
    let src_root_path = &args[2];
    let dst_root_path = &args[3];
    process_recursive(&src_root_path, dst_root_path)?;
  }
  // single file mode
  else {
    if args.len() != 3 {
      print_usage_and_quit(&args);
    }
    let template_path = &args[1];
    let data_path = &args[2];
    process_single_file(template_path, data_path)?;
  }

  Ok(())
}

fn print_usage_and_quit(args: &[String]) {
  eprintln!("Usage: {} <template-file> <data-file>", args[0]);
  eprintln!("   or: {} -r <source-dir> <dest-dir>", args[0]);
  std::process::exit(1);
}

fn process_single_file(template_path: &str, data_path: &str) -> Result<(), Box<dyn std::error::Error>> {
  let data_content = fs::read_to_string(data_path)?;
  let data = data_file_parser::parse(&data_content)?;
  let data_set = DataSet::from(&data);

  let output = process_file(template_path, &data_set)?;
  println!("{}", output);

  Ok(())
}

fn process_recursive(src_root_path: &str, dst_root_path: &str) -> Result<(), Box<dyn std::error::Error>> {
  let root = Path::new(src_root_path);
  if !root.exists() {
    return Err(format!("Root path does not exist: {}", src_root_path).into());
  }

  for entry in WalkDir::new(root)
    .into_iter()
    .filter_map(|e| e.ok())
    .filter(|e| e.file_type().is_file())
    .filter(|e| e.path().extension().map_or(false, |ext| ext == "yml"))
    {
      let data_path = entry.path();
      let data_content = fs::read_to_string(data_path)?;
      let data = data_file_parser::parse(&data_content)?;
      let data_set = DataSet::from(&data);

      let template_path = data_set.get_str(&expressions::Path::from(vec!["template"]))?;
      let parent_path = data_path.parent().unwrap();
      let template_path = parent_path.join(template_path);

      let output = process_file(template_path.to_str().unwrap(), &data_set)?;

      let relative_path = data_path.strip_prefix(src_root_path)
        .map_err(|e| format!("Failed to resolve data file relative path: {}", e))?;
      let output_path = Path::new(dst_root_path)
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

fn process_file(template_path: &str, data_set: &DataSet) -> Result<String, Box<dyn std::error::Error>> {
  let template_content = fs::read_to_string(template_path)?;
  let template_tokens = template_tokenizer::tokenize(&template_content)?;
  let template_tree = template_parser::parse(&template_tokens)?;

  let result = visitor::visit(&template_tree, data_set)?;
  Ok(result)
}
