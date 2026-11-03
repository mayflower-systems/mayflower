//! Turn an Anchor IDL into client code that does not depend on Anchor.
//!
//! [`parse_anchor_idl`] converts IDL JSON into [`ProgramData`], a model with no
//! Anchor or target-language types. A materializer, such as
//! [`materialization::rust::materialize`] or
//! [`materialization::typescript::materialize`], turns that model into source
//! files held in memory. Neither step reads or writes files or starts processes.

#![forbid(unsafe_code)]

use anchor_lang_idl::types::Idl;
use anyhow::{Context, Result};

mod anchor_idl;
pub mod materialization;
pub mod program_data;
#[cfg(test)]
mod test_support;

pub use materialization::rust::RustFileBundle;
pub use materialization::typescript::TypeScriptFileBundle;
pub use program_data::ProgramData;

/// The input to [`parse_anchor_idl`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenerateConfig {
    /// The complete Anchor IDL JSON document.
    pub idl_json: String,
    /// The UpperCamelCase prefix for generated type names, such as `Mayflower`.
    pub program_name: String,
}

/// Parse Anchor IDL JSON and convert it into [`ProgramData`].
pub fn parse_anchor_idl(config: GenerateConfig) -> Result<ProgramData> {
    let idl: Idl = serde_json::from_str(&config.idl_json).context("invalid Anchor IDL JSON")?;
    anchor_idl::convert(idl, config.program_name)
}

/// Run [`parse_anchor_idl`], then the Rust materializer.
pub fn generate(config: GenerateConfig) -> Result<RustFileBundle> {
    let program = parse_anchor_idl(config)?;
    materialization::rust::materialize(&program)
}

/// Run [`parse_anchor_idl`], then the TypeScript materializer.
pub fn generate_typescript(config: GenerateConfig) -> Result<TypeScriptFileBundle> {
    let program = parse_anchor_idl(config)?;
    materialization::typescript::materialize(&program)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{sample_idl, sample_program};

    fn sample_config(program_name: &str) -> GenerateConfig {
        GenerateConfig {
            idl_json: serde_json::to_string(&sample_idl()).unwrap(),
            program_name: program_name.to_owned(),
        }
    }

    #[test]
    fn parses_idl_json_into_the_same_program_data_as_the_idl_value() {
        let program = parse_anchor_idl(sample_config("Example")).unwrap();

        assert_eq!(program, sample_program());
    }

    #[test]
    fn client_name_controls_program_specific_type_names() {
        let generated = materialization::rust::materialize(&sample_program()).unwrap();

        let pda = String::from_utf8(generated.pda_rs).unwrap();
        let instructions = String::from_utf8(generated.instructions_rs).unwrap();
        let errors = String::from_utf8(generated.errors_rs).unwrap();
        let lib = String::from_utf8(generated.lib_rs).unwrap();

        assert!(!pda.contains("PROGRAM_ID"));
        assert!(pda.contains("pub trait ExamplePda"));
        assert!(instructions.contains("pub trait ExampleInstructions: ExamplePda"));
        assert!(errors.contains("pub enum ExampleError"));
        assert!(lib.contains("pub struct ExampleApi"));
        assert!(lib.contains("pub use instructions::ExampleInstructions;"));
    }

    #[test]
    fn rust_output_contains_exactly_the_complete_module_set() {
        let generated = generate(sample_config("Example")).unwrap();

        assert_eq!(
            generated.files().map(|(name, _)| name),
            [
                "lib.rs",
                "accounts.rs",
                "errors.rs",
                "events.rs",
                "instructions.rs",
                "models.rs",
                "pda.rs",
            ]
        );
        assert!(generated.files().iter().all(|(_, bytes)| !bytes.is_empty()));
    }

    #[test]
    fn rejects_invalid_json_and_program_names() {
        let invalid_json = generate(GenerateConfig {
            idl_json: "not json".to_owned(),
            program_name: "Example".to_owned(),
        })
        .unwrap_err();
        assert!(invalid_json.to_string().contains("invalid Anchor IDL JSON"));

        let invalid_name = generate(sample_config("example-name")).unwrap_err();
        assert!(invalid_name.to_string().contains("UpperCamelCase"));
    }
}
