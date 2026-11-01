//! Anchor IDL adapter for the language-neutral program model.

use anchor_lang_idl::types::{
    Idl, IdlArrayLen, IdlDefinedFields, IdlEnumVariant, IdlField, IdlGenericArg, IdlInstruction,
    IdlInstructionAccount, IdlInstructionAccountItem, IdlRepr, IdlReprModifier, IdlSeed,
    IdlSerialization, IdlType, IdlTypeDef, IdlTypeDefGeneric, IdlTypeDefTy,
};
use anyhow::{Context, Result, bail};
use solana_address::Address;

use crate::program_data::{
    ArrayLength, Constant, DefinedFields, EnumVariant, Field, GenericArgument, GenericParameter,
    Instruction, InstructionAccount, InstructionAccountGroup, InstructionAccountItem, Pda,
    ProgramAccount, ProgramData, ProgramError, ProgramEvent, Representation,
    RepresentationModifier, Seed, Serialization, Type, TypeDefinition, TypeDefinitionKind,
};

pub(crate) fn convert(idl: Idl, name: String) -> Result<ProgramData> {
    Ok(ProgramData {
        name,
        docs: idl.docs,
        instructions: idl
            .instructions
            .into_iter()
            .map(instruction)
            .collect::<Result<_>>()?,
        accounts: idl
            .accounts
            .into_iter()
            .map(|account| ProgramAccount {
                name: account.name,
                discriminator: account.discriminator,
            })
            .collect(),
        events: idl
            .events
            .into_iter()
            .map(|event| ProgramEvent {
                name: event.name,
                discriminator: event.discriminator,
            })
            .collect(),
        errors: idl
            .errors
            .into_iter()
            .map(|error| ProgramError {
                code: error.code,
                name: error.name,
                message: error.msg,
            })
            .collect(),
        types: idl
            .types
            .into_iter()
            .map(type_definition)
            .collect::<Result<_>>()?,
        constants: idl
            .constants
            .into_iter()
            .map(|constant| {
                Ok(Constant {
                    name: constant.name,
                    docs: constant.docs,
                    ty: value_type(constant.ty)?,
                    value: constant.value,
                })
            })
            .collect::<Result<_>>()?,
    })
}

fn instruction(instruction: IdlInstruction) -> Result<Instruction> {
    Ok(Instruction {
        name: instruction.name,
        docs: instruction.docs,
        discriminator: instruction.discriminator,
        accounts: instruction
            .accounts
            .into_iter()
            .map(instruction_account_item)
            .collect::<Result<_>>()?,
        args: instruction
            .args
            .into_iter()
            .map(field)
            .collect::<Result<_>>()?,
        returns: instruction.returns.map(value_type).transpose()?,
    })
}

fn instruction_account_item(item: IdlInstructionAccountItem) -> Result<InstructionAccountItem> {
    Ok(match item {
        IdlInstructionAccountItem::Single(account) => {
            InstructionAccountItem::Single(instruction_account(account)?)
        }
        IdlInstructionAccountItem::Composite(group) => {
            InstructionAccountItem::Composite(InstructionAccountGroup {
                name: group.name,
                accounts: group
                    .accounts
                    .into_iter()
                    .map(instruction_account_item)
                    .collect::<Result<_>>()?,
            })
        }
    })
}

fn instruction_account(account: IdlInstructionAccount) -> Result<InstructionAccount> {
    Ok(InstructionAccount {
        name: account.name,
        docs: account.docs,
        writable: account.writable,
        signer: account.signer,
        optional: account.optional,
        address: account.address.map(fixed_address).transpose()?,
        pda: account.pda.map(pda).transpose()?,
        relations: account.relations,
    })
}

fn fixed_address(address: String) -> Result<[u8; 32]> {
    address
        .parse::<Address>()
        .map(|address| address.to_bytes())
        .with_context(|| format!("invalid fixed account address {address}"))
}

fn pda(pda: anchor_lang_idl::types::IdlPda) -> Result<Pda> {
    Ok(Pda {
        seeds: pda.seeds.into_iter().map(seed).collect(),
        program: pda.program.map(seed),
    })
}

fn seed(seed: IdlSeed) -> Seed {
    match seed {
        IdlSeed::Const(seed) => Seed::Constant(seed.value),
        IdlSeed::Arg(seed) => Seed::Argument { path: seed.path },
        IdlSeed::Account(seed) => Seed::Account {
            path: seed.path,
            account_type: seed.account,
        },
    }
}

fn type_definition(definition: IdlTypeDef) -> Result<TypeDefinition> {
    Ok(TypeDefinition {
        name: definition.name,
        docs: definition.docs,
        serialization: serialization(definition.serialization)?,
        representation: definition.repr.map(representation).transpose()?,
        generic_parameters: definition
            .generics
            .into_iter()
            .map(generic_parameter)
            .collect(),
        kind: type_definition_kind(definition.ty)?,
    })
}

