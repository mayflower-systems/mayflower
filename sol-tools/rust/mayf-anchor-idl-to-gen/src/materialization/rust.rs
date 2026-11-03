//! Materializes language-neutral program data as anchor-free Rust modules.
//!
//! The output is plain Rust over `borsh`, `solana-address`, and
//! `solana-instruction`. Every fact it encodes — discriminators, PDA seeds,
//! account layouts, error codes — comes from the input [`ProgramData`].

// Format strings here are the emitted text, so their newlines are load-bearing
// and belong in the string rather than in the choice of macro.
#![allow(clippy::write_with_newline)]

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::Write as _;

use anyhow::{Context, Result, bail};
use solana_address::Address;

use crate::program_data::{
    ArrayLength, DefinedFields, Field, Instruction, InstructionAccount, InstructionAccountItem,
    ProgramData, Seed, Type, TypeDefinition, TypeDefinitionKind,
};

/// One PDA derivation: an account name plus the seeds that derive it.
struct PdaDerivation<'data> {
    /// Name of the generated function.
    function: String,
    /// Account whose address this derives.
    account: &'data str,
    /// Instruction whose argument types define any argument seeds.
    instruction: &'data Instruction,
    /// Named types used to resolve fields below a defined instruction argument.
    types: &'data [TypeDefinition],
    seeds: &'data [Seed],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SeedParameter<'data> {
    Account(&'data str),
    U32Argument(&'data str),
    FixedBytesArgument { path: &'data str, length: usize },
}

/// An account whose address the IDL fixes, emitted as a constant.
struct FixedAddress {
    constant: String,
    address: [u8; 32],
}

/// Rust source files produced for one program client.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RustFileBundle {
    pub lib_rs: Vec<u8>,
    pub errors_rs: Vec<u8>,
    pub events_rs: Vec<u8>,
    pub instructions_rs: Vec<u8>,
    pub accounts_rs: Vec<u8>,
    pub pda_rs: Vec<u8>,
    pub models_rs: Vec<u8>,
}

impl RustFileBundle {
    /// Every generated file in stable filename order.
    pub fn files(&self) -> [(&'static str, &[u8]); 7] {
        [
            ("lib.rs", &self.lib_rs),
            ("accounts.rs", &self.accounts_rs),
            ("errors.rs", &self.errors_rs),
            ("events.rs", &self.events_rs),
            ("instructions.rs", &self.instructions_rs),
            ("models.rs", &self.models_rs),
            ("pda.rs", &self.pda_rs),
        ]
    }
}

struct RenderedRustFiles {
    pub lib: String,
    pub accounts: String,
    pub errors: String,
    pub events: String,
    pub instructions: String,
    pub models: String,
    pub pda: String,
}

/// Materialize a complete Rust client without performing filesystem IO.
pub fn materialize(program: &ProgramData) -> Result<RustFileBundle> {
    validate_program_name(&program.name)?;
    validate_discriminators(program)?;

    let rendered = render(program, &program.name)?;

    Ok(RustFileBundle {
        lib_rs: rendered.lib.into_bytes(),
        errors_rs: rendered.errors.into_bytes(),
        events_rs: rendered.events.into_bytes(),
        instructions_rs: rendered.instructions.into_bytes(),
        accounts_rs: rendered.accounts.into_bytes(),
        pda_rs: rendered.pda.into_bytes(),
        models_rs: rendered.models.into_bytes(),
    })
}

fn render(program: &ProgramData, program_name: &str) -> Result<RenderedRustFiles> {
    let api_name = format!("{program_name}Api");
    let pda_trait_name = format!("{program_name}Pda");
    let instruction_trait_name = format!("{program_name}Instructions");
    let error_name = format!("{program_name}Error");
    let types = program
        .types
        .iter()
        .map(|def| (def.name.as_str(), def))
        .collect::<HashMap<_, _>>();
    let seed_constants = seed_constants(program)?;
    let pdas = pdas(program)?;
    let fixed = fixed_addresses(program);

    let account_names = program
        .accounts
        .iter()
        .map(|account| account.name.as_str())
        .collect::<BTreeSet<_>>();
    let event_names = program
        .events
        .iter()
        .map(|event| event.name.as_str())
        .collect::<BTreeSet<_>>();

    let mut accounts = String::new();
    header(&mut accounts);
    accounts.push_str(
        "use borsh::{BorshDeserialize, BorshSerialize};\n\
         use solana_address::Address;\n\
         \n\
         use crate::AccountData;\n\
         #[allow(unused_imports)] // Some programs' accounts use only primitive fields.\n\
         use crate::models::*;\n",
    );
    type_defs(&mut accounts, program, &types, |name| {
        account_names.contains(name)
    })?;
    account_impls(&mut accounts, program);

    let mut events = String::new();
    header(&mut events);
    if !program.events.is_empty() {
        events.push_str(
            "use borsh::{BorshDeserialize, BorshSerialize};\n\
             use solana_address::Address;\n\
             \n\
             use crate::EventData;\n\
             #[allow(unused_imports)] // Some programs' events use only primitive fields.\n\
             use crate::models::*;\n",
        );
    }
    type_defs(&mut events, program, &types, |name| {
        event_names.contains(name)
    })?;
    event_impls(&mut events, program);

    let mut models = String::new();
    header(&mut models);
    models.push_str(
        "use borsh::{BorshDeserialize, BorshSerialize};\n\
         #[allow(unused_imports)] // Some programs' models use only primitive fields.\n\
         use solana_address::Address;\n",
    );
    type_defs(&mut models, program, &types, |name| {
        !account_names.contains(name) && !event_names.contains(name)
    })?;

    let mut pda = String::new();
    header(&mut pda);
    if !pdas.is_empty() {
        pda.push_str("use solana_address::Address;\n\n");
    }
    pda.push_str("use crate::ProgramAddress;\n");
    seeds(&mut pda, &seed_constants);
    pda_trait(&mut pda, &pdas, &seed_constants, &pda_trait_name)?;

    let mut instructions = String::new();
    header(&mut instructions);
    let _ = write!(
        instructions,
        "use borsh::{{BorshDeserialize, BorshSerialize}};\n\
         use solana_address::{{Address, address}};\n\
         use solana_instruction::{{AccountMeta, Instruction}};\n\
         \n\
         use crate::{{InstructionArgs, {pda_trait_name}, models::*}};\n"
    );
    fixed_address_constants(&mut instructions, &fixed);
    instruction_args(&mut instructions, program, &types)?;
    instruction_catalog(&mut instructions, program);
    instruction_account_structs(&mut instructions, program, &pdas, &fixed, &pda_trait_name)?;
    instruction_trait(
        &mut instructions,
        program,
        &pda_trait_name,
        &instruction_trait_name,
    )?;

    let mut errors_source = String::new();
    header(&mut errors_source);
    errors(&mut errors_source, program, &error_name);

    Ok(RenderedRustFiles {
        lib: client_lib(
            program_name,
            &api_name,
            &instruction_trait_name,
            !program.events.is_empty(),
        ),
        accounts,
        errors: errors_source,
        events,
        instructions,
        models,
        pda,
    })
}

fn client_lib(
    program_name: &str,
    api_name: &str,
    instruction_trait_name: &str,
    has_events: bool,
) -> String {
    include_str!("../client_lib.rs.template")
        .replace("__PROGRAM_NAME__", program_name)
        .replace("__PROGRAM_API__", api_name)
        .replace("__PROGRAM_INSTRUCTIONS__", instruction_trait_name)
        .replace(
            "__PROGRAM_EVENTS_REEXPORT__",
            if has_events { "pub use events::*;" } else { "" },
        )
}

fn validate_program_name(name: &str) -> Result<()> {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        bail!("program name must not be empty");
    };
    if !first.is_ascii_uppercase() || !chars.all(|char| char.is_ascii_alphanumeric()) {
        bail!("program name must be an UpperCamelCase Rust type prefix, such as Mayflower");
    }
    Ok(())
}

fn validate_discriminators(program: &ProgramData) -> Result<()> {
    let mut instruction_discriminators = BTreeMap::new();
    for instruction in &program.instructions {
        if let Some(previous) = instruction_discriminators.insert(
            instruction.discriminator.as_slice(),
            instruction.name.as_str(),
        ) {
            bail!(
                "duplicate instruction discriminator for {previous} and {}",
                instruction.name
            );
        }
    }
    for (kind, name, discriminator) in program
        .instructions
        .iter()
        .map(|item| {
            (
                "instruction",
                item.name.as_str(),
                item.discriminator.as_slice(),
            )
        })
        .chain(
            program
                .accounts
                .iter()
                .map(|item| ("account", item.name.as_str(), item.discriminator.as_slice())),
        )
        .chain(
            program
                .events
                .iter()
                .map(|item| ("event", item.name.as_str(), item.discriminator.as_slice())),
        )
    {
        if discriminator.len() != 8 {
            bail!(
                "Rust materialization requires an 8-byte discriminator for {kind} {name}; got {} bytes",
                discriminator.len()
            );
        }
    }
    Ok(())
}

fn header(out: &mut String) {
    out.push_str(
        "//! Generated from an Anchor IDL. Do not edit.\n\
         \n\
         #![allow(clippy::too_many_arguments)]\n",
    );
}

fn section(out: &mut String, title: &str) {
    let rule = "-".repeat(70_usize.saturating_sub(title.len() + 4));
    let _ = write!(out, "\n// {title} {rule}\n\n");
}

// ---------------------------------------------------------------- seeds

/// Byte-valued IDL constants, which are the program's PDA seed prefixes.
fn seed_constants(program: &ProgramData) -> Result<BTreeMap<Vec<u8>, String>> {
    let mut out = BTreeMap::new();
    for constant in &program.constants {
        if !matches!(constant.ty, Type::Bytes) {
            continue;
        }
        let bytes = parse_byte_literal(&constant.value)
            .with_context(|| format!("constant {}", constant.name))?;
        out.insert(bytes, constant.name.clone());
    }
    Ok(out)
}

/// Parse the `"[103, 114, 111, 117, 112]"` form the IDL stores constants in.
fn parse_byte_literal(value: &str) -> Result<Vec<u8>> {
    let inner = value
        .trim()
        .strip_prefix('[')
        .and_then(|rest| rest.strip_suffix(']'))
        .with_context(|| format!("expected a byte array, got {value}"))?;
    if inner.trim().is_empty() {
        return Ok(Vec::new());
    }
    inner
        .split(',')
        .map(|byte| byte.trim().parse::<u8>().map_err(anyhow::Error::from))
        .collect()
}

fn seeds(out: &mut String, seed_constants: &BTreeMap<Vec<u8>, String>) {
    if seed_constants.is_empty() {
        return;
    }
    section(out, "seeds");
    for (bytes, name) in seed_constants {
        let _ = write!(
            out,
            "pub const {name}: &[u8] = {};\n",
            byte_string_literal(bytes)
        );
    }
}

/// `b"market"` when the bytes are printable ASCII, otherwise a byte array.
fn byte_string_literal(bytes: &[u8]) -> String {
    if !bytes.is_empty()
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
    {
        format!("b\"{}\"", String::from_utf8_lossy(bytes))
    } else {
        format!(
            "&[{}]",
            bytes
                .iter()
                .map(u8::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

// ---------------------------------------------------------------- fixed addresses

fn fixed_addresses(program: &ProgramData) -> Vec<FixedAddress> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for account in program.instructions.iter().flat_map(instruction_accounts) {
        let Some(address) = &account.address else {
            continue;
        };
        if seen.insert(*address) {
            out.push(FixedAddress {
                constant: format!("{}_ID", account.name.to_uppercase()),
                address: *address,
            });
        }
    }
    out
}

fn fixed_address_constants(out: &mut String, fixed: &[FixedAddress]) {
    if fixed.is_empty() {
        return;
    }
    section(out, "fixed addresses");
    for entry in fixed {
        let address = Address::from(entry.address);
        let _ = write!(
            out,
            "pub const {}: Address = address!(\"{address}\");\n",
            entry.constant
        );
    }
}

/// The constant name for an account the IDL pins to a fixed address.
fn fixed_constant_for<'a>(fixed: &'a [FixedAddress], address: &[u8; 32]) -> Option<&'a str> {
    fixed
        .iter()
        .find(|entry| entry.address == *address)
        .map(|entry| entry.constant.as_str())
}

// ---------------------------------------------------------------- types

fn type_defs(
    out: &mut String,
    program: &ProgramData,
    types: &HashMap<&str, &TypeDefinition>,
    include: impl Fn(&str) -> bool,
) -> Result<()> {
    if !program.types.iter().any(|def| include(&def.name)) {
        return Ok(());
    }
    section(out, "types");
    for def in program.types.iter().filter(|def| include(&def.name)) {
        if !def.generic_parameters.is_empty() {
            bail!("generic type {} is not supported", def.name);
        }
        docs(out, &def.docs, "");
        let _ = write!(out, "{}\n", derives(&def.name, types));
        match &def.kind {
            TypeDefinitionKind::Struct { fields } => {
                emit_struct(out, &def.name, fields.as_ref())?;
            }
            TypeDefinitionKind::Enum { variants } => {
                let _ = write!(out, "pub enum {} {{\n", def.name);
                for variant in variants {
                    emit_enum_variant(out, variant)?;
                }
                out.push_str("}\n\n");
            }
            TypeDefinitionKind::Alias(_) => bail!("type alias {} is not supported", def.name),
        }
    }
    Ok(())
}

fn emit_enum_variant(out: &mut String, variant: &crate::program_data::EnumVariant) -> Result<()> {
    match variant.fields.as_ref() {
        None => {
            let _ = write!(out, "    {},\n", variant.name);
        }
        Some(DefinedFields::Named(fields)) => {
            let _ = write!(out, "    {} {{\n", variant.name);
            for field in fields {
                docs(out, &field.docs, "        ");
                let _ = write!(out, "        {}: {},\n", field.name, rust_type(&field.ty)?);
            }
            out.push_str("    },\n");
        }
        Some(DefinedFields::Tuple(fields)) => {
            let rendered = fields.iter().map(rust_type).collect::<Result<Vec<_>>>()?;
            let _ = write!(out, "    {}({}),\n", variant.name, rendered.join(", "));
        }
    }
    Ok(())
}

fn emit_struct(out: &mut String, name: &str, fields: Option<&DefinedFields>) -> Result<()> {
    match fields {
        None => {
            let _ = write!(out, "pub struct {name};\n\n");
        }
        Some(DefinedFields::Named(named)) if named.is_empty() => {
            let _ = write!(out, "pub struct {name};\n\n");
        }
        Some(DefinedFields::Named(named)) => {
            let _ = write!(out, "pub struct {name} {{\n");
            for field in named {
                docs(out, &field.docs, "    ");
                let _ = write!(out, "    pub {}: {},\n", field.name, rust_type(&field.ty)?);
            }
            out.push_str("}\n\n");
        }
        Some(DefinedFields::Tuple(tuple)) => {
            let rendered = tuple
                .iter()
                .map(|ty| rust_type(ty).map(|ty| format!("pub {ty}")))
                .collect::<Result<Vec<_>>>()?;
            let _ = write!(out, "pub struct {name}({});\n\n", rendered.join(", "));
        }
    }
    Ok(())
}

fn rust_type(ty: &Type) -> Result<String> {
    Ok(match ty {
        Type::Bool => "bool".into(),
        Type::U8 => "u8".into(),
        Type::I8 => "i8".into(),
        Type::U16 => "u16".into(),
        Type::I16 => "i16".into(),
        Type::U32 => "u32".into(),
        Type::I32 => "i32".into(),
        Type::U64 => "u64".into(),
        Type::I64 => "i64".into(),
        Type::U128 => "u128".into(),
        Type::I128 => "i128".into(),
        Type::F32 => "f32".into(),
        Type::F64 => "f64".into(),
        Type::Bytes => "Vec<u8>".into(),
        Type::String => "String".into(),
        Type::PublicKey => "Address".into(),
        Type::Option(inner) => format!("Option<{}>", rust_type(inner)?),
        Type::Vec(inner) => format!("Vec<{}>", rust_type(inner)?),
        Type::Array(inner, len) => match len {
            ArrayLength::Value(len) => format!("[{}; {len}]", rust_type(inner)?),
            ArrayLength::Generic(name) => bail!("generic array length {name} is not supported"),
        },
        Type::Defined {
            name,
            generic_arguments,
        } => {
            if !generic_arguments.is_empty() {
                bail!("generic arguments on {name} are not supported");
            }
            name.clone()
        }
        other => bail!("IDL type {other:?} is not supported"),
    })
}

/// Derive list for a type, widened to whatever its fields actually support.
fn derives(name: &str, types: &HashMap<&str, &TypeDefinition>) -> String {
    let mut list = vec!["BorshSerialize", "BorshDeserialize", "Clone"];
    if trait_holds(name, types, &TraitCheck::Copy) {
        list.push("Copy");
    }
    list.push("Debug");
    list.push("PartialEq");
    if trait_holds(name, types, &TraitCheck::Eq) {
        list.push("Eq");
    }
    format!("#[derive({})]", list.join(", "))
}

enum TraitCheck {
    Copy,
    Eq,
}

fn trait_holds(name: &str, types: &HashMap<&str, &TypeDefinition>, check: &TraitCheck) -> bool {
    let mut visiting = BTreeSet::new();
    type_def_holds(name, types, check, &mut visiting)
}

fn type_def_holds<'a>(
    name: &'a str,
    types: &HashMap<&'a str, &'a TypeDefinition>,
    check: &TraitCheck,
    visiting: &mut BTreeSet<&'a str>,
) -> bool {
    if !visiting.insert(name) {
        // A cycle can only exist behind an indirection, which is not Copy.
        return matches!(check, TraitCheck::Eq);
    }
    let Some(def) = types.get(name) else {
        return false;
    };
    let holds = match &def.kind {
        TypeDefinitionKind::Struct { fields } => {
            defined_fields_hold(fields.as_ref(), types, check, visiting)
        }
        TypeDefinitionKind::Enum { variants } => variants
            .iter()
            .all(|variant| defined_fields_hold(variant.fields.as_ref(), types, check, visiting)),
        TypeDefinitionKind::Alias(alias) => type_holds(alias, types, check, visiting),
    };
    visiting.remove(name);
    holds
}

fn defined_fields_hold<'a>(
    fields: Option<&'a DefinedFields>,
    types: &HashMap<&'a str, &'a TypeDefinition>,
    check: &TraitCheck,
    visiting: &mut BTreeSet<&'a str>,
) -> bool {
    match fields {
        None => true,
        Some(DefinedFields::Named(named)) => named
            .iter()
            .all(|field| type_holds(&field.ty, types, check, visiting)),
        Some(DefinedFields::Tuple(tuple)) => tuple
            .iter()
            .all(|ty| type_holds(ty, types, check, visiting)),
    }
}

fn type_holds<'a>(
    ty: &'a Type,
    types: &HashMap<&'a str, &'a TypeDefinition>,
    check: &TraitCheck,
    visiting: &mut BTreeSet<&'a str>,
) -> bool {
    match ty {
        Type::F32 | Type::F64 => matches!(check, TraitCheck::Copy),
        Type::Bytes | Type::String | Type::Vec(_) => matches!(check, TraitCheck::Eq),
        Type::Option(inner) | Type::Array(inner, _) => type_holds(inner, types, check, visiting),
        Type::Defined { name, .. } => type_def_holds(name, types, check, visiting),
        Type::Generic(_) => false,
        _ => true,
    }
}

