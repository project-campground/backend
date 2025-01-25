use lazy_static::lazy_static;

lazy_static! {
    pub static ref CONSTRAINTS: types::Constraints = {
        let toml_str = include_str!("constraints.toml");
        toml::from_str(toml_str).expect("Failed to deserialize constraints.toml")
    };
}

pub mod types;