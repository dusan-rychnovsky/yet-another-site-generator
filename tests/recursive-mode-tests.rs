use tempfile::TempDir;
use std::fs;

#[test]
fn recursive_mode_processes_all_files_in_a_given_directory() {

  let src_dir_path = "tests/data/recipes";

  let temp_dir = TempDir::new().unwrap();
  let dst_dir_path = temp_dir.path().join("recipes");

  fs::remove_dir_all(&dst_dir_path).ok();
  fs::create_dir_all(&dst_dir_path).expect("Failed to create output directory");

  let result = yasg::populate_all_files(src_dir_path, dst_dir_path.to_str().unwrap());
  assert!(result.is_ok(), "Error processing recipes: {:?}", result.err());

  let salad_file_path = dst_dir_path.join("salads/shopska-salad.html");
  let salad_file_content = fs::read_to_string(&salad_file_path)
    .unwrap_or_else(|e| panic!("Failed to read file {}: {}", salad_file_path.display(), e));
  assert_eq!("\
<!DOCTYPE html>
<html lang=\"en\">
  <head>
    <title>Recept na: Šopský salát</title>
  </head>
  <body>
    <h1>Recept na: Šopský salát</h1>
    <h2>Suroviny:</h2>
    <ul>
    
      <li>1 okurka</li>
    
      <li>2 rajčata</li>
    
      <li>1 červená paprika</li>
    
      <li>1 červená cibule</li>
    
      <li>200 g sýra feta</li>
    
      <li>50 ml olivového oleje</li>
    
      <li>sůl a pepř podle chuti</li>
    
    </ul>
    <h2>Příprava:</h2>
    <ul>
    
      <li>Nakrájejte <em>okurku</em>, <em>rajčata</em>, <em>papriku</em> a <em>červenou cibuli</em> na kostičky.</li>
    
      <li>V míse smíchejte nakrájenou zeleninu.</li>
    
      <li>...</li>
    
    </ul>
  </body>
</html>
",
  salad_file_content);

  let stew_file_path = dst_dir_path.join("main/stews/beef-stew.html");
  let stew_file_content = fs::read_to_string(&stew_file_path)
    .unwrap_or_else(|e| panic!("Failed to read file {}: {}", stew_file_path.display(), e));
  assert_eq!("\
<!DOCTYPE html>
<html lang=\"en\">
  <head>
    <title>Recept na: Dušené hovězí</title>
  </head>
  <body>
    <h1>Recept na: Dušené hovězí</h1>
    <h2>Suroviny:</h2>
    <ul>
    
      <li>1 kg hovězího masa (např. kližka)</li>
    
      <li>2 velké cibule</li>
    
      <li>3 mrkve</li>
    
      <li>2 brambory</li>
    
      <li>3 stroužky česneku</li>
    
      <li>1 l vývaru (hovězí nebo zeleninový)</li>
    
      <li>2 lžíce rajčatového protlaku</li>
    
      <li>2 bobkové listy</li>
    
      <li>sůl a pepř podle chuti</li>
    
      <li>olej na smažení</li>
    
    </ul>
    <h2>Příprava:</h2>
    <ul>
    
      <li>Na pánvi rozehřejte olej a osmahněte na něm nakrájenou <em>cibuli</em> dozlatova.</li>
    
      <li>Přidejte nakrájené <em>hovězí maso</em> a opékejte, dokud nezhnědne ze všech stran.</li>
    
      <li>...</li>
    
    </ul>
  </body>
</html>
",
  stew_file_content);
}

#[test]
fn process_recursive_fails_if_src_dir_does_not_exist() {
  assert_process_recursive_fails_with_error(
    "tests/data/non-existing-dir",
    "tests/data",
    "Failed to load directory. Dir: 'tests/data/non-existing-dir'. Error: 'Path does not exist.'."
  );
}

#[test]
fn process_recursive_fails_if_src_dir_is_not_a_directory() {
  assert_process_recursive_fails_with_error(
    "tests/data/recipes/salads/shopska-salad.yml",
    "tests/data",
    "Failed to load directory. Dir: 'tests/data/recipes/salads/shopska-salad.yml'. Error: 'Path is not a directory.'."
  );
}

#[test]
fn process_recursive_fails_if_dst_dir_does_not_exist() {
  assert_process_recursive_fails_with_error(
    "tests/data/recipes",
    "tests/data/non-existing-dir",
    "Failed to load directory. Dir: 'tests/data/non-existing-dir'. Error: 'Path does not exist.'."
  );
}

#[test]
fn process_recursive_fails_if_dst_dir_is_not_a_directory() {
  assert_process_recursive_fails_with_error(
    "tests/data/recipes",
    "tests/data/recipes/salads/shopska-salad.yml",
    "Failed to load directory. Dir: 'tests/data/recipes/salads/shopska-salad.yml'. Error: 'Path is not a directory.'."
  );
}

#[test]
fn process_recursive_fails_if_data_file_does_not_have_template_path() {
  let temp_dir = TempDir::new().unwrap();
  assert_process_recursive_fails_with_error(
    "tests/data/invalid-files/data-without-template/",
    temp_dir.path().to_str().unwrap(),
    "Failed to parse data file content. File: 'tests/data/invalid-files/data-without-template/invalid-data.yml'. Error: 'Path [template] is not defined in data file.'"
  );
}

#[test]
fn process_recursive_fails_if_template_file_does_not_exist() {
  let temp_dir = TempDir::new().unwrap();
  assert_process_recursive_fails_with_error(
    "tests/data/invalid-files/data-with-non-existing-template/",
    temp_dir.path().to_str().unwrap(),
    "Failed to populate data file. File: 'tests/data/invalid-files/data-with-non-existing-template/invalid-data.yml'. Failed to read template file content. File: 'tests/data/invalid-files/"
  );
}

#[test]
fn process_recursive_fails_if_data_file_is_not_a_valid_yaml() {
  let temp_dir = TempDir::new().unwrap();
  assert_process_recursive_fails_with_error(
    "tests/data/invalid-files/data-with-syntax-error/",
    temp_dir.path().to_str().unwrap(),
    "Failed to parse data file content. File: 'tests/data/invalid-files/data-with-syntax-error/invalid-data.yml'. Error: 'mapping values are not allowed in this context"
  );
}

#[test]
fn process_recursive_fails_if_template_file_is_not_valid() {
  let temp_dir = TempDir::new().unwrap();
  assert_process_recursive_fails_with_error(
    "tests/data/invalid-files/data-with-template-with-syntax-error/",
    temp_dir.path().to_str().unwrap(),
    "Failed to populate data file. File: 'tests/data/invalid-files/data-with-template-with-syntax-error/invalid-data.yml'. Failed to parse template file content. File: 'tests/data/invalid-files/"
  );
}

fn assert_process_recursive_fails_with_error(
  src_dir_path: &str,
  dst_dir_path: &str,
  expected_error_prefix: &str
) {
  let result = yasg::populate_all_files(src_dir_path, dst_dir_path);
  assert!(result.is_err(), "Expected process_recursive to fail for src dir: '{}' and dst dir: '{}'.", src_dir_path, dst_dir_path);

  let error = result.unwrap_err().to_string();
  assert!(
    error.starts_with(expected_error_prefix),
    "Expected error to start with '{}', but got: '{}'",
    expected_error_prefix,
    error
  );
}
