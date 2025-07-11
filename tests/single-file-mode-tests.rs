#[test]
fn single_file_mode_processes_given_data_file() {

  let data_file_path = "tests/data/recipes/salads/shopska-salad.yml";
  let template_file_path = "tests/data/recipes/template.html";

  let result = yasg::process_single_file(data_file_path, template_file_path)
    .unwrap_or_else(|e| panic!("Error processing shopska-salad.yaml: {:?}", e));

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
  result);
}

#[test]
fn process_single_file_fails_if_given_a_directory_instead_of_data_file() {
  assert_process_single_file_fails_with_error(
    "tests/data/recipes",
    "tests/data/recipes/template.html",
    "Failed to read data file content. File: 'tests/data/recipes'. Error: "
  );
}

#[test]
fn process_single_file_fails_if_data_file_does_not_exist() {
  assert_process_single_file_fails_with_error(
    "tests/data/recipes/non-existing-file.yml",
    "tests/data/recipes/template.html",
    "Failed to read data file content. File: 'tests/data/recipes/non-existing-file.yml'. Error: "
  );
}

#[test]
fn process_single_file_fails_if_data_file_is_not_a_valid_yaml() {
  assert_process_single_file_fails_with_error(
    "tests/data/invalid-files/invalid-data.yml",
    "tests/data/recipes/template.html",
    "Failed to parse data file content. File: 'tests/data/invalid-files/invalid-data.yml'. Error: "
  );
}

#[test]
fn process_single_file_fails_if_given_a_directory_instead_of_template_file() {
  assert_process_single_file_fails_with_error(
    "tests/data/recipes/salads/shopska-salad.yml",
    "tests/data/recipes/salads",
    "Failed to read template file content. File: 'tests/data/recipes/salads'. Error: "
  );
}

#[test]
fn process_single_file_fails_if_template_file_does_not_exist() {
  assert_process_single_file_fails_with_error(
    "tests/data/recipes/salads/shopska-salad.yml",
    "tests/non-existing-template.html",
    "Failed to read template file content. File: 'tests/non-existing-template.html'. Error: "
  );
}

#[test]
fn process_single_file_fails_if_template_file_is_not_valid() {
  assert_process_single_file_fails_with_error(
    "tests/data/recipes/salads/shopska-salad.yml",
    "tests/data/invalid-files/invalid-template.html",
    "Failed to parse template file content. File: 'tests/data/invalid-files/invalid-template.html'. Error: 'Missing closing bracket.'"
  );
}

fn assert_process_single_file_fails_with_error(
  data_file_path: &str,
  template_file_path: &str,
  expected_error_prefix: &str
) {
  let result = yasg::process_single_file(data_file_path, template_file_path);
  assert!(result.is_err(), "Expected process_single_file to fail for data file: '{}' and template file: '{}'.", data_file_path, template_file_path);

  let error = result.unwrap_err().to_string();
  assert!(
    error.starts_with(expected_error_prefix),
    "Expected error to start with '{}', but got: '{}'",
    expected_error_prefix,
    error
  );
}
