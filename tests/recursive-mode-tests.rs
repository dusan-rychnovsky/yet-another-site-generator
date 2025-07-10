use tempfile::TempDir;
use std::fs;

#[test]
fn recursive_mode_processes_all_files_in_a_given_directory() {
    
  let src_root_path = "tests/data/recipes";

  let temp_dir = TempDir::new().unwrap();
  let dst_root_path = temp_dir.path().join("recipes");

  fs::remove_dir_all(&dst_root_path).ok();
  fs::create_dir_all(&dst_root_path).expect("Failed to create output directory");

  let result = yasg::process_recursive(src_root_path, dst_root_path.to_str().unwrap());
  assert!(result.is_ok(), "Error processing recipes: {:?}", result.err());

  let salad_file = dst_root_path.join("salads/shopska-salad.html");
  let salad_file_content = fs::read_to_string(&salad_file)
    .unwrap_or_else(|e| panic!("Failed to read file {}: {}", salad_file.display(), e));
  assert_eq!("\
<html>
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

  let stew_file = dst_root_path.join("main/stews/beef-stew.html");
  let stew_file_content = fs::read_to_string(&stew_file)
    .unwrap_or_else(|e| panic!("Failed to read file {}: {}", stew_file.display(), e));
  assert_eq!("\
<html>
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