fn serialization(serialization: IdlSerialization) -> Result<Serialization> {
    Ok(match serialization {
        IdlSerialization::Borsh => Serialization::Borsh,
        IdlSerialization::Bytemuck => Serialization::Bytemuck,
        IdlSerialization::BytemuckUnsafe => Serialization::BytemuckUnsafe,
        IdlSerialization::Custom(name) => Serialization::Custom(name),
        other => bail!("unsupported Anchor IDL serialization {other:?}"),
    })
}

fn representation(representation: IdlRepr) -> Result<Representation> {
    Ok(match representation {
        IdlRepr::Rust(modifier) => Representation::Rust(representation_modifier(modifier)),
        IdlRepr::C(modifier) => Representation::C(representation_modifier(modifier)),
        IdlRepr::Transparent => Representation::Transparent,
        other => bail!("unsupported Anchor IDL representation {other:?}"),
    })
}

fn representation_modifier(modifier: IdlReprModifier) -> RepresentationModifier {
    RepresentationModifier {
        packed: modifier.packed,
        align: modifier.align,
    }
}

fn generic_parameter(parameter: IdlTypeDefGeneric) -> GenericParameter {
    match parameter {
        IdlTypeDefGeneric::Type { name } => GenericParameter::Type { name },
        IdlTypeDefGeneric::Const { name, ty } => GenericParameter::Constant { name, ty },
    }
}

fn type_definition_kind(kind: IdlTypeDefTy) -> Result<TypeDefinitionKind> {
    Ok(match kind {
        IdlTypeDefTy::Struct { fields } => TypeDefinitionKind::Struct {
            fields: fields.map(defined_fields).transpose()?,
        },
        IdlTypeDefTy::Enum { variants } => TypeDefinitionKind::Enum {
            variants: variants
                .into_iter()
                .map(enum_variant)
                .collect::<Result<_>>()?,
        },
        IdlTypeDefTy::Type { alias } => TypeDefinitionKind::Alias(value_type(alias)?),
    })
}

fn enum_variant(variant: IdlEnumVariant) -> Result<EnumVariant> {
    Ok(EnumVariant {
        name: variant.name,
        fields: variant.fields.map(defined_fields).transpose()?,
    })
}

fn defined_fields(fields: IdlDefinedFields) -> Result<DefinedFields> {
    Ok(match fields {
        IdlDefinedFields::Named(fields) => {
            DefinedFields::Named(fields.into_iter().map(field).collect::<Result<_>>()?)
        }
        IdlDefinedFields::Tuple(types) => {
            DefinedFields::Tuple(types.into_iter().map(value_type).collect::<Result<_>>()?)
        }
    })
}

fn field(field: IdlField) -> Result<Field> {
    Ok(Field {
        name: field.name,
        docs: field.docs,
        ty: value_type(field.ty)?,
    })
}

fn value_type(ty: IdlType) -> Result<Type> {
    Ok(match ty {
        IdlType::Bool => Type::Bool,
        IdlType::U8 => Type::U8,
        IdlType::I8 => Type::I8,
        IdlType::U16 => Type::U16,
        IdlType::I16 => Type::I16,
        IdlType::U32 => Type::U32,
        IdlType::I32 => Type::I32,
        IdlType::F32 => Type::F32,
        IdlType::U64 => Type::U64,
        IdlType::I64 => Type::I64,
        IdlType::F64 => Type::F64,
        IdlType::U128 => Type::U128,
        IdlType::I128 => Type::I128,
        IdlType::U256 => Type::U256,
        IdlType::I256 => Type::I256,
        IdlType::Bytes => Type::Bytes,
        IdlType::String => Type::String,
        IdlType::Pubkey => Type::PublicKey,
        IdlType::Option(inner) => Type::Option(Box::new(value_type(*inner)?)),
        IdlType::Vec(inner) => Type::Vec(Box::new(value_type(*inner)?)),
        IdlType::Array(inner, length) => {
            Type::Array(Box::new(value_type(*inner)?), array_length(length))
        }
        IdlType::Defined { name, generics } => Type::Defined {
            name,
            generic_arguments: generics
                .into_iter()
                .map(generic_argument)
                .collect::<Result<_>>()?,
        },
        IdlType::Generic(name) => Type::Generic(name),
        other => bail!("unsupported Anchor IDL type {other:?}"),
    })
}

fn array_length(length: IdlArrayLen) -> ArrayLength {
    match length {
        IdlArrayLen::Value(length) => ArrayLength::Value(length),
        IdlArrayLen::Generic(name) => ArrayLength::Generic(name),
    }
}

