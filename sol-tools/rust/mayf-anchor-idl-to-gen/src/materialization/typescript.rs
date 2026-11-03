//! Materializes language-neutral program data as Anchor-free TypeScript.
//!
//! Generated wire codecs are plain `borsh` schemas. Solana-specific code is
//! limited to discriminators, public-key conversion, account metas, and PDA
//! derivation through `@solana/web3.js` v1.

#![allow(clippy::write_with_newline)]

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::Write as _;

use anyhow::{Context, Result, bail};

use crate::program_data::{
    ArrayLength, Constant, DefinedFields, EnumVariant, Instruction, InstructionAccount,
    InstructionAccountItem, ProgramData, Seed, Serialization, Type, TypeDefinition,
    TypeDefinitionKind,
};

/// TypeScript source files produced for one program client.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeScriptFileBundle {
    pub index_ts: Vec<u8>,
    pub codecs_ts: Vec<u8>,
    pub accounts_ts: Vec<u8>,
    pub errors_ts: Vec<u8>,
    pub events_ts: Vec<u8>,
    pub instructions_ts: Vec<u8>,
    pub models_ts: Vec<u8>,
    pub pda_ts: Vec<u8>,
}

impl TypeScriptFileBundle {
    /// Every generated file in stable filename order.
    pub fn files(&self) -> [(&'static str, &[u8]); 8] {
        [
            ("index.ts", &self.index_ts),
            ("accounts.ts", &self.accounts_ts),
            ("codecs.ts", &self.codecs_ts),
            ("errors.ts", &self.errors_ts),
            ("events.ts", &self.events_ts),
            ("instructions.ts", &self.instructions_ts),
            ("models.ts", &self.models_ts),
            ("pda.ts", &self.pda_ts),
        ]
    }
}

struct PdaDerivation<'data> {
    function: String,
    account: &'data str,
    instruction: &'data Instruction,
    seeds: &'data [Seed],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SeedParameter<'data> {
    Account(&'data str),
    U32Argument(&'data str),
    FixedBytesArgument { path: &'data str, length: usize },
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum AccountMetaSeedType {
    U32,
    FixedBytes { length: usize },
}

impl AccountMetaSeedType {
    fn typescript_type(&self) -> &'static str {
        match self {
            Self::U32 => "number",
            Self::FixedBytes { .. } => "ReadonlyArray<number>",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AccountMetaSeed {
    /// Original instruction-argument path from the IDL.
    path: String,
    /// Collision-checked TypeScript property name exposed to callers.
    field: String,
    ty: AccountMetaSeedType,
}

/// Complete account-construction dependencies for one instruction.
///
/// Computing this plan once keeps the standalone account-meta builder and the
/// full instruction builder structurally coupled: they share one ordered list,
/// one minimal seed set, and one decision about deployment scoping.
struct InstructionAccountPlan<'data> {
    ordered: Vec<&'data InstructionAccount>,
    inputs: Vec<&'data InstructionAccount>,
    seeds: Vec<AccountMetaSeed>,
    uses_program_id: bool,
}

impl<'data> InstructionAccountPlan<'data> {
    fn new(
        instruction: &'data Instruction,
        types: &HashMap<&str, &TypeDefinition>,
        pdas: &[PdaDerivation<'_>],
    ) -> Result<Self> {
        let ordered = instruction_accounts(instruction);
        let inputs = ordered
            .iter()
            // Optional PDAs remain caller-chosen. Deriving one unconditionally
            // would make Anchor's `null`/program-id sentinel impossible.
            .filter(|account| {
                account.address.is_none() && (account.pda.is_none() || account.optional)
            })
            .copied()
            .collect::<Vec<_>>();

        let mut seed_fields = BTreeMap::new();
        let mut seeds = Vec::new();
        for account in ordered
            .iter()
            .filter(|account| account.pda.is_some() && !account.optional)
        {
            let derivation =
                pda_for(pdas, &instruction.name, &account.name).with_context(|| {
                    format!(
                        "no PDA derivation for {} in {}",
                        account.name, instruction.name
                    )
                })?;
            for parameter in seed_parameters(derivation, types)? {
                let (path, ty) = match parameter {
                    SeedParameter::Account(_) => continue,
                    SeedParameter::U32Argument(path) => (path, AccountMetaSeedType::U32),
                    SeedParameter::FixedBytesArgument { path, length } => {
                        (path, AccountMetaSeedType::FixedBytes { length })
                    }
                };
                let field = seed_parameter_name(path);
                if let Some(previous_path) = seed_fields.get(&field) {
                    if previous_path != path {
                        bail!(
                            "PDA seed argument paths {previous_path} and {path} both map to account-meta seed field {field} in instruction {}",
                            instruction.name
                        );
                    }
                    continue;
                }
                seed_fields.insert(field.clone(), path.to_owned());
                seeds.push(AccountMetaSeed {
                    path: path.to_owned(),
                    field,
                    ty,
                });
            }
        }

        let uses_program_id = ordered.iter().any(|account| {
            (account.pda.is_some() && !account.optional)
                || (account.optional && account.address.is_none())
        });

        Ok(Self {
            ordered,
            inputs,
            seeds,
            uses_program_id,
        })
    }

    fn seed_field(&self, path: &str) -> Option<&str> {
        self.seeds
            .iter()
            .find(|seed| seed.path == path)
            .map(|seed| seed.field.as_str())
    }
}

struct FixedAddress {
    constant: String,
    address: [u8; 32],
}

/// Materialize a complete TypeScript client without performing filesystem IO.
pub fn materialize(program: &ProgramData) -> Result<TypeScriptFileBundle> {
    validate_program(program)?;
    let types = type_map(program);
    let ordered_types = schema_order(program, &types)?;
    let pdas = pdas(program)?;
    let fixed = fixed_addresses(program);
    let seed_constants = seed_constants(program)?;

    Ok(TypeScriptFileBundle {
        index_ts: index().into_bytes(),
        codecs_ts: codecs().into_bytes(),
        accounts_ts: accounts(program)?.into_bytes(),
        errors_ts: errors(program).into_bytes(),
        events_ts: events(program)?.into_bytes(),
        instructions_ts: instructions(program, &types, &pdas, &fixed)?.into_bytes(),
        models_ts: models(program, &types, &ordered_types)?.into_bytes(),
        pda_ts: pda(&pdas, &seed_constants, &types)?.into_bytes(),
    })
}

fn validate_program(program: &ProgramData) -> Result<()> {
    validate_type_name(&program.name, "program name")?;
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
                "TypeScript materialization requires an 8-byte discriminator for {kind} {name}; got {} bytes",
                discriminator.len()
            );
        }
    }

    for definition in &program.types {
        validate_type_name(&definition.name, "type name")?;
        if !matches!(definition.serialization, Serialization::Borsh) {
            bail!(
                "TypeScript materialization requires Borsh serialization for type {}; got {:?}",
                definition.name,
                definition.serialization
            );
        }
        if !definition.generic_parameters.is_empty() {
            bail!(
                "generic type {} is not supported by TypeScript materialization",
                definition.name
            );
        }
    }
    Ok(())
}

fn validate_type_name(name: &str, kind: &str) -> Result<()> {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        bail!("{kind} must not be empty");
    };
    if !first.is_ascii_uppercase() || !chars.all(|character| character.is_ascii_alphanumeric()) {
        bail!("{kind} {name:?} must be an UpperCamelCase TypeScript identifier");
    }
    Ok(())
}

fn type_map(program: &ProgramData) -> HashMap<&str, &TypeDefinition> {
    program
        .types
        .iter()
        .map(|definition| (definition.name.as_str(), definition))
        .collect()
}

fn header(out: &mut String) {
    out.push_str("// Generated from an Anchor IDL. Do not edit.\n");
}

fn section(out: &mut String, title: &str) {
    let rule = "-".repeat(70_usize.saturating_sub(title.len() + 4));
    let _ = write!(out, "\n// {title} {rule}\n\n");
}

fn docs(out: &mut String, lines: &[String], indent: &str) {
    if lines.is_empty() {
        return;
    }
    let _ = write!(out, "{indent}/**\n");
    for line in lines {
        let _ = write!(out, "{indent} * {}\n", line.replace("*/", "* /"));
    }
    let _ = write!(out, "{indent} */\n");
}

fn index() -> String {
    "// Generated from an Anchor IDL. Do not edit.\n\
     \n\
     export * from \"./accounts.js\";\n\
     export * from \"./codecs.js\";\n\
     export * from \"./errors.js\";\n\
     export * from \"./events.js\";\n\
     export * from \"./instructions.js\";\n\
     export * from \"./models.js\";\n\
     export * from \"./pda.js\";\n"
        .to_owned()
}

fn codecs() -> String {
    r#"// Generated from an Anchor IDL. Do not edit.

import { deserialize, serialize, type Schema } from "borsh";
import bs58 from "bs58";
import { PublicKey } from "@solana/web3.js";

export type PublicKeyBytes = ReadonlyArray<number>;

export const publicKeyToBytes = (value: PublicKey): PublicKeyBytes =>
  Array.from(value.toBytes());

export const publicKeyFromBytes = (value: PublicKeyBytes): PublicKey =>
  new PublicKey(Uint8Array.from(value));

export const publicKeyBytesToBase58 = (value: PublicKeyBytes): string =>
  bs58.encode(Uint8Array.from(value));

export const hasDiscriminator = (
  data: Uint8Array,
  discriminator: Uint8Array,
): boolean =>
  data.length >= discriminator.length &&
  discriminator.every((byte, index) => data[index] === byte);

export const decodeData = <T>(
  schema: Schema,
  discriminator: Uint8Array,
  data: Uint8Array,
): T => {
  if (!hasDiscriminator(data, discriminator)) {
    throw new Error("data does not match the expected discriminator");
  }
  return deserialize(schema, data.slice(discriminator.length)) as T;
};

export const encodeData = (
  schema: Schema,
  discriminator: Uint8Array,
  value: unknown,
): Uint8Array => {
  const body = serialize(schema, value);
  const data = new Uint8Array(discriminator.length + body.length);
  data.set(discriminator);
  data.set(body, discriminator.length);
  return data;
};
"#
    .to_owned()
}

// ---------------------------------------------------------------- models and schemas

fn models(
    program: &ProgramData,
    types: &HashMap<&str, &TypeDefinition>,
    ordered_types: &[&TypeDefinition],
) -> Result<String> {
    let mut out = String::new();
    header(&mut out);
    out.push_str(
        "\nimport type { Schema } from \"borsh\";\n\
         import type { PublicKeyBytes } from \"./codecs.js\";\n",
    );

    if !program.constants.is_empty() {
        section(&mut out, "constants");
        for constant in &program.constants {
            emit_constant(&mut out, constant, types)?;
        }
    }

    section(&mut out, "types");
    for definition in &program.types {
        emit_type_definition(&mut out, definition, types)?;
    }

    section(&mut out, "Borsh schemas");
    for definition in ordered_types {
        let schema = schema_for_definition(definition, types, "")?;
        let _ = write!(
            out,
            "export const {}: Schema = {schema};\n",
            schema_name(&definition.name)
        );
    }
    Ok(out)
}

fn emit_constant(
    out: &mut String,
    constant: &Constant,
    types: &HashMap<&str, &TypeDefinition>,
) -> Result<()> {
    docs(out, &constant.docs, "");
    let ty = typescript_type(&constant.ty, types, "")?;
    let value = match &constant.ty {
        Type::Bool
        | Type::U8
        | Type::I8
        | Type::U16
        | Type::I16
        | Type::U32
        | Type::I32
        | Type::F32
        | Type::F64 => constant.value.clone(),
        Type::U64 | Type::I64 | Type::U128 | Type::I128 => {
            format!("BigInt({})", js_string(&constant.value))
        }
        Type::Bytes => numeric_array(&parse_byte_literal(&constant.value)?),
        Type::Array(element, ArrayLength::Value(length)) if **element == Type::U8 => {
            let bytes = parse_byte_literal(&constant.value)?;
            if bytes.len() != *length {
                bail!(
                    "constant {} requires {length} bytes, got {}",
                    constant.name,
                    bytes.len()
                );
            }
            numeric_array(&bytes)
        }
        Type::String => constant.value.clone(),
        other => bail!(
            "constant {} has unsupported TypeScript value type {other:?}",
            constant.name
        ),
    };
    let _ = write!(
        out,
        "export const {}: {ty} = {value};\n",
        constant_name(&constant.name)
    );
    Ok(())
}

fn emit_type_definition(
    out: &mut String,
    definition: &TypeDefinition,
    types: &HashMap<&str, &TypeDefinition>,
) -> Result<()> {
    docs(out, &definition.docs, "");
    match &definition.kind {
        TypeDefinitionKind::Struct { fields } => {
            emit_struct_type(out, &definition.name, fields.as_ref(), types)?;
        }
        TypeDefinitionKind::Enum { variants } => {
            emit_enum_type(out, &definition.name, variants, types)?;
        }
        TypeDefinitionKind::Alias(alias) => {
            let _ = write!(
                out,
                "export type {} = {};\n\n",
                definition.name,
                typescript_type(alias, types, "")?
            );
        }
    }
    Ok(())
}

fn emit_struct_type(
    out: &mut String,
    name: &str,
    fields: Option<&DefinedFields>,
    types: &HashMap<&str, &TypeDefinition>,
) -> Result<()> {
    let fields = normalized_fields(fields);
    if fields.is_empty() {
        let _ = write!(out, "export type {name} = Record<string, never>;\n\n");
        return Ok(());
    }
    let _ = write!(out, "export interface {name} {{\n");
    for field in fields {
        docs(out, &field.docs, "  ");
        let _ = write!(
            out,
            "  {}: {};\n",
            field.name,
            typescript_type(&field.ty, types, "")?
        );
    }
    out.push_str("}\n\n");
    Ok(())
}

fn emit_enum_type(
    out: &mut String,
    name: &str,
    variants: &[EnumVariant],
    types: &HashMap<&str, &TypeDefinition>,
) -> Result<()> {
    if variants.is_empty() {
        bail!("enum {name} has no variants");
    }
    let variants = variants
        .iter()
        .map(|variant| {
            let fields = fields_type(variant.fields.as_ref(), types)?;
            Ok(format!(
                "  | {{ {}: {fields} }}",
                to_camel_case(&variant.name)
            ))
        })
        .collect::<Result<Vec<_>>>()?
        .join("\n");
    let _ = write!(out, "export type {name} =\n{variants};\n\n");
    Ok(())
}

fn fields_type(
    fields: Option<&DefinedFields>,
    types: &HashMap<&str, &TypeDefinition>,
) -> Result<String> {
    let fields = normalized_fields(fields);
    if fields.is_empty() {
        return Ok("Record<string, never>".into());
    }
    let fields = fields
        .iter()
        .map(|field| {
            Ok(format!(
                "{}: {}",
                field.name,
                typescript_type(&field.ty, types, "")?
            ))
        })
        .collect::<Result<Vec<_>>>()?
        .join("; ");
    Ok(format!("{{ {fields} }}"))
}

fn schema_for_definition(
    definition: &TypeDefinition,
    types: &HashMap<&str, &TypeDefinition>,
    prefix: &str,
) -> Result<String> {
    match &definition.kind {
        TypeDefinitionKind::Struct { fields } => fields_schema(fields.as_ref(), types, prefix),
        TypeDefinitionKind::Enum { variants } => {
            let variants = variants
                .iter()
                .map(|variant| {
                    Ok(format!(
                        "{{ struct: {{ {}: {} }} }}",
                        value_name(&variant.name),
                        fields_schema(variant.fields.as_ref(), types, prefix)?
                    ))
                })
                .collect::<Result<Vec<_>>>()?
                .join(", ");
            Ok(format!("{{ enum: [{variants}] }}"))
        }
        TypeDefinitionKind::Alias(alias) => schema_type(alias, types, prefix),
    }
}

fn fields_schema(
    fields: Option<&DefinedFields>,
    types: &HashMap<&str, &TypeDefinition>,
    prefix: &str,
) -> Result<String> {
    let fields = normalized_fields(fields)
        .iter()
        .map(|field| {
            Ok(format!(
                "{}: {}",
                value_name(&field.name),
                schema_type(&field.ty, types, prefix)?
            ))
        })
        .collect::<Result<Vec<_>>>()?
        .join(", ");
    Ok(format!("{{ struct: {{ {fields} }} }}"))
}

fn schema_order<'a>(
    program: &'a ProgramData,
    types: &HashMap<&'a str, &'a TypeDefinition>,
) -> Result<Vec<&'a TypeDefinition>> {
    fn visit<'a>(
        name: &'a str,
        types: &HashMap<&'a str, &'a TypeDefinition>,
        visiting: &mut BTreeSet<&'a str>,
        emitted: &mut BTreeSet<&'a str>,
        out: &mut Vec<&'a TypeDefinition>,
    ) -> Result<()> {
        if emitted.contains(name) {
            return Ok(());
        }
        if !visiting.insert(name) {
            bail!("recursive Borsh type {name} is not supported")
        }
        let definition = types
            .get(name)
            .with_context(|| format!("defined type {name} is missing from ProgramData.types"))?;
        let mut dependencies = BTreeSet::new();
        definition_dependencies(definition, &mut dependencies);
        for dependency in dependencies {
            visit(dependency, types, visiting, emitted, out)?;
        }
        visiting.remove(name);
        emitted.insert(name);
        out.push(*definition);
        Ok(())
    }

