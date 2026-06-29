use std::fs;
use tempfile::TempDir;

#[test]
fn populate_blog_exposes_pages_placeholder_and_generates_pages() {
    let src_dir_path = "tests/data/blog";

    let temp_dir = TempDir::new().unwrap();
    let dst_dir_path = temp_dir.path();

    let result = yasg::populate_blog(src_dir_path, dst_dir_path.to_str().unwrap());
    assert!(result.is_ok(), "Error processing blog: {:?}", result.err());

    let index_file_path = dst_dir_path.join("index.html");
    let index_file_content = fs::read_to_string(&index_file_path)
        .unwrap_or_else(|e| panic!("Failed to read file {}: {}", index_file_path.display(), e));
    assert_eq!(
        "\
<!DOCTYPE html>
<html lang=\"en\">
  <head>
    <title>Welcome to my blog!</title>
  </head>
  <body>
    <h1>Welcome to my blog!</h1>
    
      <h2>Overnight Oats</h2>
    
      <h2>Car Clowns</h2>
    
      <h2>Welcome to my blog!</h2>
    
  </body>
</html>
",
        index_file_content
    );

    let oats_file_path = dst_dir_path.join("cooking/recipes/overnight-oats.html");
    let oats_file_content = fs::read_to_string(&oats_file_path)
        .unwrap_or_else(|e| panic!("Failed to read file {}: {}", oats_file_path.display(), e));
    assert_eq!(
        "\
<!DOCTYPE html>
<html lang=\"en\">
  <head>
    <title>Overnight Oats</title>
  </head>
  <body>
    <h1>Overnight Oats</h1>
    
      <p>Overnight oats are a quick and easy breakfast option.</p>
    
      <p>They can be customized with various toppings and flavors.</p>
    
  </body>
</html>
",
        oats_file_content
    );
}
