# rust_yaml — A Minimal YAML Representation and Parser for Unity-Style Scene Files

`rust_yaml` is a lightweight Rust library providing a minimal but practical `Yaml` data structure and parser for Unity-like `.yaml` scene and asset files.  
It is **not** a full YAML implementation — it’s intentionally limited to a subset of YAML commonly found in Unity `.prefab`, `.asset`, and `.scene` files.

The main goal is to:
- Represent nested Unity YAML data structures in Rust (`Yaml::Hash`, `Yaml::Array`, `Yaml::Value`)
- Load and save `.yaml` files in a human-readable form
- Provide helper utilities for working with Unity-like nested data
- Support round-trip testing (parse → format → parse)

---

## ✨ Features

- **Simple, recursive data model:**
  ```rust
  pub enum Yaml {
      Value(String),
      Hash(HashMap<String, Yaml>),
      Array(Vec<Yaml>),
  }
  ```

- **Human-readable output:** via `Display`  
  Produces valid, indented YAML text that can be written directly to disk.

- **File I/O helpers:**
  - `Yaml::save_to_file()` — pretty-print and save a YAML structure
  - `Yaml::load_from_file()` — parse YAML text from disk

----

- **Convenience utilities:**
  - `Yaml::get_str()` — safely retrieve scalar values from a hash
  - `Yaml::is_flat_hash()` — detect if a hash contains only flat key→value pairs
  - `Yaml::to_indented_string()` — print nested YAML under a key
  - `Yaml::guess_field_type()` — infer Rust-like type names from values (useful for codegen)

---

## 🚀 Example Usage

```rust
use rust_yaml::Yaml;
use std::collections::HashMap;

fn main() -> std::io::Result<()> {
    // Build a nested YAML structure
    let mut inner = HashMap::new();
    inner.insert("r".to_string(), Yaml::Value("0.2".into()));
    inner.insert("g".to_string(), Yaml::Value("0.3".into()));
    inner.insert("b".to_string(), Yaml::Value("0.4".into()));
    inner.insert("a".to_string(), Yaml::Value("1".into()));

    let mut outer = HashMap::new();
    outer.insert("m_Color".to_string(), Yaml::Hash(inner));

    let yaml = Yaml::Hash(outer);

    // Save to file
    yaml.save_to_file("color.yaml")?;

    // Load it back
    let loaded = Yaml::load_from_file("color.yaml")?;
    println!("Loaded YAML:\n{}", loaded);

    Ok(())
}
```

Output file `color.yaml`:
```yaml
m_Color:
  r: 0.2
  g: 0.3
  b: 0.4
  a: 1
```

---

## 🧪 Running Tests

The library includes a set of tests covering roundtrip parsing, array handling, and key-value lookups.

Run all tests using:
```bash
cargo test
```

Example successful test:
```bash
running 5 tests
test tests::test_yaml_array_save_and_load_roundtrip ... ok
test tests::test_parse_modifications_array ... ok
test tests::test_yaml_save_and_load_roundtrip ... ok
```

---

## 🧠 Design Philosophy

This library is not intended to replace Serde or `yaml-rust`.  
It’s designed for **predictable, human-readable YAML parsing and serialization** for game-engine–style configuration formats, particularly Unity asset files althou the first line of each Unity asset would need to be filtered out.
This lib does not parse data as anythig else but `String`. 

**Key principles:**
- Minimal dependencies  
- Deterministic serialization  
- Simple recursive structure  
- Focus on interoperability Unity YAML

---

## 🧩 Example: Parsing Unity-Like YAML

```rust
let yaml_text = r#"
m_TransformParent: {fileID: 0}
m_Modifications:
  - target: {fileID: 8455400915583205629, guid: c9c31f173b4e3274385d017b2f88d207, type: 3}
    propertyPath: m_Name
    value: Canvas
    objectReference: {fileID: 0}
"#;

let lines: Vec<&str> = yaml_text.lines().collect();
let parsed = Yaml::parse_unity_object(&lines);

println!("{}", parsed);
```

Produces:
```yaml
m_TransformParent:
  fileID: 0
m_Modifications:
  - target:
      fileID: 8455400915583205629
      guid: c9c31f173b4e3274385d017b2f88d207
      type: 3
    propertyPath: m_Name
    value: Canvas
    objectReference:
      fileID: 0
```

---

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
rust_yaml = { git = "https://github.com/stela2502/rust_yaml", branch = "main" }
```

---

## 📜 License

BSD 3-Clause License  
© 2025 Stefan Lang

This library is free to use, modify, and redistribute with attribution.

---

## 🧭 Future Directions

Honestly it seam to work for what I need.
If you need to parse to any number - do it from the &str that you can get from the Yaml object's `get_str()`

- Improve indentation and array detection logic
- Codegen helpers for Unity → Rust struct mapping

---

**Enjoy working with structured Unity YAML in Rust!**