    let mut visiting = BTreeSet::new();
    let mut emitted = BTreeSet::new();
    let mut out = Vec::new();
    for definition in &program.types {
        visit(
            &definition.name,
            types,
            &mut visiting,
            &mut emitted,
            &mut out,
        )?;
    }
    Ok(out)
}

fn definition_dependencies<'a>(definition: &'a TypeDefinition, out: &mut BTreeSet<&'a str>) {
    match &definition.kind {
        TypeDefinitionKind::Struct { fields } => fields_dependencies(fields.as_ref(), out),
        TypeDefinitionKind::Enum { variants } => {
            for variant in variants {
                fields_dependencies(variant.fields.as_ref(), out);
            }
        }
        TypeDefinitionKind::Alias(alias) => type_dependencies(alias, out),
    }
}

fn fields_dependencies<'a>(fields: Option<&'a DefinedFields>, out: &mut BTreeSet<&'a str>) {
    match fields {
        None => {}
        Some(DefinedFields::Named(fields)) => {
            for field in fields {
                type_dependencies(&field.ty, out);
            }
        }
        Some(DefinedFields::Tuple(types)) => {
            for ty in types {
                type_dependencies(ty, out);
            }
        }
    }
}

fn type_dependencies<'a>(ty: &'a Type, out: &mut BTreeSet<&'a str>) {
    match ty {
        Type::Option(inner) | Type::Vec(inner) | Type::Array(inner, _) => {
            type_dependencies(inner, out);
        }
        Type::Defined { name, .. } => {
            out.insert(name);
        }
        _ => {}
    }
}

