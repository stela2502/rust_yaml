
use crate::yaml::Yaml;
use std::any::type_name;

/// Core translation trait between Unity YAML and Godot TSCN
pub trait UnityValue: Sized {
    /// Convert this Unity type into a Godot representation (text fragment or node)
    fn to_godot(&self) -> String;

    /// Try to construct this Unity type from a YAML node.
    /// Returns `None` if parsing fails or fields are missing.
    fn from_yaml(yaml: &Yaml) -> Option<Self>;

    // like Python’s type(obj) or C#’s obj.GetType()
    fn type_name(&self) -> &'static str {
        type_name::<Self>()
    }
}