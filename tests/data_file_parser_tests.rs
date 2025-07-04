use yasg::data_file_parser;
use std::option::Option::None;

#[test]
fn parse_loads_and_parses_data_file() {
  let data = data_file_parser::parse("tests/data/go-data.yml");

  assert!(data.is_ok(), "Expected to parse data file successfully. Error: {:?}", data.err());
  let data = data.unwrap().data;

  // title
  assert_eq!(data["title"], "Hra Go");

  // crumbs
  let crumbs = data["crumbs"].as_sequence()
    .expect("crumbs should be a sequence");
  assert_eq!(crumbs.len(), 3);
  assert_eq!(crumbs[0]["text"], "Domů");
  assert_eq!(crumbs[0]["href"], "/");
  assert_eq!(crumbs[1]["text"], "Zdroje");
  assert_eq!(crumbs[2]["text"], "Go");

  // sections
  let sections = data["sections"].as_sequence()
    .expect("sections should be a sequence");
  assert_eq!(sections.len(), 4);

  // section 0
  let section = &sections[0];
  assert_eq!(section["title"], "Go klub Můstek");
  assert_eq!(section["labels"], "CZ. Klub.");
  assert_eq!(section["img"], "img/go-club-mustek.png");
  let content = section["content"].as_sequence()
    .expect("sections.content should be a sequence");
  assert_eq!(content.len(), 2);
  assert_eq!(content[1], "Adresa: Na Můstku 8/380, Praha 1");
  assert_eq!(None, section.get("subsections"), "section[0] should have no subsections");
  let links = section["links"].as_sequence()
    .expect("sections.links should be a sequence");
  assert_eq!(links.len(), 2);
  assert_eq!(links[0]["kind"], "Web");
  assert_eq!(links[0]["text"], "Go klub Můstek");
  assert_eq!(links[0]["href"], "https://goklubmustek.j2m.cz/");
  assert_eq!(links[1]["kind"], "Facebook");
  assert_eq!(links[1]["text"], "Go klub Můstek");
  assert_eq!(links[1]["href"], "https://www.facebook.com/groups/389119397921527/");

  // section 2
  let section = &sections[2];
  assert_eq!(section["title"], "Dwyrin (BattsGo)");
  let subsections = section["subsections"].as_sequence()
    .expect("sections.subsections should be a sequence");
  assert_eq!(subsections.len(), 2);
  assert_eq!(subsections[0]["title"], "YouTube kanál Dwyrin");
  assert_eq!(subsections[1]["title"], "Knižní série Learning Go");
  assert_eq!(subsections[1]["content"], "Principy hry Go vysvětlené názornou a velmi čtivou formou. Obsahově i stylem podobné jeho YouTube videím.");

  // other sections
  assert_eq!(sections[1]["title"], "Go Magic");
  assert_eq!(sections[3]["title"], "Triton Baduk");
}