#[derive(Clone)]
struct NormalizedField {
    name: String,
    docs: Vec<String>,
    ty: Type,
}

fn normalized_fields(fields: Option<&DefinedFields>) -> Vec<NormalizedField> {
    match fields {
        None => Vec::new(),
        Some(DefinedFields::Named(fields)) => fields
            .iter()
            .map(|field| NormalizedField {
                name: value_name(&field.name),
                docs: field.docs.clone(),
                ty: field.ty.clone(),
            })
            .collect(),
        Some(DefinedFields::Tuple(types)) => types
            .iter()
            .enumerate()
            .map(|(index, ty)| NormalizedField {
                name: format!("item{index}"),
                docs: Vec::new(),
                ty: ty.clone(),
            })
            .collect(),
    }
}

fn typescript_type(
    ty: &Type,
    types: &HashMap<&str, &TypeDefinition>,
    prefix: &str,
) -> Result<String> {
    Ok(match ty {
        Type::Bool => "boolean".into(),
        Type::U8
        | Type::I8
        | Type::U16
        | Type::I16
        | Type::U32
        | Type::I32
        | Type::F32
        | Type::F64 => "number".into(),
        Type::U64 | Type::I64 | Type::U128 | Type::I128 => "bigint".into(),
        Type::U256 | Type::I256 => bail!("borsh JavaScript does not support 256-bit integers"),
        Type::Bytes => "ReadonlyArray<number>".into(),
        Type::String => "string".into(),
        Type::PublicKey => "PublicKeyBytes".into(),
        Type::Option(inner) => format!("{} | null", typescript_type(inner, types, prefix)?),
        Type::Vec(inner) | Type::Array(inner, ArrayLength::Value(_)) => {
            format!("ReadonlyArray<{}>", typescript_type(inner, types, prefix)?)
        }
        Type::Array(_, ArrayLength::Generic(name)) => {
            bail!("generic array length {name} is not supported")
        }
        Type::Defined {
            name,
            generic_arguments,
        } => {
            if !generic_arguments.is_empty() {
                bail!("generic arguments on {name} are not supported")
            }
            if !types.contains_key(name.as_str()) {
                bail!("defined type {name} is missing from ProgramData.types")
            }
            format!("{prefix}{name}")
        }
        Type::Generic(name) => bail!("generic type parameter {name} is not supported"),
    })
}

fn schema_type(ty: &Type, types: &HashMap<&str, &TypeDefinition>, prefix: &str) -> Result<String> {
    Ok(match ty {
        Type::Bool => js_string("bool"),
        Type::U8 => js_string("u8"),
        Type::I8 => js_string("i8"),
        Type::U16 => js_string("u16"),
        Type::I16 => js_string("i16"),
        Type::U32 => js_string("u32"),
        Type::I32 => js_string("i32"),
        Type::F32 => js_string("f32"),
        Type::F64 => js_string("f64"),
        Type::U64 => js_string("u64"),
        Type::I64 => js_string("i64"),
        Type::U128 => js_string("u128"),
        Type::I128 => js_string("i128"),
        Type::U256 | Type::I256 => bail!("borsh JavaScript does not support 256-bit integers"),
        Type::Bytes => "{ array: { type: \"u8\" } }".into(),
        Type::String => js_string("string"),
        Type::PublicKey => "{ array: { type: \"u8\", len: 32 } }".into(),
        Type::Option(inner) => format!("{{ option: {} }}", schema_type(inner, types, prefix)?),
        Type::Vec(inner) => format!(
            "{{ array: {{ type: {} }} }}",
            schema_type(inner, types, prefix)?
        ),
        Type::Array(inner, ArrayLength::Value(length)) => format!(
            "{{ array: {{ type: {}, len: {length} }} }}",
            schema_type(inner, types, prefix)?
        ),
        Type::Array(_, ArrayLength::Generic(name)) => {
            bail!("generic array length {name} is not supported")
        }
        Type::Defined {
            name,
            generic_arguments,
        } => {
            if !generic_arguments.is_empty() {
                bail!("generic arguments on {name} are not supported")
            }
            if !types.contains_key(name.as_str()) {
                bail!("defined type {name} is missing from ProgramData.types")
            }
            format!("{prefix}{}", schema_name(name))
        }
        Type::Generic(name) => bail!("generic type parameter {name} is not supported"),
    })
}

// ---------------------------------------------------------------- account and event codecs

fn accounts(program: &ProgramData) -> Result<String> {
    let mut out = String::new();
    header(&mut out);
    out.push_str(
        "\nimport { decodeData, encodeData, hasDiscriminator } from \"./codecs.js\";\n\
         import * as models from \"./models.js\";\n",
    );
    section(&mut out, "account codecs");
    for account in &program.accounts {
        ensure_data_type(program, &account.name, "account")?;
        let function = to_pascal_case(&account.name);
        let discriminator = format!("{}_ACCOUNT_DISCRIMINATOR", constant_name(&account.name));
        let schema = format!("models.{}", schema_name(&account.name));
        let _ = write!(
            out,
            "export const {discriminator} = Uint8Array.from({});\n\n\
             export const decode{function}Account = (data: Uint8Array): models.{} =>\n  \
             decodeData({schema}, {discriminator}, data);\n\n\
             export const encode{function}Account = (value: models.{}): Uint8Array =>\n  \
             encodeData({schema}, {discriminator}, value);\n\n",
            numeric_array(&account.discriminator),
            account.name,
            account.name
        );
    }
    emit_decode_union(&mut out, program, DataKind::Account);
    Ok(out)
}

