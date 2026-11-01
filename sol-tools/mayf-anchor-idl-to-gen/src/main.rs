//! Build Anchor IDLs and generate Anchor-free clients from them.
//!
//! The library does the conversion in memory. This binary owns every side
//! effect: it reads and writes files, and `build-idl` runs `cargo`.

use std::fs;
use std::path::{Path, PathBuf};

use anchor_lang_idl::build::IdlBuilder;
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use mayf_anchor_idl_to_gen::{
    GenerateConfig,
    materialization::{rust, typescript},
    parse_anchor_idl,
};

/// Build an Anchor IDL or generate an anchor-free client from one.
#[derive(Debug, Parser)]
#[command(name = "anchor-idl-to-gen", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Build an Anchor IDL JSON document from a program crate.
    BuildIdl {
        /// Path to the Anchor program crate.
        #[arg(value_name = "PROGRAM_DIR")]
        program_dir: PathBuf,

        /// File where the generated IDL JSON will be written.
        #[arg(long, value_name = "IDL_PATH")]
        output: PathBuf,
    },

    /// Generate anchor-free Rust bindings from an Anchor IDL JSON document.
    GenerateRust {
        /// Path to the Anchor IDL JSON document.
        #[arg(value_name = "IDL_PATH")]
        idl_path: PathBuf,

        /// UpperCamelCase prefix for generated Rust types, such as Mayflower.
        #[arg(long, value_name = "NAME")]
        program_name: String,

        /// Directory where the seven generated Rust files will be written.
        #[arg(long, value_name = "DIR")]
        output_dir: PathBuf,

        /// Treat PDA accounts as caller-supplied and preserve an existing pda.rs.
        #[arg(long)]
        manual_pdas: bool,
    },

    /// Generate Anchor-free TypeScript bindings from an Anchor IDL JSON document.
    GenerateTypescript {
        /// Path to the Anchor IDL JSON document.
        #[arg(value_name = "IDL_PATH")]
        idl_path: PathBuf,

        /// UpperCamelCase prefix for generated TypeScript types, such as Mayflower.
        #[arg(long, value_name = "NAME")]
        program_name: String,

        /// Directory where the eight generated TypeScript files will be written.
        #[arg(long, value_name = "DIR")]
        output_dir: PathBuf,

        /// Treat PDA accounts as caller-supplied and preserve an existing pda.ts.
        #[arg(long)]
        manual_pdas: bool,
    },
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::BuildIdl {
            program_dir,
            output,
        } => build_idl(&program_dir, &output),
        Command::GenerateRust {
            idl_path,
            program_name,
            output_dir,
            manual_pdas,
        } => generate_rust(&idl_path, program_name, &output_dir, manual_pdas),
        Command::GenerateTypescript {
            idl_path,
            program_name,
            output_dir,
            manual_pdas,
        } => generate_typescript(&idl_path, program_name, &output_dir, manual_pdas),
    }
}

fn build_idl(program_dir: &Path, output: &Path) -> Result<()> {
    // anchor-lang-idl 0.1.4 passes the literal argument `+{toolchain}` to cargo
    // whenever RUSTUP_TOOLCHAIN is set, and cargo rejects it. Clearing the
    // variable lets rustup pick the toolchain from rust-toolchain.toml.
    // SAFETY: the process is still single-threaded here, so no other thread
    // can read the environment while it changes.
    unsafe { std::env::remove_var("RUSTUP_TOOLCHAIN") };

    let idl = IdlBuilder::new()
        .program_path(program_dir.to_owned())
        .cargo_args(vec!["--locked".to_owned()])
        .build()
        .with_context(|| format!("failed to build the IDL for {}", program_dir.display()))?;
    let idl_json = format!("{}\n", serde_json::to_string_pretty(&idl)?);

    write(output, idl_json.as_bytes())?;
    println!("wrote {}", output.display());
    Ok(())
}

fn generate_rust(
    idl_path: &Path,
    program_name: String,
    output_dir: &Path,
    manual_pdas: bool,
) -> Result<()> {
    let idl_json = fs::read_to_string(idl_path)
        .with_context(|| format!("failed to read {}", idl_path.display()))?;
    let mut program = parse_anchor_idl(GenerateConfig {
        idl_json,
        program_name,
    })?;
    if manual_pdas {
        program.clear_pdas();
    }
    let generated = rust::materialize(&program)?;

    for (filename, bytes) in generated.files() {
        if manual_pdas && filename == "pda.rs" {
            continue;
        }
        write(&output_dir.join(filename), bytes)?;
    }
    println!("wrote generated Rust bindings to {}", output_dir.display());
    Ok(())
}

fn generate_typescript(
    idl_path: &Path,
    program_name: String,
    output_dir: &Path,
    manual_pdas: bool,
) -> Result<()> {
    let idl_json = fs::read_to_string(idl_path)
        .with_context(|| format!("failed to read {}", idl_path.display()))?;
    let mut program = parse_anchor_idl(GenerateConfig {
        idl_json,
        program_name,
    })?;
    if manual_pdas {
        program.clear_pdas();
    }
    let generated = typescript::materialize(&program)?;

    for (filename, bytes) in generated.files() {
        if manual_pdas && filename == "pda.ts" {
            continue;
        }
        write(&output_dir.join(filename), bytes)?;
    }
    println!(
        "wrote generated TypeScript bindings to {}",
        output_dir.display()
    );
    Ok(())
}

fn write(path: &Path, contents: &[u8]) -> Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, contents).with_context(|| format!("failed to write {}", path.display()))
}