fn generic_argument(argument: IdlGenericArg) -> Result<GenericArgument> {
    Ok(match argument {
        IdlGenericArg::Type { ty } => GenericArgument::Type(value_type(ty)?),
        IdlGenericArg::Const { value } => GenericArgument::Constant(value),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{SYSTEM_PROGRAM, sample_idl};

    fn single(item: &InstructionAccountItem) -> &InstructionAccount {
        let InstructionAccountItem::Single(account) = item else {
            panic!("expected a single account, got {item:?}");
        };
        account
    }

    #[test]
    fn converts_program_level_items() {
        let program = convert(sample_idl(), "Example".to_owned()).unwrap();

        assert_eq!(program.name, "Example");
        assert_eq!(program.docs, ["A vault that records deposits."]);
        assert_eq!(
            program.accounts,
            [ProgramAccount {
                name: "Vault".to_owned(),
                discriminator: vec![3; 8],
            }]
        );
        assert_eq!(
            program.events,
            [ProgramEvent {
                name: "Deposited".to_owned(),
                discriminator: vec![4; 8],
            }]
        );
        assert_eq!(
            program.errors,
            [
                ProgramError {
                    code: 6000,
                    name: "InsufficientFunds".to_owned(),
                    message: Some("The vault balance is too low.".to_owned()),
                },
                ProgramError {
                    code: 6001,
                    name: "VaultFrozen".to_owned(),
                    message: None,
                },
            ]
        );
        assert_eq!(
            program.constants[0],
            Constant {
                name: "VAULT_SEED".to_owned(),
                docs: Vec::new(),
                ty: Type::Bytes,
                value: "[118, 97, 117, 108, 116]".to_owned(),
            }
        );
    }

    #[test]
    fn converts_instruction_accounts_seeds_args_and_returns() {
        let program = convert(sample_idl(), "Example".to_owned()).unwrap();
        let [initialize, deposit] = &program.instructions[..] else {
            panic!("expected two instructions");
        };

        let authority = single(&initialize.accounts[0]);
        assert!(authority.signer && authority.writable && !authority.optional);
        assert_eq!(
            single(&initialize.accounts[1]).pda,
            Some(Pda {
                seeds: vec![
                    Seed::Constant(b"vault".to_vec()),
                    Seed::Account {
                        path: "authority".to_owned(),
                        account_type: None,
                    },
                ],
                program: None,
            })
        );
        assert_eq!(single(&initialize.accounts[2]).address, Some([0; 32]));

        assert_eq!(
            single(&deposit.accounts[2]).pda.as_ref().unwrap().seeds[2],
            Seed::Argument {
                path: "slot".to_owned(),
            }
        );
        assert!(single(&deposit.accounts[3]).optional);
        let InstructionAccountItem::Composite(common) = &deposit.accounts[4] else {
            panic!("expected the common account group");
        };
        assert_eq!(common.name, "common");
        assert_eq!(common.accounts.len(), 2);

        assert_eq!(
            deposit.args.iter().map(|arg| &arg.ty).collect::<Vec<_>>(),
            [
                &Type::U64,
                &Type::U32,
                &Type::Option(Box::new(Type::String))
            ]
        );
        assert_eq!(deposit.returns, Some(Type::U64));
    }

    #[test]
    fn converts_struct_and_enum_definitions() {
        let program = convert(sample_idl(), "Example".to_owned()).unwrap();
        let types = &program.types;

        let TypeDefinitionKind::Struct {
            fields: Some(DefinedFields::Named(vault_fields)),
        } = &types[0].kind
        else {
            panic!("expected Vault to be a struct with named fields");
        };
        assert_eq!(
            vault_fields
                .iter()
                .map(|field| &field.ty)
                .collect::<Vec<_>>(),
            [
                &Type::PublicKey,
                &Type::Array(Box::new(Type::U8), ArrayLength::Value(8)),
                &Type::U64,
                &Type::Defined {
                    name: "Status".to_owned(),
                    generic_arguments: Vec::new(),
                },
                &Type::Bool,
                &Type::Vec(Box::new(Type::PublicKey)),
            ]
        );

        assert_eq!(types[3].name, "Status");
        assert_eq!(types[3].serialization, Serialization::Borsh);
        let TypeDefinitionKind::Enum { variants } = &types[3].kind else {
            panic!("expected Status to be an enum");
        };
        assert_eq!(
            variants.iter().map(|v| &v.fields).collect::<Vec<_>>(),
            [
                &None,
                &Some(DefinedFields::Tuple(vec![Type::I64])),
                &Some(DefinedFields::Named(vec![Field {
                    name: "reason".to_owned(),
                    docs: Vec::new(),
                    ty: Type::String,
                }])),
            ]
        );
    }

    #[test]
    fn rejects_an_invalid_fixed_account_address() {
        let mut idl = sample_idl();
        let IdlInstructionAccountItem::Single(system_program) =
            &mut idl.instructions[0].accounts[2]
        else {
            panic!("expected the system program account");
        };
        assert_eq!(system_program.address.as_deref(), Some(SYSTEM_PROGRAM));
        system_program.address = Some("not-base58".to_owned());

        let error = convert(idl, "Example".to_owned()).unwrap_err();

        assert!(
            error
                .to_string()
                .contains("invalid fixed account address not-base58")
        );
    }
}
