use yet_another_site_generator::data_file_parser;

#[test]
fn parse_loads_and_parses_data_file() {
  let page_data = data_file_parser::parse("tests/data/data-go.yml");

  assert!(page_data.is_ok(), "Expected to parse data file successfully. Error: {:?}", page_data.err());
  let page_data = page_data.unwrap();

  // title
  assert_eq!(page_data.title, "Hra Go");

  // crumbs
  assert_eq!(page_data.crumbs.len(), 3);
  assert_eq!(page_data.crumbs[0].text, "Domů");
  assert_eq!(page_data.crumbs[0].href, Option::Some(String::from("/")));
  assert_eq!(page_data.crumbs[1].text, "Zdroje");
  assert_eq!(page_data.crumbs[2].text, "Go");

  // sections
  assert_eq!(page_data.sections.len(), 4);

  // section 0
  assert_eq!(page_data.sections[0].title, "Go klub Můstek");
  assert_eq!(page_data.sections[0].labels, "CZ. Klub.");
  assert_eq!(page_data.sections[0].img, "img/go-club-mustek.png");
  assert_eq!(page_data.sections[0].content.len(), 2);
  assert_eq!(page_data.sections[0].content[1], "Adresa: Na Můstku 8/380, Praha 1");
  assert_eq!(page_data.sections[0].links.len(), 2);
  assert_eq!(page_data.sections[0].links[0].kind, "Web");
  assert_eq!(page_data.sections[0].links[0].text, "Go klub Můstek");
  assert_eq!(page_data.sections[0].links[0].href, "https://goklubmustek.j2m.cz/");
  assert_eq!(page_data.sections[0].links[1].kind, "Facebook");
  assert_eq!(page_data.sections[0].links[1].text, "Go klub Můstek");
  assert_eq!(page_data.sections[0].links[1].href, "https://www.facebook.com/groups/389119397921527/");

  // other sections
  assert_eq!(page_data.sections[1].title, "Go Magic");
  assert_eq!(page_data.sections[2].title, "Dwyrin (BattsGo)");
  assert_eq!(page_data.sections[3].title, "Triton Baduk");
}
