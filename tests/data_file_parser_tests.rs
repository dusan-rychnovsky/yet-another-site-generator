use yet_another_site_generator::data_file_parser;

#[test]
fn parse_loads_and_parses_data_file() {
  let page_data = data_file_parser::parse("tests/data/data-go.yml");

  assert!(page_data.is_ok(), "Expected to parse data file successfully. Error: {:?}", page_data.err());
  let page_data = page_data.unwrap();

  // title
  assert_eq!(page_data.title, "Hra Go");

  // crumbs
  let crumbs = &page_data.crumbs;
  assert_eq!(crumbs.len(), 3);
  assert_eq!(crumbs[0].text, "Domů");
  assert_eq!(crumbs[0].href, Some(String::from("/")));
  assert_eq!(crumbs[1].text, "Zdroje");
  assert_eq!(crumbs[2].text, "Go");

  // sections
  assert_eq!(page_data.sections.len(), 4);

  // section 0
  let section = &page_data.sections[0];
  assert_eq!(section.title, "Go klub Můstek");
  assert_eq!(section.labels, "CZ. Klub.");
  assert_eq!(section.img, "img/go-club-mustek.png");
  assert_eq!(section.content.len(), 2);
  assert_eq!(section.content[1], "Adresa: Na Můstku 8/380, Praha 1");
  assert_eq!(section.subsections.len(), 0);
  let links = &section.links;
  assert_eq!(links.len(), 2);
  assert_eq!(links[0].kind, "Web");
  assert_eq!(links[0].text, "Go klub Můstek");
  assert_eq!(links[0].href, "https://goklubmustek.j2m.cz/");
  assert_eq!(links[1].kind, "Facebook");
  assert_eq!(links[1].text, "Go klub Můstek");
  assert_eq!(links[1].href, "https://www.facebook.com/groups/389119397921527/");

  // section 2
  let section = &page_data.sections[2];
  assert_eq!(section.title, "Dwyrin (BattsGo)");
  let subsections = &section.subsections;
  assert_eq!(subsections.len(), 2);
  assert_eq!(subsections[0].title, "YouTube kanál Dwyrin");
  assert_eq!(subsections[1].title, "Knižní série Learning Go");
  assert_eq!(subsections[1].content, "Principy hry Go vysvětlené názornou a velmi čtivou formou. Obsahově i stylem podobné jeho YouTube videím.");

  // other sections
  assert_eq!(page_data.sections[1].title, "Go Magic");
  assert_eq!(page_data.sections[3].title, "Triton Baduk");
}