fn docs(out: &mut String, docs: &[String], indent: &str) {
    for line in docs {
        let _ = write!(out, "{indent}/// {line}\n");
    }
}

// ---------------------------------------------------------------- accounts

fn account_impls(out: &mut String, program: &ProgramData) {
    if program.accounts.is_empty() {
        return;
    }
    section(out, "account discriminators");
    for account in &program.accounts {
        let _ = write!(
            out,
            "impl AccountData for {} {{\n    const DISCRIMINATOR: [u8; 8] = {:?};\n}}\n\n",
            account.name, account.discriminator
        );
    }
}

// ---------------------------------------------------------------- events

fn event_impls(out: &mut String, program: &ProgramData) {
    if program.events.is_empty() {
        return;
    }
    section(out, "event discriminators");
    for event in &program.events {
        let _ = write!(
            out,
            "impl EventData for {} {{\n    const DISCRIMINATOR: [u8; 8] = {:?};\n}}\n\n",
            event.name, event.discriminator
        );
    }
}

// ---------------------------------------------------------------- pdas

fn instruction_accounts(ix: &Instruction) -> Vec<&InstructionAccount> {
    fn walk<'a>(items: &'a [InstructionAccountItem], out: &mut Vec<&'a InstructionAccount>) {
        for item in items {
            match item {
                InstructionAccountItem::Single(account) => out.push(account),
                InstructionAccountItem::Composite(group) => walk(&group.accounts, out),
            }
        }
    }
    let mut out = Vec::new();
    walk(&ix.accounts, &mut out);
    out
}