fn events(program: &ProgramData) -> Result<String> {
    let mut out = String::new();
    header(&mut out);
    if !program.events.is_empty() {
        out.push_str(
            "\nimport { decodeData, encodeData, hasDiscriminator } from \"./codecs.js\";\n\
             import * as models from \"./models.js\";\n",
        );
    }
    section(&mut out, "event codecs");
    for event in &program.events {
        ensure_data_type(program, &event.name, "event")?;
        let function = to_pascal_case(&event.name);
        let discriminator = format!("{}_EVENT_DISCRIMINATOR", constant_name(&event.name));
        let schema = format!("models.{}", schema_name(&event.name));
        let _ = write!(
            out,
            "export const {discriminator} = Uint8Array.from({});\n\n\
             export const decode{function}Event = (data: Uint8Array): models.{} =>\n  \
             decodeData({schema}, {discriminator}, data);\n\n\
             export const encode{function}Event = (value: models.{}): Uint8Array =>\n  \
             encodeData({schema}, {discriminator}, value);\n\n",
            numeric_array(&event.discriminator),
            event.name,
            event.name
        );
    }
    emit_decode_union(&mut out, program, DataKind::Event);
    Ok(out)
}

enum DataKind {
    Account,
    Event,
}

fn emit_decode_union(out: &mut String, program: &ProgramData, kind: DataKind) {
    let (items, label) = match kind {
        DataKind::Account => (
            program
                .accounts
                .iter()
                .map(|item| (&item.name, &item.discriminator))
                .collect::<Vec<_>>(),
            "Account",
        ),
        DataKind::Event => (
            program
                .events
                .iter()
                .map(|item| (&item.name, &item.discriminator))
                .collect::<Vec<_>>(),
            "Event",
        ),
    };
    if items.is_empty() {
        return;
    }
    let decoded = format!("Decoded{}{label}", program.name);
    let default_decoder = format!("decode{}{label}", program.name);
    let decoder = if items
        .iter()
        .any(|(name, _)| format!("decode{}{label}", to_pascal_case(name)) == default_decoder)
    {
        format!("{default_decoder}Union")
    } else {
        default_decoder
    };
    let union = items
        .iter()
        .map(|(name, _)| {
            format!(
                "  | {{ name: {}; data: models.{name} }}",
                js_string(&to_camel_case(name))
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let _ = write!(out, "export type {decoded} =\n{union};\n\n");
    let _ = write!(
        out,
        "export const {decoder} = (data: Uint8Array): {decoded} | null => {{\n"
    );
    for (name, _) in items {
        let function = to_pascal_case(name);
        let discriminator = format!(
            "{}_{}_DISCRIMINATOR",
            constant_name(name),
            label.to_uppercase()
        );
        let _ = write!(
            out,
            "  if (hasDiscriminator(data, {discriminator})) {{\n    return {{ name: {}, data: decode{function}{label}(data) }};\n  }}\n",
            js_string(&to_camel_case(name))
        );
    }
    out.push_str("  return null;\n};\n");
}

fn ensure_data_type(program: &ProgramData, name: &str, kind: &str) -> Result<()> {
    if program.types.iter().any(|definition| {
        definition.name == name && !matches!(definition.kind, TypeDefinitionKind::Alias(_))
    }) {
        Ok(())
    } else {
        bail!("{kind} {name} has no concrete type definition")
    }
}

// ---------------------------------------------------------------- errors

fn errors(program: &ProgramData) -> String {
    let mut out = String::new();
    header(&mut out);
    if program.errors.is_empty() {
        out.push_str("\nexport type ProgramErrorDetails = never;\n");
        return out;
    }
    section(&mut out, "program errors");
    let collection = format!("{}_ERRORS", constant_name(&program.name));
    let error_name = format!("{}ErrorName", program.name);
    let details_name = format!("{}ErrorDetails", program.name);
    let class_name = format!("{}ProgramError", program.name);
    let _ = write!(out, "export const {collection} = {{\n");
    for error in &program.errors {
        let message = error.message.as_deref().unwrap_or(&error.name);
        let _ = write!(
            out,
            "  {}: {{ code: {}, message: {} }},\n",
            error.name,
            error.code,
            js_string(message)
        );
    }
    let _ = write!(
        out,
        "}} as const;\n\n\
         export type {error_name} = keyof typeof {collection};\n\n\
         export interface {details_name} {{\n  name: {error_name};\n  code: number;\n  message: string;\n}}\n\n\
         export const get{}Error = (code: number): {details_name} | null => {{\n  switch (code) {{\n",
        program.name
    );
    for error in &program.errors {
        let _ = write!(
            out,
            "    case {}:\n      return {{ name: {}, ...{collection}.{} }};\n",
            error.code,
            js_string(&error.name),
            error.name
        );
    }
    let _ = write!(
        out,
        "    default:\n      return null;\n  }}\n}};\n\n\
         export class {class_name} extends Error {{\n  readonly code: number;\n  readonly errorName: {error_name};\n\n  constructor(details: {details_name}) {{\n    super(`${{details.name}} (${{details.code}}): ${{details.message}}`);\n    this.name = {};\n    this.code = details.code;\n    this.errorName = details.name;\n  }}\n\n  static fromCode(code: number): {class_name} | null {{\n    const details = get{}Error(code);\n    return details === null ? null : new {class_name}(details);\n  }}\n}}\n",
        js_string(&class_name),
        program.name
    );
    out
}

// ---------------------------------------------------------------- PDAs

fn pda(
    pdas: &[PdaDerivation<'_>],
    seed_constants: &BTreeMap<Vec<u8>, String>,
    types: &HashMap<&str, &TypeDefinition>,
) -> Result<String> {
    let mut out = String::new();
    header(&mut out);
    if !pdas.is_empty() {
        out.push_str("\nimport { PublicKey } from \"@solana/web3.js\";\n");
    }
    let imports = pdas
        .iter()
        .flat_map(|derivation| derivation.seeds)
        .filter_map(|seed| match seed {
            Seed::Constant(bytes) => seed_constants.get(bytes),
            Seed::Account { .. } | Seed::Argument { .. } => None,
        })
        .map(|name| constant_name(name))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if !imports.is_empty() {
        let imports = imports.join(", ");
        let _ = write!(out, "import {{ {imports} }} from \"./models.js\";\n");
    }
    let uses_u32_argument = pdas.iter().try_fold(false, |uses_u32, derivation| {
        Ok::<_, anyhow::Error>(
            uses_u32
                || seed_parameters(derivation, types)?
                    .iter()
                    .any(|parameter| matches!(parameter, SeedParameter::U32Argument(_))),
        )
    })?;
    if uses_u32_argument {
        out.push_str(
            "\nconst u32ToLittleEndianBytes = (value: number): Uint8Array => {\n  \
             if (!Number.isInteger(value) || value < 0 || value > 0xffff_ffff) {\n    \
             throw new RangeError(`PDA u32 seed ${value} is out of range`);\n  }\n  \
             const bytes = new Uint8Array(4);\n  \
             new DataView(bytes.buffer).setUint32(0, value, true);\n  \
             return bytes;\n};\n",
        );
    }
    let uses_fixed_bytes_argument =
        pdas.iter()
            .try_fold(false, |uses_fixed_bytes, derivation| {
                Ok::<_, anyhow::Error>(
                    uses_fixed_bytes
                        || seed_parameters(derivation, types)?.iter().any(|parameter| {
                            matches!(parameter, SeedParameter::FixedBytesArgument { .. })
                        }),
                )
            })?;
    if uses_fixed_bytes_argument {
        out.push_str(
            "\nconst fixedBytesToSeed = (\n  value: ReadonlyArray<number>,\n  length: number,\n): Uint8Array => {\n  \
             if (\n    value.length !== length ||\n    value.some(\n      (byte) => !Number.isInteger(byte) || byte < 0 || byte > 0xff,\n    )\n  ) {\n    \
             throw new RangeError(`PDA seed must contain exactly ${length} bytes`);\n  }\n  \
             return Uint8Array.from(value);\n};\n",
        );
    }
    section(&mut out, "PDA derivations");
    for derivation in pdas {
        let parameters = seed_parameters(derivation, types)?;
        let signature = parameters
            .iter()
            .map(|parameter| match parameter {
                SeedParameter::Account(path) => {
                    format!("{}: PublicKey", seed_parameter_name(path))
                }
                SeedParameter::U32Argument(path) => {
                    format!("{}: number", seed_parameter_name(path))
                }
                SeedParameter::FixedBytesArgument { path, .. } => {
                    format!("{}: ReadonlyArray<number>", seed_parameter_name(path))
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        let mut seeds = Vec::new();
        for seed in derivation.seeds {
            match seed {
                Seed::Constant(bytes) => seeds.push(
                    seed_constants
                        .get(bytes)
                        .map(|name| format!("Uint8Array.from({})", constant_name(name)))
                        .unwrap_or_else(|| format!("Uint8Array.from({})", numeric_array(bytes))),
                ),
                Seed::Account { path, .. } => {
                    seeds.push(format!("{}.toBytes()", seed_parameter_name(path)));
                }
                Seed::Argument { path } => {
                    match resolve_argument_seed(derivation.instruction, types, path)? {
                        SeedParameter::U32Argument(path) => seeds.push(format!(
                            "u32ToLittleEndianBytes({})",
                            seed_parameter_name(path)
                        )),
                        SeedParameter::FixedBytesArgument { path, length } => seeds.push(format!(
                            "fixedBytesToSeed({}, {length})",
                            seed_parameter_name(path)
                        )),
                        SeedParameter::Account(_) => {
                            unreachable!("argument seeds never resolve to account parameters")
                        }
                    }
                }
            }
        }
        let comma = if signature.is_empty() { "" } else { ", " };
        let _ = write!(
            out,
            "export const {} = (\n  programId: PublicKey{comma}{signature},\n): [PublicKey, number] =>\n  \
             PublicKey.findProgramAddressSync([{}], programId);\n\n",
            derivation.function,
            seeds.join(", ")
        );
    }
    Ok(out)
}

fn pdas(program: &ProgramData) -> Result<Vec<PdaDerivation<'_>>> {
    let mut by_name: BTreeMap<&str, Vec<(&Instruction, &[Seed])>> = BTreeMap::new();
    for instruction in &program.instructions {
        for account in instruction_accounts(instruction) {
            let Some(pda) = &account.pda else { continue };
            if pda.program.is_some() {
                bail!(
                    "account {} derives from another program, which TypeScript materialization does not support",
                    account.name
                );
            }
            let variants = by_name.entry(&account.name).or_default();
            if !variants
                .iter()
                .any(|(_, seeds)| *seeds == pda.seeds.as_slice())
            {
                variants.push((instruction, pda.seeds.as_slice()));
            }
        }
    }
    let mut out = Vec::new();
    for (account, variants) in by_name {
        let ambiguous = variants.len() > 1;
        for (instruction, seeds) in variants {
            let scope = instruction
                .name
                .strip_prefix("initialize_")
                .unwrap_or(&instruction.name);
            let function = if ambiguous {
                format!(
                    "find{}PdaFor{}",
                    to_pascal_case(account),
                    to_pascal_case(scope)
                )
            } else {
                format!("find{}Pda", to_pascal_case(account))
            };
            out.push(PdaDerivation {
                function,
                account,
                instruction,
                seeds,
            });
        }
    }
    Ok(out)
}

fn pda_for<'a>(
    pdas: &'a [PdaDerivation<'a>],
    instruction: &str,
    account: &str,
) -> Option<&'a PdaDerivation<'a>> {
    pdas.iter()
        .find(|pda| pda.account == account && pda.instruction.name == instruction)
        .or_else(|| {
            let mut candidates = pdas.iter().filter(|pda| pda.account == account);
            let first = candidates.next()?;
            candidates.next().is_none().then_some(first)
        })
}

/// One parameter per distinct seed value.
///
/// A derivation may legitimately use the same account twice — the referral
/// root node is seeded on its network and owned by it — and one value needs
/// one parameter, however many seeds read it.
fn seed_parameters<'data>(
    derivation: &'data PdaDerivation<'data>,
    types: &HashMap<&str, &TypeDefinition>,
) -> Result<Vec<SeedParameter<'data>>> {
    let mut parameters: Vec<SeedParameter<'data>> = Vec::new();
    for seed in derivation.seeds {
        let parameter = match seed {
            Seed::Constant(_) => continue,
            Seed::Account { path, .. } => {
                SeedParameter::Account(direct_seed_path(path, "account")?)
            }
            Seed::Argument { path } => resolve_argument_seed(derivation.instruction, types, path)?,
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
    instruction: &Instruction,
    types: &HashMap<&str, &TypeDefinition>,
    path: &'data str,
) -> Result<SeedParameter<'data>> {
    let mut path_segments = path.split('.');
    let argument_name = path_segments
        .next()
        .filter(|name| !name.is_empty())
        .with_context(|| "PDA seed instruction argument path must not be empty")?;
    let argument = instruction
        .args
        .iter()
        .find(|argument| argument.name == argument_name)
        .with_context(|| {
            format!(
                "PDA seed argument {argument_name} is not an argument of instruction {}",
                instruction.name
            )
        })?;
    let mut ty = &argument.ty;
    for field_name in path_segments {
        if field_name.is_empty() {
            bail!("PDA seed instruction argument path {path} contains an empty field name");
        }
        ty = defined_struct_field(types, ty, field_name).with_context(|| {
            format!(
                "resolving PDA seed argument path {}.{path}",
                instruction.name
            )
        })?;
    }
    match ty {
        Type::U32 => Ok(SeedParameter::U32Argument(path)),
        Type::Array(element, ArrayLength::Value(length))
            if matches!(element.as_ref(), Type::U8) && (1..=32).contains(length) =>
        {
            Ok(SeedParameter::FixedBytesArgument {
                path,
                length: *length,
            })
        }
        _ => bail!(
            "PDA seed argument {}.{path} must be u32 or a byte array of 1 through 32 bytes; got {ty:?}",
            instruction.name
        ),
    }
}

fn defined_struct_field<'data>(
    types: &HashMap<&str, &'data TypeDefinition>,
    ty: &'data Type,
    field_name: &str,
) -> Result<&'data Type> {
    let Type::Defined { name, .. } = ty else {
        bail!("cannot select field {field_name} from non-struct type {ty:?}");
    };
    let definition = types
        .get(name.as_str())
        .with_context(|| format!("defined type {name} does not exist"))?;
    let TypeDefinitionKind::Struct {
        fields: Some(DefinedFields::Named(fields)),
    } = &definition.kind
    else {
        bail!("defined type {name} is not a struct with named fields");
    };
    fields
        .iter()
        .find(|field| field.name == field_name)
        .map(|field| &field.ty)
        .with_context(|| format!("defined type {name} has no field {field_name}"))
}

fn seed_parameter_name(path: &str) -> String {
    value_name(&path.replace('.', "_"))
}

fn argument_path_expression(path: &str) -> String {
    path.split('.')
        .map(value_name)
        .collect::<Vec<_>>()
        .join(".")
}

fn seed_constants(program: &ProgramData) -> Result<BTreeMap<Vec<u8>, String>> {
    let mut out = BTreeMap::new();
    for constant in &program.constants {
        if matches!(constant.ty, Type::Bytes) {
            out.insert(
                parse_byte_literal(&constant.value)
                    .with_context(|| format!("constant {}", constant.name))?,
                constant.name.clone(),
            );
        }
    }
    Ok(out)
}

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

// ---------------------------------------------------------------- instructions

fn instructions(
    program: &ProgramData,
    types: &HashMap<&str, &TypeDefinition>,
    pdas: &[PdaDerivation<'_>],
    fixed: &[FixedAddress],
) -> Result<String> {
    let mut out = String::new();
    header(&mut out);
    out.push_str(
        "\nimport type { Schema } from \"borsh\";\n\
         import bs58 from \"bs58\";\n\
         import { PublicKey, TransactionInstruction, type AccountMeta } from \"@solana/web3.js\";\n\
         import { encodeData } from \"./codecs.js\";\n\
         import * as models from \"./models.js\";\n",
    );
    if !pdas.is_empty() {
        out.push_str("import * as pda from \"./pda.js\";\n");
    }
    if program.instructions.iter().any(|instruction| {
        instruction
            .args
            .iter()
            .any(|argument| type_uses_direct_public_key(&argument.ty))
    }) {
        out.push_str("import type { PublicKeyBytes } from \"./codecs.js\";\n");
    }
    if !fixed.is_empty() {
        section(&mut out, "fixed addresses");
        for entry in fixed {
            let address_name = entry
                .constant
                .strip_suffix("_ID")
                .unwrap_or(&entry.constant);
            let _ = write!(
                out,
                "export const {address_name}_ADDRESS = bs58.encode(Uint8Array.from({}));\n\
                 export const {} = new PublicKey({address_name}_ADDRESS);\n",
                numeric_array(&entry.address),
                entry.constant
            );
        }
    }
    section(&mut out, "instruction schemas");
    for instruction in &program.instructions {
        emit_instruction_types(&mut out, instruction, types)?;
    }
    section(&mut out, "instruction builders");
    for instruction in &program.instructions {
        emit_instruction_builders(&mut out, instruction, types, pdas, fixed)?;
    }
    Ok(out)
}

fn type_uses_direct_public_key(ty: &Type) -> bool {
    match ty {
        Type::PublicKey => true,
        Type::Option(inner) | Type::Vec(inner) | Type::Array(inner, _) => {
            type_uses_direct_public_key(inner)
        }
        Type::Defined { .. } => false,
        _ => false,
    }
}

fn emit_instruction_types(
    out: &mut String,
    instruction: &Instruction,
    types: &HashMap<&str, &TypeDefinition>,
) -> Result<()> {
    let name = to_pascal_case(&instruction.name);
    let discriminator = format!(
        "{}_INSTRUCTION_DISCRIMINATOR",
        constant_name(&instruction.name)
    );
    let _ = write!(
        out,
        "export const {discriminator} = Uint8Array.from({});\n",
        numeric_array(&instruction.discriminator)
    );
    if instruction.args.is_empty() {
        let _ = write!(
            out,
            "export type {name}InstructionArgs = Record<string, never>;\n"
        );
    } else {
        let _ = write!(out, "export interface {name}InstructionArgs {{\n");
        for field in &instruction.args {
            docs(out, &field.docs, "  ");
            let _ = write!(
                out,
                "  {}: {};\n",
                value_name(&field.name),
                instruction_type(&field.ty, types)?
            );
        }
        out.push_str("}\n");
    }
    let fields = instruction
        .args
        .iter()
        .map(|field| {
            Ok(format!(
                "{}: {}",
                value_name(&field.name),
                schema_type(&field.ty, types, "models.")?
            ))
        })
        .collect::<Result<Vec<_>>>()?
        .join(", ");
    let _ = write!(
        out,
        "export const {}: Schema = {{ struct: {{ {fields} }} }};\n\n",
        instruction_schema_name(&instruction.name)
    );
    Ok(())
}

fn instruction_type(ty: &Type, types: &HashMap<&str, &TypeDefinition>) -> Result<String> {
    match ty {
        Type::PublicKey => Ok("PublicKeyBytes".into()),
        Type::Option(inner) => Ok(format!("{} | null", instruction_type(inner, types)?)),
        Type::Vec(inner) | Type::Array(inner, ArrayLength::Value(_)) => Ok(format!(
            "ReadonlyArray<{}>",
            instruction_type(inner, types)?
        )),
        Type::Array(_, ArrayLength::Generic(name)) => {
            bail!("generic array length {name} is not supported")
        }
        Type::Defined { name, .. } => Ok(format!("models.{name}")),
        other => typescript_type(other, types, "models."),
    }
}

fn emit_instruction_builders(
    out: &mut String,
    instruction: &Instruction,
    types: &HashMap<&str, &TypeDefinition>,
    pdas: &[PdaDerivation<'_>],
    fixed: &[FixedAddress],
) -> Result<()> {
    let name = to_pascal_case(&instruction.name);
    let plan = InstructionAccountPlan::new(instruction, types, pdas)?;
    emit_instruction_account_types(out, &name, &plan);
    emit_account_meta_builder(out, instruction, &name, &plan, types, pdas, fixed)?;
    emit_full_instruction_builder(out, instruction, &name, &plan);
    Ok(())
}

fn emit_instruction_account_types(out: &mut String, name: &str, plan: &InstructionAccountPlan<'_>) {
    let _ = write!(out, "export interface {name}Accounts {{\n");
    for account in &plan.inputs {
        docs(out, &account.docs, "  ");
        let optional = if account.optional { " | null" } else { "" };
        let _ = write!(
            out,
            "  {}: PublicKey{optional};\n",
            value_name(&account.name)
        );
    }
    out.push_str("}\n\n");

    if !plan.seeds.is_empty() {
        let _ = write!(out, "export interface {name}AccountMetaSeeds {{\n");
        for seed in &plan.seeds {
            let _ = write!(
                out,
                "  /** Instruction argument `{}`, used only to derive account addresses. */\n  {}: {};\n",
                seed.path,
                seed.field,
                seed.ty.typescript_type()
            );
        }
        out.push_str("}\n\n");
    }
}

fn emit_account_meta_builder(
    out: &mut String,
    instruction: &Instruction,
    name: &str,
    plan: &InstructionAccountPlan<'_>,
    types: &HashMap<&str, &TypeDefinition>,
    pdas: &[PdaDerivation<'_>],
    fixed: &[FixedAddress],
) -> Result<()> {
    let seed_parameter = if plan.seeds.is_empty() {
        String::new()
    } else {
        format!(",\n  seeds: {name}AccountMetaSeeds")
    };
    let program_id_parameter = if plan.uses_program_id {
        "  programId: PublicKey,\n"
    } else {
        ""
    };
    let _ = write!(
        out,
        "/**\n * Builds account metas for `{}` in the exact order the program expects.\n * No instruction data is encoded, so the result can be forwarded as remaining accounts.\n */\nexport const create{name}AccountMetas = (\n{program_id_parameter}  accounts: {name}Accounts{seed_parameter},\n): AccountMeta[] => {{\n",
        instruction.name
    );

    let mut expressions = BTreeMap::new();
    for account in &plan.ordered {
        if let Some(address) = account.address {
            expressions.insert(
                account.name.as_str(),
                fixed_constant_for(fixed, &address)
                    .expect("every fixed address has a constant")
                    .to_owned(),
            );
        } else if account.pda.is_none() || account.optional {
            expressions.insert(
                account.name.as_str(),
                format!("accounts.{}", value_name(&account.name)),
            );
        }
    }
    for account in &plan.ordered {
        let Some(_) = &account.pda else {
            continue;
        };
        if account.optional {
            continue;
        }
        let derivation = pda_for(pdas, &instruction.name, &account.name).with_context(|| {
            format!(
                "no PDA derivation for {} in {}",
                account.name, instruction.name
            )
        })?;
        let arguments = seed_parameters(derivation, types)?
            .iter()
            .map(|parameter| match parameter {
                SeedParameter::Account(path) => {
                    let expression = expressions.get(path).with_context(|| {
                        format!(
                            "PDA account {} depends on unresolved seed account {path}",
                            account.name
                        )
                    })?;
                    let optional = plan
                        .ordered
                        .iter()
                        .find(|candidate| candidate.name == *path)
                        .is_some_and(|seed| seed.optional);
                    if optional {
                        bail!(
                            "PDA account {} depends on optional seed account {path}",
                            account.name
                        );
                    }
                    Ok(expression.clone())
                }
                SeedParameter::U32Argument(path)
                | SeedParameter::FixedBytesArgument { path, .. } => {
                    let field = plan.seed_field(path).with_context(|| {
                        format!(
                            "PDA account {} depends on unplanned argument seed {path}",
                            account.name
                        )
                    })?;
                    Ok(format!("seeds.{field}"))
                }
            })
            .collect::<Result<Vec<_>>>()?
            .join(", ");
        let comma = if arguments.is_empty() { "" } else { ", " };
        let local = value_name(&account.name);
        let _ = write!(
            out,
            "  const {local} = pda.{}(programId{comma}{arguments})[0];\n",
            derivation.function
        );
        expressions.insert(account.name.as_str(), local);
    }
    out.push_str("  return [\n");
    for account in &plan.ordered {
        let mut expression = expressions
            .get(account.name.as_str())
            .with_context(|| format!("no value for instruction account {}", account.name))?
            .clone();
        let optional_input = account.optional && account.address.is_none();
        let is_writable = if optional_input && account.writable {
            format!("{expression} !== null")
        } else {
            account.writable.to_string()
        };
        let is_signer = if optional_input && account.signer {
            format!("{expression} !== null")
        } else {
            account.signer.to_string()
        };
        if optional_input {
            expression = format!("{expression} ?? programId");
        }
        let _ = write!(
            out,
            "    {{ pubkey: {expression}, isWritable: {is_writable}, isSigner: {is_signer} }},\n",
        );
    }
    out.push_str("  ];\n};\n\n");
    Ok(())
}

fn emit_full_instruction_builder(
    out: &mut String,
    instruction: &Instruction,
    name: &str,
    plan: &InstructionAccountPlan<'_>,
) {
    docs(out, &instruction.docs, "");
    let args = if instruction.args.is_empty() {
        String::new()
    } else {
        format!(",\n  args: {name}InstructionArgs")
    };
    let _ = write!(
        out,
        "export const create{name}Instruction = (\n  programId: PublicKey,\n  accounts: {name}Accounts{args},\n): TransactionInstruction => {{\n"
    );
    let program_id_argument = if plan.uses_program_id {
        "programId, "
    } else {
        ""
    };
    if plan.seeds.is_empty() {
        let _ = write!(
            out,
            "  const keys = create{name}AccountMetas({program_id_argument}accounts);\n"
        );
    } else {
        let _ = write!(
            out,
            "  const keys = create{name}AccountMetas({program_id_argument}accounts, {{\n"
        );
        for seed in &plan.seeds {
            let _ = write!(
                out,
                "    {}: args.{},\n",
                seed.field,
                argument_path_expression(&seed.path)
            );
        }
        out.push_str("  });\n");
    }
    let value = if instruction.args.is_empty() {
        "{}"
    } else {
        "args"
    };
    let _ = write!(
        out,
        "  const data = encodeData({}, {}_INSTRUCTION_DISCRIMINATOR, {value});\n  \
         return new TransactionInstruction({{ programId, keys, data: Buffer.from(data) }});\n}};\n\n",
        instruction_schema_name(&instruction.name),
        constant_name(&instruction.name)
    );
}

fn instruction_accounts(instruction: &Instruction) -> Vec<&InstructionAccount> {
    fn walk<'a>(items: &'a [InstructionAccountItem], out: &mut Vec<&'a InstructionAccount>) {
        for item in items {
            match item {
                InstructionAccountItem::Single(account) => out.push(account),
                InstructionAccountItem::Composite(group) => walk(&group.accounts, out),
            }
        }
    }
    let mut out = Vec::new();
    walk(&instruction.accounts, &mut out);
    out
}

fn fixed_addresses(program: &ProgramData) -> Vec<FixedAddress> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for account in program.instructions.iter().flat_map(instruction_accounts) {
        let Some(address) = account.address else {
            continue;
        };
        if seen.insert(address) {
            out.push(FixedAddress {
                constant: format!("{}_ID", constant_name(&account.name)),
                address,
            });
        }
    }
    out
}

fn fixed_constant_for<'a>(fixed: &'a [FixedAddress], address: &[u8; 32]) -> Option<&'a str> {
    fixed
        .iter()
        .find(|entry| entry.address == *address)
        .map(|entry| entry.constant.as_str())
}

