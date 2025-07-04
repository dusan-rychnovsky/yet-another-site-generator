# YASG - Yet Another (Static) Site Generator

A generator of static HTML pages. Compiles a given HTML template and a given YAML data file to an HTML page with populated content.

## How To

### Build

```
git clone https://github.com/dusan-rychnovsky/yet-another-site-generator.git
cd yet-another-site-generator
cargo test
cargo build --release
```

### Run

```
./target/release/yasg [TEMPLATE-FILE] [DATA-FILE] > output.html
```

## Example

Consider this simple example.

Step 1] Create files `example-template.html` and `example-data.yml` with following contents:

**example-template.html:**
```html
<html>
  <head>
    <title>[title]</title>
  </head>
  <body>
    <h1>[title]</h1>
    <p>This is a testing page.</p>
    [if exists backpack.items]
      <h2>Items in Backpack:</h2>
      <ul>
        [for item in backpack.items]
          <li>[item.name] - weight: [item.weight]</li>
        [endfor item]
      </ul>
    [endif]
  </body>
</html>
```

**example-data.yml:**
```yml
title: Hello World!
backpack:
  items:
    - name: sleeping bag
      weight: '1.5kg'
    - name: tent
      weight: '2.0kg'
    - name: water bottle
      weight: '0.5kg'
```

Step 2] Execute YASG:
```
./yasg example-template.html example-data.yml > output.html
```

Step 3] The result will be:

**output.html:**
```html
<html>
  <head>
    <title>Hello World!</title>
  </head>
  <body>
    <h1>Hello World!</h1>
    <p>This is a testing page.</p>
      <h2>Items in Backpack:</h2>
      <ul>
          <li>sleeping bag - weight: 1.5kg</li>
          <li>tent - weight: 2.0kg</li>
          <li>water bottle - weight: 0.5kg</li>
      </ul>
  </body>
</html>
```
