use std::fs;
use yasg::data_file_parser;

#[test]
fn parse_loads_and_parses_data_file() {
    let content = fs::read_to_string("tests/data/example-data.yml")
        .unwrap_or_else(|e| panic!("Failed to read data file: {}", e));
    let data = data_file_parser::parse(&content)
        .unwrap_or_else(|e| panic!("Failed to parse data file: {}", e));

    assert_eq!(data["title"], "Hello World!");

    let backpack = data["backpack"]
        .as_mapping()
        .expect("backpack should be a mapping");
    let items = backpack["items"]
        .as_sequence()
        .expect("backpack.items should be a sequence");
    assert_eq!(items.len(), 3);

    assert_eq!(items[0]["name"], "sleeping bag");
    assert_eq!(items[0]["weight"], "1.5kg");

    assert_eq!(items[1]["name"], "tent");
    assert_eq!(items[1]["weight"], "2.0kg");

    assert_eq!(items[2]["name"], "water bottle");
    assert_eq!(items[2]["weight"], "0.5kg");
}