/// One generated function per distinct (account name, seed list). When a name
/// derives differently in different instructions, the instruction disambiguates.
fn pdas(program: &ProgramData) -> Result<Vec<PdaDerivation<'_>>> {
    let mut by_name: BTreeMap<&str, Vec<(&Instruction, &[Seed])>> = BTreeMap::new();
    for ix in &program.instructions {
        for account in instruction_accounts(ix) {
            let Some(pda) = &account.pda else { continue };
            if pda.program.is_some() {
                bail!(
                    "account {} derives from another program, which is not supported",
                    account.name
                );
            }
            let variants = by_name.entry(account.name.as_str()).or_default();
            if !variants
                .iter()
                .any(|(_, seeds)| *seeds == pda.seeds.as_slice())
            {
                variants.push((ix, pda.seeds.as_slice()));
            }
        }
    }

    let mut out = Vec::new();
    for (account, variants) in by_name {
        let ambiguous = variants.len() > 1;
        for (instruction, seeds) in variants {
            let function = if ambiguous {
                let scope = instruction
                    .name
                    .strip_prefix("initialize_")
                    .unwrap_or(&instruction.name);
                format!("{account}_pda_for_{scope}")
            } else {
                format!("{account}_pda")
            };
            out.push(PdaDerivation {
                function,
                account,
                instruction,
                types: &program.types,
                seeds,
            });
        }
    }
    Ok(out)
}

