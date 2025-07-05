# YASG - Yet Another (Static) Site Generator

A generator of static HTML pages. Compiles a given HTML template and a given YAML data file to an HTML page with populated content.

Runs in two modes:

* **Single-File mode**:  
  Takes a single template- and a single data file and prints the result to standard output.
* **Recursive mode**:  
  Takes a source- and a destination directory. Traverses the source directory recursively, processes
  each data file (i.e. `*.yml`) found and generates corresponding HTML files in the destination directory,
  while preserving directory structure.
  In recursive mode, each data file contains a root-level `template` field, which specifies the path to its corresponding template file.

See examples below.

## How To

### Build

```
$ git clone https://github.com/dusan-rychnovsky/yet-another-site-generator.git
$ cd yet-another-site-generator
$ cargo test
$ cargo build --release
```

### Run

```
// single-file mode
$ ./target/release/yasg [TEMPLATE-FILE] [DATA-FILE] > output.html

// recursive mode
$ ./target/release/yasg -r [SRC-DIR] [DEST-DIR]
```

## Examples

### Example #1: Generating a single file

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
