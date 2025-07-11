#[test]
fn single_file_mode_processes_given_data_file() {

  let data_file_path = "tests/data/recipes/salads/shopska-salad.yml";
  let template_file_path = "tests/data/recipes/template.html";

  let result = yasg::process_single_file(data_file_path,template_file_path)
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