/// The PDA derivation that applies to `account` within `instruction`.
fn pda_for<'a>(
    pdas: &'a [PdaDerivation<'a>],
    instruction: &str,
    account: &str,
) -> Option<&'a PdaDerivation<'a>> {
    let candidates = pdas
        .iter()
        .filter(|pda| pda.account == account)
        .collect::<Vec<_>>();
    match candidates.as_slice() {
        [] => None,
        [only] => Some(only),
        ambiguous => {
            let scope = instruction
                .strip_prefix("initialize_")
                .unwrap_or(instruction);
            let suffix = format!("_pda_for_{scope}");
            ambiguous
                .iter()
                .copied()
                .find(|pda| pda.function.ends_with(&suffix))
        }
    }
}

/// Direct account-address seeds and supported instruction-argument seeds become
/// helper parameters. Argument paths may descend through named struct fields;
/// their terminal value must have an unambiguous Solana seed encoding.
/// One parameter per distinct seed value.
///
/// A derivation may legitimately use the same account twice — the referral
/// root node is seeded on its network and owned by it — and one value needs
/// one parameter, however many seeds read it.
fn seed_parameters<'data>(pda: &'data PdaDerivation<'data>) -> Result<Vec<SeedParameter<'data>>> {
    let mut parameters: Vec<SeedParameter<'data>> = Vec::new();
    for seed in pda.seeds {
        let parameter = match seed {
            Seed::Constant(_) => continue,
            Seed::Account { path, .. } => {
                SeedParameter::Account(direct_seed_path(path, "account")?)
            }
            Seed::Argument { path } => resolve_argument_seed(pda, path)?,
        };
        if !parameters.contains(&parameter) {
            parameters.push(parameter);
        }
    }
    Ok(parameters)
}

fn direct_seed_path<'a>(path: &'a str, kind: &str) -> Result<&'a str> {
    if path.contains('.') {
        bail!("nested PDA seed {kind} path {path} is not supported");
    }
    Ok(path)
}

fn resolve_argument_seed<'data>(
    pda: &'data PdaDerivation<'data>,
    path: &'data str,
) -> Result<SeedParameter<'data>> {
    let ty = resolve_argument_type(pda.instruction, pda.types, path)?;
    match ty {
        Type::U32 => Ok(SeedParameter::U32Argument(path)),
        Type::Array(element, ArrayLength::Value(length))
            if **element == Type::U8 && (1..=32).contains(length) =>
        {
            Ok(SeedParameter::FixedBytesArgument {
                path,
                length: *length,
            })
        }
        _ => bail!(
            "PDA seed argument {}.{path} must be u32 or a byte array of 1 through 32 bytes; got {ty:?}",
            pda.instruction.name
        ),
    }
}

