use std::collections::HashMap;
use std::fmt;

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

#[derive(Debug, Clone)]
pub enum Yaml {
    Value(String),             // just a raw string
    Hash(HashMap<String, Yaml>), // key -> Yaml
    Array(Vec<Yaml>),          // sequence of Yaml
}

impl fmt::Display for Yaml {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with_indent(f, 0)
    }
}

impl Yaml {

    /// Save a Yaml structure to a file as human-readable text.
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        // If you already have `Display` implemented for Yaml, this will just work.
        let text = format!("{}", self);
        fs::write(path, text)
    }

    /// Load a Yaml structure from a file.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let text = fs::read_to_string(path)?;
        let lines: Vec<&str> = text.lines().collect();
        // Assuming you have a `parse_yaml` function returning Yaml
        Ok(Self::parse_unity_object(&lines))
    }

    // determine if it can be written inline
    pub fn is_flat_hash(&self) -> bool {
        match self {
            Yaml::Hash(map) => map.values().all(|v| matches!(v, Yaml::Value(_))),
            _ => false,
        }
    }

    fn to_snake_case(name: &str) -> String {
        let mut result = String::new();
        for (i, ch) in name.chars().enumerate() {
            if ch.is_uppercase() {
                if i > 0 {
                    result.push('_');
                }
                result.push(ch.to_ascii_lowercase());
            } else {
                result.push(ch);
            }
        }
        result
    }

    fn guess_field_type(yaml: &Yaml) -> String {
        match yaml {
            Yaml::Value(_) => "String".to_string(),
            Yaml::Hash(map) if map.is_empty() => "UnityEmpty".to_string(),
            Yaml::Hash(_) => "UnityData".to_string(),
            Yaml::Array(_) => "Vec<String>".to_string(),
        }
    }

    /// Pretty-print this Yaml node as if it were nested under a parent key.
    /// Example output:
    ///   m_Color:
    ///     r: 0.2
    ///     g: 0.3
    ///     b: 0.4
    ///     a: 1
    pub fn to_indented_string(&self, key:&str ) -> String{
        let mut tmp = HashMap::<String, Yaml>::new();
        tmp.insert(key.to_string(), self.clone() );
        format!("{}", Yaml::Hash(tmp))
    }


    pub fn get_val(&self) -> Option<&str> {
        if let Yaml::Value(s) = self {
            Some(s)
        } else {
            None
        }
    }
    

    /// Convenience helper: get a string field from a Yaml::Hash.
    pub fn get_str(&self, key: &str) -> Option<&str> {
        if let Yaml::Hash(map) = self {
            match map.get(key)? {
                Yaml::Value(v) => Some(v),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn fmt_with_indent(&self, f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
        let indent_str = "  ".repeat(indent);
        match self {
            Yaml::Value(v) => write!(f, "{}", v),
            Yaml::Hash(h) => {
                for (k, v) in h {
                    match v {
                        Yaml::Value(_) => writeln!(f, "{}{}: {}", indent_str, k, v)?,
                        Yaml::Hash(_) | Yaml::Array(_) => {
                            writeln!(f, "{}{}:", indent_str, k)?;
                            v.fmt_with_indent(f, indent + 1)?;
                        }
                    }
                }
                Ok(())
            }
            Yaml::Array(a) => {
                for item in a {
                    write!(f, "{}- ", indent_str)?;
                    match item {
                        Yaml::Value(_) => writeln!(f, "{}", item)?,
                        Yaml::Hash(_) | Yaml::Array(_) => {
                            writeln!(f)?;
                            item.fmt_with_indent(f, indent + 1)?;
                        }
                    }
                }
                Ok(())
            }
        }
    }

    pub fn parse_unity_object(lines: &[&str]) -> Yaml {
        fn parse_block(lines: &[&str], start_indent: usize) -> (Yaml, usize) {
            let mut map: HashMap<String, Yaml> = HashMap::new();
            let mut arr: Vec<Yaml> = Vec::new();
            let mut is_array = false;
            let mut i = 0;

            #[cfg(debug_assertions)]
            println!(
                "\n🧩 ENTERING block (indent {}) with {} lines.",
                start_indent,
                lines.len(),
            );

            while i < lines.len() {
                let line = lines[i];
                let indent = line.chars().take_while(|c| *c == ' ').count();

                // Block termination condition
                if indent < start_indent {
                    #[cfg(debug_assertions)]
                    println!("↩️  Exiting block at line {} (indent {} < start_indent {})", i, indent, start_indent);
                    break;
                }

                let trimmed = line.trim();

                if trimmed.is_empty() {
                    #[cfg(debug_assertions)]
                    println!("🪶 Skipping empty line {}", i);
                    i += 1;
                    continue;
                }

                // --- ARRAY ELEMENT DETECTED ---
                if trimmed.starts_with('-') {
                    is_array = true;
                    let val_str = trimmed[1..].trim();

                    #[cfg(debug_assertions)]
                    println!("📜 Line {} (indent {}): ARRAY element '{}'", i, indent, val_str);

                    // Case 1: "-" followed by nested block
                    if val_str.is_empty() {
                        //#[cfg(debug_assertions)]
                        panic!("  ↳ Array element with nested block below (indent > {})", indent);

                        let (child, consumed) = parse_block(&lines[i + 1..], indent);
                        arr.push(child);
                        i += consumed + 1;
                        continue;
                    }

                    // Case 2: Inline array element with "key: value"
                    if let Some(idx) = val_str.find(':') {
                        let key = val_str[..idx].trim().to_string();
                        let value = val_str[idx + 1..].trim();
                        let mut child_map: HashMap<String, Yaml> = HashMap::new();

                        #[cfg(debug_assertions)]
                        println!("  ↳ Inline key/value: {} : {}", key, value);

                        if value.starts_with('{') && value.ends_with('}') {
                            child_map.insert(key, parse_inline_mapping(value));
                        } else {
                            child_map.insert(key, Yaml::Value(value.to_string()));
                        }

                        // Check next lines for nested fields under same array element
                        if i + 1 < lines.len() {
                            let next_indent = lines[i + 1].chars().take_while(|c| *c == ' ').count();
                            if next_indent > indent {
                                #[cfg(debug_assertions)]
                                println!(
                                    "  ↳ Parsing nested block for array element (indent {} -> {})",
                                    indent, next_indent
                                );
                                let (nested, consumed) = parse_block(&lines[i + 1..], indent + 2);

                                if let Yaml::Hash(nmap) = nested {
                                    #[cfg(debug_assertions)]
                                    println!("    ↳ Merging nested keys into array element: {:?}", nmap.keys());
                                    for (k, v) in nmap {
                                        child_map.insert(k, v);
                                    }
                                } else {
                                    panic!(
                                        "❌ Unexpected YAML structure in array element at line {} (partial {:?}). Problem line:\n'{}'",
                                        i + consumed + 1,
                                        child_map,
                                        lines.get(i + consumed + 1).unwrap_or(&"<EOF>")
                                    );
                                }
                                i += consumed + 1;
                            } else {
                                i += 1;
                            }
                        } else {
                            i += 1;
                        }

                        arr.push(Yaml::Hash(child_map));
                        continue;
                    }

                    // Case 3: Simple scalar array element
                    arr.push(Yaml::Value(val_str.to_string()));
                    i += 1;
                    continue;
                }

                // --- REGULAR KEY: VALUE ---
                if let Some(idx) = trimmed.find(':') {
                    let key = trimmed[..idx].trim().to_string();
                    let val_str = trimmed[idx + 1..].trim();

                    #[cfg(debug_assertions)]
                    println!("🧾 Line {} (indent {}): Key '{}' => '{}'", i, indent, key, val_str);

                    if val_str.is_empty() {
                        #[cfg(debug_assertions)]
                        println!("  ↳ Nested block detected for key '{}'", key);
                        let (child, consumed) = parse_block(&lines[i + 1..], indent + 2);
                        map.insert(key, child);
                        i += consumed + 1;
                    } else if val_str.starts_with('{') && val_str.ends_with('}') {
                        map.insert(key, parse_inline_mapping(val_str));
                        i += 1;
                    } else {
                        map.insert(key, Yaml::Value(val_str.to_string()));
                        i += 1;
                    }
                    continue;
                }

                #[cfg(debug_assertions)]
                println!("⚠️  Unrecognized line {}: '{}'", i, trimmed);
                i += 1;
            }

            let arr_len= arr.len();
            let hash_len = map.len();

            let ret_arr = Yaml::Array(arr);
            let ret_hash = Yaml::Hash(map);

            #[cfg(debug_assertions)]
            {
            println!(
                "🏁 EXIT block (indent {}) as {} with {} entries",
                start_indent,
                if is_array { "Array" } else { "Hash" },
                if is_array { arr_len } else { hash_len }
            );
            println!("And we collected the hash as:\n{}\nand the array as \n{}", ret_hash, ret_arr );
            }
            if is_array {
                (ret_arr, i)
            } else {
                (ret_hash, i)
            }
        }

        fn parse_inline_mapping(s: &str) -> Yaml {
            let mut map = HashMap::new();
            let inner = s.strip_prefix('{').and_then(|v| v.strip_suffix('}')).unwrap_or(s);

            #[cfg(debug_assertions)]
            println!("🧩 Inline mapping: {}", inner);

            for part in inner.split(',') {
                let kv: Vec<&str> = part.splitn(2, ':').collect();
                if kv.len() == 2 {
                    let k = kv[0].trim().to_string();
                    let v = kv[1].trim().to_string();
                    map.insert(k.clone(), Yaml::Value(v.clone()));

                    #[cfg(debug_assertions)]
                    println!("   ↳ Inline pair {}: {}", k, v);
                }
            }
            Yaml::Hash(map)
        }

        #[cfg(debug_assertions)]
        println!("🚀 Starting YAML parse of {} lines", lines.len());

        let (yaml, _) = parse_block(lines, 0);

        #[cfg(debug_assertions)]
        println!("✅ Completed top-level parse");

        yaml
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::fs;

    #[test]
    fn test_yaml_array_save_and_load_roundtrip() {
        use std::collections::HashMap;
        use std::fs;

        // 🧩 1️⃣ Build an array inside a hash
        let array = Yaml::Array(vec![
            Yaml::Value("one".to_string()),
            Yaml::Value("two".to_string()),
            Yaml::Value("three".to_string()),
        ]);

        let mut outer = HashMap::new();
        outer.insert("values".to_string(), array);
        let yaml = Yaml::Hash(outer);

        // 2️⃣ Write to temp file
        let tmp_path = std::env::temp_dir().join("yaml_array_test.yaml");
        yaml.save_to_file(&tmp_path).expect("save_to_file failed");

        // 3️⃣ Verify file exists and contains YAML array markers
        let content = fs::read_to_string(&tmp_path).expect("failed to read file");
        assert!(content.contains("- one"), "YAML file missing array items");
        assert!(content.contains("values:"), "YAML file missing array key");

        // 4️⃣ Load it back — should succeed without panicking
        let loaded = Yaml::load_from_file(&tmp_path).expect("load_from_file failed");

        // 5️⃣ Check structure: expect hash → array → 3 values
        if let Yaml::Hash(map) = loaded {
            let arr = map.get("values").expect("missing key 'values'");
            if let Yaml::Array(items) = arr {
                assert_eq!(items.len(), 3, "expected 3 items in array");
                assert_eq!(items[0].get_val(), Some("one"));
                assert_eq!(items[2].get_val(), Some("three"));
            } else {
                panic!("expected 'values' to be a Yaml::Array");
            }
        } else {
            panic!("expected root Yaml::Hash");
        }

        // 6️⃣ Cleanup
        fs::remove_file(&tmp_path).ok();
    }
    #[test]
    fn test_yaml_save_and_load_roundtrip() {
        // 🧩 1. Create a small test YAML structure
        let mut map = HashMap::new();
        map.insert("guidA".to_string(), Yaml::Value("res://textures/UI/Button.png".to_string()));
        map.insert("guidB".to_string(), Yaml::Value("res://materials/Metal.tres".to_string()));
        let yaml = Yaml::Hash(map);

        // 2️⃣ Write to temp file
        let tmp_path = std::env::temp_dir().join("yaml_basic_test.yaml");
        yaml.save_to_file(&tmp_path).expect("save_to_file failed");

        // 3️⃣ Check file exists and contains expected substring
        let content = fs::read_to_string(&tmp_path).expect("failed to read file");
        assert!(content.contains("guidA"), "saved YAML missing key");
        assert!(content.contains("res://textures/UI/Button.png"), "saved YAML missing value");

        // 4️⃣ Try loading it back (just check no error)
        let loaded = Yaml::load_from_file(&tmp_path).expect("load_from_file failed");
        let loaded_text = format!("{}", loaded);
        assert!(loaded_text.contains("guidA"), "loaded YAML missing key");

        // 5️⃣ Clean up
        fs::remove_file(&tmp_path).ok();
    }

    #[test]
    fn test_parse_modifications_array() {
        let yaml_text = r#"
m_TransformParent: {fileID: 0}
m_Modifications:
  - target: {fileID: 8455400915583205629, guid: c9c31f173b4e3274385d017b2f88d207, type: 3}
    propertyPath: m_Name
    value: Canvas
    objectReference: {fileID: 0}
  - target: {fileID: 8455400915583205625, guid: c9c31f173b4e3274385d017b2f88d207, type: 3}
    propertyPath: m_LocalPosition.x
    value: 0
    objectReference: {fileID: 0}
  - target: {fileID: 8455400915583205625, guid: c9c31f173b4e3274385d017b2f88d207, type: 3}
    propertyPath: m_LocalPosition.y
    value: 0
    objectReference: {fileID: 0}
"#;

        let lines: Vec<&str> = yaml_text.lines().collect();
        let parsed = Yaml::parse_unity_object(&lines);
        let parsed_str = format!("{}", parsed);
        // Check that top-level is a Hash
        if let Yaml::Hash(map) = parsed {
            // m_TransformParent is present
            assert!(map.contains_key("m_TransformParent"));

            // m_Modifications is an Array
            if let Yaml::Array(arr) = &map["m_Modifications"] {
                assert_eq!(arr.len(), 3);

                // Each element should be a Hash with expected keys
                for element in arr {
                    if let Yaml::Hash(el_map) = element {
                        assert!(el_map.contains_key("target"));
                        assert!(el_map.contains_key("propertyPath"));
                        assert!(el_map.contains_key("value"));
                        assert!(el_map.contains_key("objectReference"));
                    } else {
                        panic!("Array element is not a Hash! {}", parsed_str);
                    }
                }
            } else {
                panic!("m_Modifications is not an Array! {parsed_str}");
            }
        } else {
            panic!("Top-level YAML is not a Hash! {parsed_str}");
        }
    }

    #[test]
    fn test_get_str_from_value() {
        let yaml = Yaml::Value("hello".to_string());
        assert_eq!(yaml.get_str("hello"), None); // not a hash, so no key lookup
    }
    #[test]
    fn test_get_str_from_array() {
        let yaml = Yaml::Value("hello".to_string());
        let arr = Yaml::Array(vec![yaml]);
        assert_eq!(arr.get_str("hello"), None); // not a hash, so no key lookup
    }


    #[test]
    fn test_get_str_from_hash() {
        let mut map = HashMap::new();
        map.insert("name".to_string(), Yaml::Value("Alice".to_string()));
        let yaml = Yaml::Hash(map);

        assert_eq!(yaml.get_str("name"), Some("Alice"));
        assert_eq!(yaml.get_str("missing"), None);
    }
}