// ---------------------------------------------------------------- naming and literals

fn schema_name(name: &str) -> String {
    format!("{}_SCHEMA", constant_name(name))
}

fn instruction_schema_name(name: &str) -> String {
    format!("{}_INSTRUCTION_SCHEMA", constant_name(name))
}

fn to_pascal_case(value: &str) -> String {
    value
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

fn to_camel_case(value: &str) -> String {
    let pascal = to_pascal_case(value);
    let mut chars = pascal.chars();
    match chars.next() {
        Some(first) => first.to_ascii_lowercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}

fn value_name(value: &str) -> String {
    let name = to_camel_case(value);
    if TYPESCRIPT_RESERVED_WORDS.contains(&name.as_str()) {
        format!("{name}_")
    } else {
        name
    }
}

fn constant_name(value: &str) -> String {
    let mut out = String::new();
    let mut previous_lowercase = false;
    for character in value.chars() {
        if matches!(character, '-' | ' ' | '_') {
            if !out.ends_with('_') {
                out.push('_');
            }
            previous_lowercase = false;
        } else {
            if character.is_ascii_uppercase() && previous_lowercase {
                out.push('_');
            }
            out.push(character.to_ascii_uppercase());
            previous_lowercase = character.is_ascii_lowercase() || character.is_ascii_digit();
        }
    }
    out
}

fn numeric_array(bytes: &[u8]) -> String {
    format!(
        "[{}]",
        bytes
            .iter()
            .map(u8::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn js_string(value: &str) -> String {
    serde_json::to_string(value).expect("serializing a string cannot fail")
}

const TYPESCRIPT_RESERVED_WORDS: &[&str] = &[
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "function",
    "if",
    "import",
    "in",
    "instanceof",
    "new",
    "null",
    "return",
    "super",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
    "yield",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{
        direct_u32_seed_and_account_ancestry_program, nested_byte_array_seed_program,
        sample_program,
    };

    #[test]
    fn fixed_byte_constants_preserve_their_declared_length() {
        let mut constant = Constant {
            name: "HOOK_DISCRIMINATOR".into(),
            docs: Vec::new(),
            ty: Type::Array(Box::new(Type::U8), ArrayLength::Value(8)),
            value: "[97, 118, 109, 45, 104, 111, 111, 107]".into(),
        };
        let mut generated = String::new();
        emit_constant(&mut generated, &constant, &HashMap::new()).unwrap();
        assert!(generated.contains("[97, 118, 109, 45, 104, 111, 111, 107]"));
        constant.value = "[97]".into();
        assert!(emit_constant(&mut String::new(), &constant, &HashMap::new()).is_err());
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
        let generated = materialize(&program).expect("eight-byte label should materialize");
        let pda = String::from_utf8(generated.pda_ts).unwrap();
        assert!(pda.contains("fixedBytesToSeed(argsIdentifierAscii, 8)"));
        let TypeDefinitionKind::Struct {
            fields: Some(DefinedFields::Named(fields)),
        } = &mut program.types[0].kind
        else {
            panic!("expected named fixture fields")
        };
        fields[0].ty = Type::Array(Box::new(Type::U8), ArrayLength::Value(33));
        assert!(materialize(&program).is_err());
    }

    #[test]
    fn emits_plain_borsh_schemas_for_the_complete_client_surface() {
        let generated = materialize(&sample_program()).expect("program data should materialize");
        assert_eq!(
            generated.files().map(|(name, _)| name),
            [
                "index.ts",
                "accounts.ts",
                "codecs.ts",
                "errors.ts",
                "events.ts",
                "instructions.ts",
                "models.ts",
                "pda.ts",
            ]
        );
        let models = String::from_utf8(generated.models_ts).unwrap();
        let codecs = String::from_utf8(generated.codecs_ts).unwrap();
        assert!(models.contains("authority: { array: { type: \"u8\", len: 32 } }"));
        assert!(codecs.contains("serialize(schema, value)"));
        assert!(codecs.contains("deserialize(schema, data.slice"));
        assert!(!codecs.contains("BinaryReader"));
        assert!(!codecs.contains("BinaryWriter"));
    }

    #[test]
    fn builders_keep_solana_effects_outside_the_schema_layer() {
        let generated = materialize(&sample_program()).expect("program data should materialize");
        let instructions = String::from_utf8(generated.instructions_ts).unwrap();
        let pda = String::from_utf8(generated.pda_ts).unwrap();
        assert!(
            instructions
                .contains("{ pubkey: SYSTEM_PROGRAM_ID, isWritable: false, isSigner: false }")
        );
        assert!(pda.contains("authority.toBytes()"));
        assert!(pda.contains("vault.toBytes(), u32ToLittleEndianBytes(slot)"));
        assert!(!pda.contains("PROGRAM_ID"));
        assert!(!pda.contains("PROGRAM_ADDRESS"));
    }

    #[test]
    fn direct_u32_seed_arguments_and_account_ancestry_generate_end_to_end() {
        let generated = materialize(&direct_u32_seed_and_account_ancestry_program())
            .expect("seed graph should materialize");
        let instructions = String::from_utf8(generated.instructions_ts).unwrap();
        let pda = String::from_utf8(generated.pda_ts).unwrap();

        assert!(pda.contains("epochCounter: number"));
        assert!(pda.contains("u32ToLittleEndianBytes(epochCounter)"));
        assert!(pda.contains("findFinalEpochPda"));
        assert!(pda.contains("epoch: PublicKey"));
        let seed_interface = instructions
            .split_once("export interface InitializeEpochAccountMetaSeeds {")
            .expect("account-meta seed interface should be generated")
            .1
            .split_once("}\n")
            .expect("account-meta seed interface should be closed")
            .0;
        assert!(seed_interface.contains("epochCounter: number;"));
        assert!(!seed_interface.contains("amount"));
        let account_meta_builder = instructions
            .split_once("export const createInitializeEpochAccountMetas")
            .expect("standalone account-meta builder should be generated")
            .1
            .split_once("export const createInitializeEpochInstruction")
            .expect("full instruction builder should follow it")
            .0;
        assert!(!account_meta_builder.contains("encodeData"));
        assert!(!account_meta_builder.contains("amount"));
        let channel_index = account_meta_builder
            .find("pubkey: accounts.channel")
            .expect("channel meta should be present");
        let epoch_index = account_meta_builder
            .find("pubkey: epoch")
            .expect("epoch meta should be present");
        let final_epoch_index = account_meta_builder
            .find("pubkey: finalEpoch")
            .expect("final epoch meta should be present");
        assert!(channel_index < epoch_index && epoch_index < final_epoch_index);
        assert!(instructions.contains(
            "const epoch = pda.findEpochPda(programId, accounts.channel, seeds.epochCounter)[0];"
        ));
        assert!(
            instructions.contains("const finalEpoch = pda.findFinalEpochPda(programId, epoch)[0];")
        );
        assert!(
            instructions
                .contains("const keys = createInitializeEpochAccountMetas(programId, accounts, {")
        );
        assert!(instructions.contains("epochCounter: args.epochCounter,"));
        assert!(instructions.contains("amount: bigint;"));
    }

    #[test]
    fn account_meta_builders_ask_only_for_the_values_they_use() {
        let mut program = direct_u32_seed_and_account_ancestry_program();
        for item in &mut program.instructions[0].accounts {
            let InstructionAccountItem::Single(account) = item else {
                panic!("fixture accounts should not be composite");
            };
            account.pda = None;
        }

        let generated = materialize(&program).expect("plain account list should materialize");
        let instructions = String::from_utf8(generated.instructions_ts).unwrap();

        assert!(instructions.contains(
            "export const createInitializeEpochAccountMetas = (\n  accounts: InitializeEpochAccounts,\n): AccountMeta[]"
        ));
        assert!(instructions.contains("const keys = createInitializeEpochAccountMetas(accounts);"));
        assert!(!instructions.contains("InitializeEpochAccountMetaSeeds"));
    }

    #[test]
    fn optional_pdas_remain_caller_chosen_and_use_the_anchor_sentinel() {
        let mut program = direct_u32_seed_and_account_ancestry_program();
        let InstructionAccountItem::Single(epoch) = &mut program.instructions[0].accounts[1] else {
            panic!("fixture epoch should not be composite");
        };
        epoch.optional = true;
        let InstructionAccountItem::Single(final_epoch) = &mut program.instructions[0].accounts[2]
        else {
            panic!("fixture final epoch should not be composite");
        };
        final_epoch.pda = None;

        let generated = materialize(&program).expect("optional PDA should remain caller-chosen");
        let instructions = String::from_utf8(generated.instructions_ts).unwrap();

        assert!(instructions.contains("epoch: PublicKey | null;"));
        assert!(instructions.contains(
            "{ pubkey: accounts.epoch ?? programId, isWritable: accounts.epoch !== null, isSigner: false }"
        ));
        assert!(!instructions.contains("InitializeEpochAccountMetaSeeds"));
        assert!(!instructions.contains("const epoch = pda.findEpochPda"));
    }

    #[test]
    fn nested_fixed_byte_array_seed_arguments_generate_end_to_end() {
        let generated = materialize(&nested_byte_array_seed_program())
            .expect("nested fixed-byte argument seed should materialize");
        let instructions = String::from_utf8(generated.instructions_ts).unwrap();
        let pda = String::from_utf8(generated.pda_ts).unwrap();

        assert!(pda.contains("argsIdentifierAscii: ReadonlyArray<number>"));
        assert!(pda.contains("fixedBytesToSeed(argsIdentifierAscii, 32)"));
        assert!(pda.contains("value.length !== length"));
        assert!(instructions.contains("export interface InitializeFleetAccountMetaSeeds"));
        assert!(instructions.contains("argsIdentifierAscii: ReadonlyArray<number>;"));
        assert!(
            instructions.contains(
                "const fleet = pda.findFleetPda(programId, seeds.argsIdentifierAscii)[0];"
            )
        );
        assert!(instructions.contains("argsIdentifierAscii: args.args.identifierAscii,"));
    }

    #[test]
    fn programs_without_events_or_pdas_omit_unused_module_imports() {
        let mut program = nested_byte_array_seed_program();
        let InstructionAccountItem::Single(fleet) = &mut program.instructions[0].accounts[0] else {
            panic!("fixture instruction account should be flat");
        };
        fleet.pda = None;

        let generated =
            materialize(&program).expect("program without events or PDAs should materialize");
        let events = String::from_utf8(generated.events_ts).expect("events should be UTF-8");
        let pda = String::from_utf8(generated.pda_ts).expect("PDA helpers should be UTF-8");

        assert!(!events.contains("import "));
        assert!(!pda.contains("import "));
    }

    #[test]
    fn union_decoder_names_change_only_when_a_concrete_decoder_collides() {
        let unchanged = materialize(&sample_program()).expect("ordinary names should materialize");
        let unchanged_accounts =
            String::from_utf8(unchanged.accounts_ts).expect("accounts should be UTF-8");
        let unchanged_events =
            String::from_utf8(unchanged.events_ts).expect("events should be UTF-8");
        assert!(
            unchanged_accounts.contains("export const decodeExampleAccount ="),
            "non-colliding account union decoder should keep its existing name"
        );
        assert!(
            unchanged_events.contains("export const decodeExampleEvent ="),
            "non-colliding event union decoder should keep its existing name"
        );

        let mut account_collision = sample_program();
        account_collision.name = "Vault".to_owned();
        let generated =
            materialize(&account_collision).expect("account name collision should materialize");
        let accounts = String::from_utf8(generated.accounts_ts).expect("accounts should be UTF-8");
        assert!(accounts.contains("export const decodeVaultAccount ="));
        assert!(accounts.contains("export const decodeVaultAccountUnion ="));

        let mut event_collision = sample_program();
        event_collision.name = "Deposited".to_owned();
        let generated =
            materialize(&event_collision).expect("event name collision should materialize");
        let events = String::from_utf8(generated.events_ts).expect("events should be UTF-8");
        assert!(events.contains("export const decodeDepositedEvent ="));
        assert!(events.contains("export const decodeDepositedEventUnion ="));
    }

    #[test]
    fn nested_argument_seed_reports_a_missing_struct_field() {
        let mut program = nested_byte_array_seed_program();
        let InstructionAccountItem::Single(fleet) = &mut program.instructions[0].accounts[0] else {
            panic!("fixture should contain the fleet account");
        };
        let Seed::Argument { path } = &mut fleet.pda.as_mut().unwrap().seeds[1] else {
            panic!("fixture should contain the argument seed");
        };
        *path = "args.missing".to_owned();

        let error = materialize(&program).expect_err("missing nested field should be rejected");
        assert!(
            error
                .to_string()
                .contains("resolving PDA seed argument path initialize_fleet.args.missing")
        );
    }
}