fn resolve_argument_type<'data>(
    instruction: &'data Instruction,
    types: &'data [TypeDefinition],
    path: &str,
) -> Result<&'data Type> {
    let mut segments = path.split('.');
    let argument_name = segments
        .next()
        .filter(|segment| !segment.is_empty())
        .with_context(|| {
            format!(
                "PDA seed argument path is empty in instruction {}",
                instruction.name
            )
        })?;
    let argument = instruction
        .args
        .iter()
        .find(|argument| argument.name == argument_name)
        .with_context(|| {
            format!(
                "PDA seed argument {argument_name} does not exist in instruction {}",
                instruction.name
            )
        })?;

    let mut ty = &argument.ty;
    for field_name in segments {
        let Type::Defined {
            name,
            generic_arguments,
        } = ty
        else {
            bail!(
                "PDA seed argument path {path} descends through non-struct type {ty:?} in instruction {}",
                instruction.name
            );
        };
        if !generic_arguments.is_empty() {
            bail!(
                "PDA seed argument path {path} descends through generic type {name} in instruction {}",
                instruction.name
            );
        }
        let definition = types
            .iter()
            .find(|definition| definition.name == *name)
            .with_context(|| {
                format!(
                    "PDA seed argument path {path} references missing type {name} in instruction {}",
                    instruction.name
                )
            })?;
        let TypeDefinitionKind::Struct {
            fields: Some(DefinedFields::Named(fields)),
        } = &definition.kind
        else {
            bail!(
                "PDA seed argument path {path} descends through non-named struct type {name} in instruction {}",
                instruction.name
            );
        };
        ty = &fields
            .iter()
            .find(|field| field.name == field_name)
            .with_context(|| {
                format!(
                    "PDA seed argument path {path} references missing field {name}.{field_name} in instruction {}",
                    instruction.name
                )
            })?
            .ty;
    }

    Ok(ty)
}

fn seed_argument_parameter_name(path: &str) -> String {
    path.replace('.', "_")
}

fn pda_trait(
    out: &mut String,
    pdas: &[PdaDerivation<'_>],
    seed_constants: &BTreeMap<Vec<u8>, String>,
    pda_trait_name: &str,
) -> Result<()> {
    section(out, "pda derivations");
    let _ = write!(
        out,
        "/// PDA derivations scoped to one program deployment.\n\
         pub trait {pda_trait_name}: ProgramAddress {{\n",
    );
    for pda in pdas {
        let parameters = seed_parameters(pda)?;
        let signature = parameters
            .iter()
            .map(|parameter| match parameter {
                SeedParameter::Account(path) => format!("{path}: &Address"),
                SeedParameter::U32Argument(path) => {
                    format!("{}: u32", seed_argument_parameter_name(path))
                }
                SeedParameter::FixedBytesArgument { path, length } => {
                    format!("{}: &[u8; {length}]", seed_argument_parameter_name(path))
                }
            })
            .collect::<Vec<_>>()
            .join(", ");

        let mut rendered = Vec::new();
        let mut described = Vec::new();
        for seed in pda.seeds {
            match seed {
                Seed::Constant(value) => {
                    let literal = seed_constants
                        .get(value)
                        .cloned()
                        .unwrap_or_else(|| byte_string_literal(value));
                    described.push(format!("`{literal}`"));
                    rendered.push(literal);
                }
                Seed::Account { path, .. } => {
                    described.push(format!("`{path}`"));
                    rendered.push(format!("{path}.as_ref()"));
                }
                Seed::Argument { path } => match resolve_argument_seed(pda, path)? {
                    SeedParameter::U32Argument(path) => {
                        let parameter = seed_argument_parameter_name(path);
                        described.push(format!("`{path}` as little-endian `u32`"));
                        rendered.push(format!("&{parameter}.to_le_bytes()"));
                    }
                    SeedParameter::FixedBytesArgument { path, length } => {
                        let parameter = seed_argument_parameter_name(path);
                        described.push(format!("`{path}` as {length} bytes"));
                        rendered.push(format!("{parameter}.as_ref()"));
                    }
                    SeedParameter::Account(_) => unreachable!("argument resolved as account"),
                },
            }
        }

        let _ = write!(
            out,
            "    /// Derives `{}` from seeds [{}].\n\
             fn {}(&self{}{}) -> (Address, u8) {{\n        \
             Address::find_program_address(&[{}], &self.program())\n    }}\n\n",
            pda.account,
            described.join(", "),
            pda.function,
            if signature.is_empty() { "" } else { ", " },
            signature,
            rendered.join(", ")
        );
    }
    let _ = write!(
        out,
        "}}\n\nimpl<T: ProgramAddress + ?Sized> {pda_trait_name} for T {{}}\n"
    );
    Ok(())
}

// ---------------------------------------------------------------- instruction args

fn instruction_args(
    out: &mut String,
    program: &ProgramData,
    types: &HashMap<&str, &TypeDefinition>,
) -> Result<()> {
    section(out, "instruction payloads");
    out.push_str(
        "/// Instruction data, one type per instruction, each carrying that\n\
         /// instruction's discriminator.\n\
         pub mod args {\n    use super::*;\n\n",
    );
    for ix in &program.instructions {
        let name = to_camel_case(&ix.name);
        let _ = write!(out, "    {}\n", args_derives(&ix.args, types));
        if ix.args.is_empty() {
            let _ = write!(out, "    pub struct {name};\n\n");
        } else {
            let _ = write!(out, "    pub struct {name} {{\n");
            for arg in &ix.args {
                let _ = write!(out, "        pub {}: {},\n", arg.name, rust_type(&arg.ty)?);
            }
            out.push_str("    }\n\n");
        }
        let _ = write!(
            out,
            "    impl InstructionArgs for {name} {{\n        const DISCRIMINATOR: [u8; 8] = {:?};\n    }}\n\n",
            ix.discriminator
        );
    }
    out.push_str("}\n");
    Ok(())
}

fn instruction_catalog(out: &mut String, program: &ProgramData) {
    section(out, "instruction catalog");
    let name = format!("{}Instruction", program.name);
    let _ = write!(
        out,
        "/// Identity of every instruction in the IDL, for dispatch and coverage.\n\
         /// Payloads are encoded by [`args`], not by this enum's ordinal.\n\
         #[derive(Clone, Copy, Debug, PartialEq, Eq)]\n\
         pub enum {name} {{\n"
    );
    for ix in &program.instructions {
        let _ = writeln!(out, "    {},", to_camel_case(&ix.name));
    }
    let _ = write!(
        out,
        "}}\n\nimpl {name} {{\n    pub const ALL: &'static [Self] = &[\n"
    );
    for ix in &program.instructions {
        let _ = writeln!(out, "        Self::{},", to_camel_case(&ix.name));
    }
    out.push_str("    ];\n\n    pub const fn name(self) -> &'static str {\n        match self {\n");
    for ix in &program.instructions {
        let _ = writeln!(
            out,
            "            Self::{} => {:?},",
            to_camel_case(&ix.name),
            ix.name
        );
    }
    out.push_str("        }\n    }\n\n    pub const fn discriminator(self) -> [u8; 8] {\n        match self {\n");
    for ix in &program.instructions {
        let variant = to_camel_case(&ix.name);
        let _ = writeln!(
            out,
            "            Self::{variant} => args::{variant}::DISCRIMINATOR,"
        );
    }
    out.push_str(
        "        }\n    }\n\n    \
         /// Identify a discriminator without decoding or validating its payload.\n    \
         pub fn from_discriminator(discriminator: [u8; 8]) -> Option<Self> {\n        \
         Self::ALL.iter().copied().find(|instruction| instruction.discriminator() == discriminator)\n    \
         }\n}\n",
    );
}

fn args_derives(args: &[Field], types: &HashMap<&str, &TypeDefinition>) -> String {
    let mut list = vec!["BorshSerialize", "BorshDeserialize", "Clone"];
    let mut visiting = BTreeSet::new();
    if args
        .iter()
        .all(|arg| type_holds(&arg.ty, types, &TraitCheck::Copy, &mut visiting))
    {
        list.push("Copy");
    }
    list.push("Debug");
    list.push("PartialEq");
    let mut visiting = BTreeSet::new();
    if args
        .iter()
        .all(|arg| type_holds(&arg.ty, types, &TraitCheck::Eq, &mut visiting))
    {
        list.push("Eq");
    }
    format!("#[derive({})]", list.join(", "))
}

// ---------------------------------------------------------------- account structs

fn instruction_account_structs(
    out: &mut String,
    program: &ProgramData,
    pdas: &[PdaDerivation<'_>],
    fixed: &[FixedAddress],
    pda_trait_name: &str,
) -> Result<()> {
    section(out, "instruction accounts");
    out.push_str(
        "/// Account lists, one type per instruction, in the order the program\n\
         /// expects. Accounts the IDL pins to a fixed address are supplied\n\
         /// automatically and are not fields.\n\
         pub mod accounts {\n    use super::*;\n\n",
    );

    for ix in &program.instructions {
        let accounts = instruction_accounts(ix);
        let name = to_camel_case(&ix.name);

        let fields = accounts
            .iter()
            .filter(|account| account.address.is_none())
            .collect::<Vec<_>>();
        let chosen = fields
            .iter()
            // An optional PDA still has to be caller-chosen: deriving it
            // unconditionally would make the `None` representation impossible.
            .filter(|account| account.pda.is_none() || account.optional)
            .collect::<Vec<_>>();
        let mut seen_seed_arguments = BTreeSet::new();
        let mut seed_arguments = Vec::new();
        for account in fields
            .iter()
            .filter(|account| account.pda.is_some() && !account.optional)
        {
            let pda = pda_for(pdas, &ix.name, &account.name).with_context(|| {
                format!("no PDA derivation for {} in {}", account.name, ix.name)
            })?;
            for parameter in seed_parameters(pda)? {
                let path = match parameter {
                    SeedParameter::Account(_) => continue,
                    SeedParameter::U32Argument(path)
                    | SeedParameter::FixedBytesArgument { path, .. } => path,
                };
                let parameter_name = seed_argument_parameter_name(path);
                if fields.iter().any(|account| account.name == parameter_name) {
                    bail!(
                        "PDA seed argument {path} maps to parameter {parameter_name}, which conflicts with an account in instruction {}",
                        ix.name
                    );
                }
                if seen_seed_arguments.insert(parameter_name) {
                    seed_arguments.push(parameter);
                }
            }
        }

        let _ = write!(
            out,
            "    /// Accounts for `{}`.\n    #[derive(Clone, Copy, Debug, PartialEq, Eq)]\n    pub struct {name} {{\n",
            ix.name
        );
        for account in &fields {
            docs(out, &account.docs, "        ");
            let ty = if account.optional {
                "Option<Address>"
            } else {
                "Address"
            };
            let _ = write!(out, "        pub {}: {ty},\n", account.name);
        }
        out.push_str("    }\n\n");

        let _ = write!(out, "    impl {name} {{\n");

        // Constructor taking only caller-chosen accounts, deriving the PDAs.
        let derives_pdas = fields
            .iter()
            .any(|account| account.pda.is_some() && !account.optional);
        let mut signature = Vec::new();
        if derives_pdas {
            signature.push(format!("api: &impl {pda_trait_name}"));
        }
        signature.extend(chosen.iter().map(|account| {
            let ty = if account.optional {
                "Option<Address>"
            } else {
                "Address"
            };
            format!("{}: {ty}", account.name)
        }));
        signature.extend(seed_arguments.iter().map(|argument| match argument {
            SeedParameter::Account(_) => unreachable!("accounts are not seed arguments"),
            SeedParameter::U32Argument(path) => {
                format!("{}: u32", seed_argument_parameter_name(path))
            }
            SeedParameter::FixedBytesArgument { path, length } => {
                format!("{}: &[u8; {length}]", seed_argument_parameter_name(path))
            }
        }));
        let signature = signature.join(", ");
        let _ = write!(
            out,
            "        /// {}\n        pub fn new({signature}) -> Self {{\n",
            if derives_pdas {
                "Derive every PDA with the supplied API and caller-chosen accounts."
            } else {
                "Build the list from caller-chosen accounts."
            }
        );
        for account in &fields {
            if account.pda.is_none() || account.optional {
                continue;
            }
            let pda = pda_for(pdas, &ix.name, &account.name).with_context(|| {
                format!("no PDA derivation for {} in {}", account.name, ix.name)
            })?;
            let arguments = seed_parameters(pda)?
                .iter()
                .map(|parameter| match parameter {
                    SeedParameter::Account(path) => format!("&{path}"),
                    SeedParameter::U32Argument(path)
                    | SeedParameter::FixedBytesArgument { path, .. } => {
                        seed_argument_parameter_name(path)
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            let _ = write!(
                out,
                "            let {} = api.{}({arguments}).0;\n",
                account.name, pda.function
            );
        }
        let initializers = fields
            .iter()
            .map(|account| account.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        let _ = write!(out, "            Self {{ {initializers} }}\n        }}\n\n");

        // Metas in IDL order.
        let has_optional = accounts.iter().any(|account| account.optional);
        let meta_parameters = if has_optional {
            "&self, program_id: Address"
        } else {
            "&self"
        };
        let _ = write!(
            out,
            "        /// Account metas in the order the program expects.\n        pub fn to_account_metas({meta_parameters}) -> Vec<AccountMeta> {{\n            vec![\n"
        );
        for account in &accounts {
            let constructor = if account.writable {
                "new"
            } else {
                "new_readonly"
            };
            let value = match &account.address {
                Some(address) => {
                    let fixed = fixed_constant_for(fixed, address)
                        .expect("every fixed address has a constant");
                    if account.optional {
                        format!("Some({fixed})")
                    } else {
                        fixed.to_string()
                    }
                }
                None => format!("self.{}", account.name),
            };
            if account.optional {
                let _ = write!(
                    out,
                    "                match {value} {{\n                    Some(address) => AccountMeta::{constructor}(address, {}),\n                    None => AccountMeta::new_readonly(program_id, false),\n                }},\n",
                    account.signer
                );
            } else {
                let _ = write!(
                    out,
                    "                AccountMeta::{constructor}({value}, {}),\n",
                    account.signer
                );
            }
        }
        out.push_str("            ]\n        }\n    }\n\n");
    }

    out.push_str("}\n");
    Ok(())
}

// ---------------------------------------------------------------- builders

fn instruction_trait(
    out: &mut String,
    program: &ProgramData,
    pda_trait_name: &str,
    instruction_trait_name: &str,
) -> Result<()> {
    section(out, "instruction builders");
    let _ = write!(
        out,
        "/// Instruction builders scoped to one program deployment.\n\
         pub trait {instruction_trait_name}: {pda_trait_name} {{\n",
    );
    for ix in &program.instructions {
        let name = to_camel_case(&ix.name);
        let has_optional_accounts = instruction_accounts(ix)
            .iter()
            .any(|account| account.optional);
        let account_metas = if has_optional_accounts {
            "accounts.to_account_metas(self.program())"
        } else {
            "accounts.to_account_metas()"
        };
        let parameters = ix
            .args
            .iter()
            .map(|arg| Ok(format!(", {}: {}", arg.name, rust_type(&arg.ty)?)))
            .collect::<Result<Vec<_>>>()?
            .join("");
        let payload = if ix.args.is_empty() {
            format!("self::args::{name}")
        } else {
            let fields = ix
                .args
                .iter()
                .map(|arg| arg.name.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            format!("self::args::{name} {{ {fields} }}")
        };

        docs(out, &ix.docs, "    ");
        let _ = write!(
            out,
            "    /// Build the `{}` instruction.\n\
             fn {}(&self, accounts: &self::accounts::{name}{parameters}) -> Instruction {{\n        \
             Instruction {{\n        \
             program_id: self.program(),\n        \
             accounts: {account_metas},\n        \
             data: {payload}.to_instruction_data(),\n    \
             }}\n    }}\n\n",
            ix.name, ix.name
        );
    }
    let _ = write!(
        out,
        "}}\n\nimpl<T: {pda_trait_name} + ?Sized> {instruction_trait_name} for T {{}}\n"
    );
    Ok(())
}

// ---------------------------------------------------------------- errors

fn errors(out: &mut String, program: &ProgramData, name: &str) {
    if program.errors.is_empty() {
        return;
    }
    section(out, "errors");

    let _ = write!(
        out,
        "/// Errors the program returns as `InstructionError::Custom`.\n\
         #[derive(Clone, Copy, Debug, PartialEq, Eq)]\n\
         #[repr(u32)]\n\
         pub enum {name} {{\n"
    );
    for error in &program.errors {
        if let Some(message) = &error.message {
            let _ = write!(out, "    /// {message}\n");
        }
        let _ = write!(out, "    {} = {},\n", error.name, error.code);
    }
    out.push_str("}\n\n");

    let all = program
        .errors
        .iter()
        .map(|error| format!("Self::{}", error.name))
        .collect::<Vec<_>>()
        .join(", ");
    let _ = write!(
        out,
        "impl {name} {{\n    \
         /// Every variant, in IDL order.\n    \
         pub const ALL: &'static [Self] = &[{all}];\n\n    \
         /// Recover the variant a custom error code refers to.\n    \
         pub fn from_code(code: u32) -> Option<Self> {{\n        \
         Self::ALL.iter().copied().find(|error| *error as u32 == code)\n    \
         }}\n\n    \
         /// The variant name, as the IDL spells it.\n    \
         pub fn name(self) -> &'static str {{\n        match self {{\n"
    );
    for error in &program.errors {
        let _ = write!(
            out,
            "            Self::{} => \"{}\",\n",
            error.name, error.name
        );
    }
    out.push_str("        }\n    }\n\n");

    out.push_str("    /// The message the program declares for this error.\n    pub fn message(self) -> &'static str {\n        match self {\n");
    for error in &program.errors {
        let message = error.message.clone().unwrap_or_else(|| error.name.clone());
        let _ = write!(
            out,
            "            Self::{} => \"{}\",\n",
            error.name,
            message.replace('\\', "\\\\").replace('"', "\\\"")
        );
    }
    out.push_str("        }\n    }\n}\n\n");

    let _ = write!(
        out,
        "impl core::fmt::Display for {name} {{\n    \
         fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {{\n        \
         write!(f, \"{{}} ({{}}): {{}}\", self.name(), *self as u32, self.message())\n    \
         }}\n}}\n\n\
         impl std::error::Error for {name} {{}}\n"
    );
}

// ---------------------------------------------------------------- naming

/// `initialize_tenant` -> `InitializeTenant`.
fn to_camel_case(snake: &str) -> String {
    snake
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{
        direct_u32_seed_and_account_ancestry_program, nested_byte_array_seed_program,
        sample_program,
    };

    #[test]
    fn instruction_catalog_rejects_ambiguous_discriminators() {
        let mut program = direct_u32_seed_and_account_ancestry_program();
        let mut second = program.instructions[0].clone();
        second.name = "another_instruction".to_owned();
        program.instructions.push(second);
        let error = materialize(&program).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("duplicate instruction discriminator")
        );
        program.instructions[1].discriminator = vec![1; 8];
        materialize(&program).expect("different discriminators identify distinct instructions");
    }

    fn sample() -> RenderedRustFiles {
        render(&sample_program(), "Example").expect("the sample program should materialize")
    }

    #[test]
    fn eight_byte_label_seeds_preserve_width_and_reject_oversized_seeds() {
        let mut program = nested_byte_array_seed_program();
        let TypeDefinitionKind::Struct {
            fields: Some(DefinedFields::Named(fields)),
        } = &mut program.types[0].kind
        else {
            panic!("expected named fixture fields")
        };
        fields[0].ty = Type::Array(Box::new(Type::U8), ArrayLength::Value(8));
        let generated = render(&program, "Example").expect("eight-byte label should materialize");
        assert!(generated.pda.contains("args_identifier_ascii: &[u8; 8]"));
        assert!(
            generated
                .instructions
                .contains("args_identifier_ascii: &[u8; 8]")
        );
        let TypeDefinitionKind::Struct {
            fields: Some(DefinedFields::Named(fields)),
        } = &mut program.types[0].kind
        else {
            panic!("expected named fixture fields")
        };
        fields[0].ty = Type::Array(Box::new(Type::U8), ArrayLength::Value(33));
        assert!(render(&program, "Example").is_err());
    }

    #[test]
    fn data_carrying_enum_variants_preserve_named_and_tuple_fields() {
        let variants = [
            crate::program_data::EnumVariant {
                name: "Unit".to_owned(),
                fields: None,
            },
            crate::program_data::EnumVariant {
                name: "Tuple".to_owned(),
                fields: Some(DefinedFields::Tuple(vec![Type::U64, Type::Bool])),
            },
            crate::program_data::EnumVariant {
                name: "Named".to_owned(),
                fields: Some(DefinedFields::Named(vec![Field {
                    name: "value".to_owned(),
                    docs: vec!["The named value.".to_owned()],
                    ty: Type::I128,
                }])),
            },
        ];
        let mut rendered = String::new();

        for variant in &variants {
            emit_enum_variant(&mut rendered, variant).unwrap();
        }

        assert_eq!(
            rendered,
            concat!(
                "    Unit,\n",
                "    Tuple(u64, bool),\n",
                "    Named {\n",
                "        /// The named value.\n",
                "        value: i128,\n",
                "    },\n",
            )
        );
    }

    #[test]
    fn program_dependent_helpers_use_the_api_program_address() {
        let generated = sample();

        assert!(generated.pda.contains("&self.program()"));
        assert!(!generated.pda.contains("&PROGRAM_ID"));
        assert!(!generated.pda.contains("pub const PROGRAM_ID"));
        assert!(
            generated
                .instructions
                .contains("program_id: self.program()")
        );
        assert!(!generated.instructions.contains("program_id: PROGRAM_ID"));

        assert!(generated.instructions.contains("api: &impl ExamplePda,"));
        assert!(
            generated
                .instructions
                .contains("let config = api.config_pda().0;")
        );
    }

    #[test]
    fn direct_u32_seed_arguments_and_account_ancestry_generate_end_to_end() {
        let program = direct_u32_seed_and_account_ancestry_program();
        let generated = render(&program, &program.name).expect("seed graph should materialize");

        assert!(
            generated
                .pda
                .contains("fn epoch_pda(&self, channel: &Address, epoch_counter: u32)")
        );
        assert!(generated.pda.contains("&epoch_counter.to_le_bytes()"));
        assert!(
            generated
                .pda
                .contains("fn final_epoch_pda(&self, epoch: &Address)")
        );
        assert!(
            generated
                .instructions
                .contains("let epoch = api.epoch_pda(&channel, epoch_counter).0;")
        );
        assert!(
            generated
                .instructions
                .contains("let final_epoch = api.final_epoch_pda(&epoch).0;")
        );
    }

    #[test]
    fn nested_byte_array_seed_arguments_generate_end_to_end() {
        let generated = render(&nested_byte_array_seed_program(), "Example")
            .expect("nested fixed-byte seed should materialize");

        assert!(
            generated
                .pda
                .contains("fn fleet_pda(&self, args_identifier_ascii: &[u8; 32])")
        );
        assert!(generated.pda.contains("args_identifier_ascii.as_ref()"));
        assert!(
            generated
                .instructions
                .contains("pub fn new(api: &impl ExamplePda, args_identifier_ascii: &[u8; 32])")
        );
        assert!(
            generated
                .instructions
                .contains("let fleet = api.fleet_pda(args_identifier_ascii).0;")
        );
    }

    #[test]
    fn generated_clients_allow_idl_dependent_unused_imports() {
        let generated = sample();

        assert!(generated.lib.contains("\n#![allow(unused_imports)]\n"));
        assert!(!generated.lib.contains("#![allow(warnings)]"));
    }

    #[test]
    fn primitive_only_models_keep_the_generated_client_warning_free() {
        let generated = render(&nested_byte_array_seed_program(), "Example")
            .expect("primitive-only model should materialize");

        assert!(generated.models.contains(
            "#[allow(unused_imports)] // Some programs' models use only primitive fields.\n\
             use solana_address::Address;"
        ));
        assert!(generated.models.contains("pub identifier_ascii: [u8; 32],"));
    }

    #[test]
    fn programs_without_events_or_pdas_omit_unused_module_imports() {
        let mut program = nested_byte_array_seed_program();
        let InstructionAccountItem::Single(fleet) = &mut program.instructions[0].accounts[0] else {
            panic!("fixture instruction account should be flat");
        };
        fleet.pda = None;

        let generated =
            render(&program, "Example").expect("program without events or PDAs should materialize");

        assert!(!generated.lib.contains("pub use events::*;"));
        assert!(!generated.events.contains("use "));
        assert!(!generated.pda.contains("use solana_address::Address;"));
        assert!(generated.pda.contains("use crate::ProgramAddress;"));
    }

    #[test]
    fn optional_accounts_use_the_deployment_program_as_anchor_sentinel() {
        let generated = sample();

        assert!(
            generated
                .instructions
                .contains("pub referrer: Option<Address>")
        );
        assert!(
            generated
                .instructions
                .contains("pub fn to_account_metas(&self, program_id: Address)")
        );
        assert!(
            generated
                .instructions
                .contains("None => AccountMeta::new_readonly(program_id, false)")
        );
        assert!(
            generated
                .instructions
                .contains("accounts: accounts.to_account_metas(self.program())")
        );
    }

    #[test]
    fn an_idl_without_pdas_still_gets_deployment_scoping_traits() {
        let mut program = sample_program();
        program.clear_pdas();

        let generated =
            render(&program, "Example").expect("program data without PDAs should materialize");

        assert!(generated.pda.contains("pub trait ExamplePda"));
        assert!(
            generated
                .instructions
                .contains("pub trait ExampleInstructions: ExamplePda")
        );
    }
}
