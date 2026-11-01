//! Small in-memory programs shared by the unit tests.
//!
//! Tests build their input here, in Rust, and never read files. [`sample_idl`]
//! is one generic Anchor IDL that covers the usual range of IDL values. The
//! other fixtures each isolate one generator feature, so a change to the
//! sample cannot alter what those tests check.

use anchor_lang_idl::types::{
    Idl, IdlAccount, IdlArrayLen, IdlConst, IdlDefinedFields, IdlEnumVariant, IdlErrorCode,
    IdlEvent, IdlField, IdlInstruction, IdlInstructionAccount, IdlInstructionAccountItem,
    IdlInstructionAccounts, IdlMetadata, IdlPda, IdlSeed, IdlSeedAccount, IdlSeedArg, IdlSeedConst,
    IdlSerialization, IdlType, IdlTypeDef, IdlTypeDefTy,
};

use crate::anchor_idl;
use crate::program_data::{
    ArrayLength, DefinedFields, Field, Instruction, InstructionAccount, InstructionAccountItem,
    Pda, ProgramData, Seed, Serialization, Type, TypeDefinition, TypeDefinitionKind,
};

/// The base58 address of the Solana system program.
pub(crate) const SYSTEM_PROGRAM: &str = "11111111111111111111111111111111";

/// A generic vault program in Anchor IDL form.
///
/// It covers the usual range of IDL values:
/// - signer, writable, optional, and fixed-address accounts;
/// - a composite account group;
/// - PDAs with constant, account, and instruction-argument seeds;
/// - primitive, public-key, array, option, vector, and defined types;
/// - a struct, and an enum with unit, tuple, and named variants;
/// - an instruction return value, one account, one event, two errors, and two
///   constants.
pub(crate) fn sample_idl() -> Idl {
    Idl {
        address: "Vau1t11111111111111111111111111111111111111".to_owned(),
        metadata: IdlMetadata {
            name: "vault".to_owned(),
            version: "0.1.0".to_owned(),
            spec: "0.1.0".to_owned(),
            description: None,
            repository: None,
            dependencies: Vec::new(),
            contact: None,
            deployments: None,
        },
        docs: vec!["A vault that records deposits.".to_owned()],
        instructions: vec![
            IdlInstruction {
                name: "initialize".to_owned(),
                docs: vec!["Create a vault for an authority.".to_owned()],
                discriminator: vec![1; 8],
                accounts: vec![
                    single(account("authority", true, true)),
                    single(IdlInstructionAccount {
                        pda: Some(pda(vec![
                            constant_seed(b"vault"),
                            account_seed("authority"),
                        ])),
                        ..account("vault", true, false)
                    }),
                    single(system_program()),
                ],
                args: vec![field("config", defined("VaultConfig"))],
                returns: None,
            },
            IdlInstruction {
                name: "deposit".to_owned(),
                docs: vec!["Add funds and return the new balance.".to_owned()],
                discriminator: vec![2; 8],
                accounts: vec![
                    single(account("depositor", true, true)),
                    single(account("vault", true, false)),
                    single(IdlInstructionAccount {
                        pda: Some(pda(vec![
                            constant_seed(b"receipt"),
                            account_seed("vault"),
                            IdlSeed::Arg(IdlSeedArg {
                                path: "slot".to_owned(),
                            }),
                        ])),
                        ..account("receipt", true, false)
                    }),
                    single(IdlInstructionAccount {
                        optional: true,
                        ..account("referrer", false, false)
                    }),
                    IdlInstructionAccountItem::Composite(IdlInstructionAccounts {
                        name: "common".to_owned(),
                        accounts: vec![
                            single(IdlInstructionAccount {
                                pda: Some(pda(vec![constant_seed(b"config")])),
                                ..account("config", false, false)
                            }),
                            single(system_program()),
                        ],
                    }),
                ],
                args: vec![
                    field("amount", IdlType::U64),
                    field("slot", IdlType::U32),
                    field("memo", IdlType::Option(Box::new(IdlType::String))),
                ],
                returns: Some(IdlType::U64),
            },
        ],
        accounts: vec![IdlAccount {
            name: "Vault".to_owned(),
            discriminator: vec![3; 8],
        }],
        events: vec![IdlEvent {
            name: "Deposited".to_owned(),
            discriminator: vec![4; 8],
        }],
        errors: vec![
            IdlErrorCode {
                code: 6000,
                name: "InsufficientFunds".to_owned(),
                msg: Some("The vault balance is too low.".to_owned()),
            },
            IdlErrorCode {
                code: 6001,
                name: "VaultFrozen".to_owned(),
                msg: None,
            },
        ],
        types: vec![
            struct_type(
                "Vault",
                vec![
                    field("authority", IdlType::Pubkey),
                    field("label", byte_array(8)),
                    field("balance", IdlType::U64),
                    field("status", defined("Status")),
                    field("frozen", IdlType::Bool),
                    field("depositors", IdlType::Vec(Box::new(IdlType::Pubkey))),
                ],
            ),
            struct_type(
                "VaultConfig",
                vec![
                    field("label", byte_array(8)),
                    field("max_deposit", IdlType::U64),
                ],
            ),
            struct_type(
                "Deposited",
                vec![
                    field("vault", IdlType::Pubkey),
                    field("amount", IdlType::U64),
                    field("memo", IdlType::Option(Box::new(IdlType::String))),
                ],
            ),
            IdlTypeDef {
                name: "Status".to_owned(),
                docs: Vec::new(),
                serialization: IdlSerialization::Borsh,
                repr: None,
                generics: Vec::new(),
                ty: IdlTypeDefTy::Enum {
                    variants: vec![
                        IdlEnumVariant {
                            name: "Active".to_owned(),
                            fields: None,
                        },
                        IdlEnumVariant {
                            name: "Paused".to_owned(),
                            fields: Some(IdlDefinedFields::Tuple(vec![IdlType::I64])),
                        },
                        IdlEnumVariant {
                            name: "Closed".to_owned(),
                            fields: Some(IdlDefinedFields::Named(vec![field(
                                "reason",
                                IdlType::String,
                            )])),
                        },
                    ],
                },
            },
        ],
        constants: vec![
            IdlConst {
                name: "VAULT_SEED".to_owned(),
                docs: Vec::new(),
                ty: IdlType::Bytes,
                value: "[118, 97, 117, 108, 116]".to_owned(),
            },
            IdlConst {
                name: "MAX_DEPOSIT".to_owned(),
                docs: Vec::new(),
                ty: IdlType::U64,
                value: "1000000".to_owned(),
            },
        ],
    }
}

/// [`sample_idl`] converted to [`ProgramData`] under the name `Example`.
pub(crate) fn sample_program() -> ProgramData {
    anchor_idl::convert(sample_idl(), "Example".to_owned()).expect("the sample IDL is valid")
}

fn account(name: &str, writable: bool, signer: bool) -> IdlInstructionAccount {
    IdlInstructionAccount {
        name: name.to_owned(),
        docs: Vec::new(),
        writable,
        signer,
        optional: false,
        address: None,
        pda: None,
        relations: Vec::new(),
    }
}

fn system_program() -> IdlInstructionAccount {
    IdlInstructionAccount {
        address: Some(SYSTEM_PROGRAM.to_owned()),
        ..account("system_program", false, false)
    }
}

fn single(account: IdlInstructionAccount) -> IdlInstructionAccountItem {
    IdlInstructionAccountItem::Single(account)
}

fn pda(seeds: Vec<IdlSeed>) -> IdlPda {
    IdlPda {
        seeds,
        program: None,
    }
}

fn constant_seed(value: &[u8]) -> IdlSeed {
    IdlSeed::Const(IdlSeedConst {
        value: value.to_vec(),
    })
}

fn account_seed(path: &str) -> IdlSeed {
    IdlSeed::Account(IdlSeedAccount {
        path: path.to_owned(),
        account: None,
    })
}

fn field(name: &str, ty: IdlType) -> IdlField {
    IdlField {
        name: name.to_owned(),
        docs: Vec::new(),
        ty,
    }
}

fn defined(name: &str) -> IdlType {
    IdlType::Defined {
        name: name.to_owned(),
        generics: Vec::new(),
    }
}

fn byte_array(length: usize) -> IdlType {
    IdlType::Array(Box::new(IdlType::U8), IdlArrayLen::Value(length))
}

fn struct_type(name: &str, fields: Vec<IdlField>) -> IdlTypeDef {
    IdlTypeDef {
        name: name.to_owned(),
        docs: Vec::new(),
        serialization: IdlSerialization::Borsh,
        repr: None,
        generics: Vec::new(),
        ty: IdlTypeDefTy::Struct {
            fields: Some(IdlDefinedFields::Named(fields)),
        },
    }
}

/// A PDA graph with a direct `u32` argument seed, and a derived account used
/// as the seed of a later derived account. The unrelated `amount` argument
/// shows that account-meta generation does not need the full payload.
pub(crate) fn direct_u32_seed_and_account_ancestry_program() -> ProgramData {
    ProgramData {
        name: "SeedGraph".to_owned(),
        docs: Vec::new(),
        instructions: vec![Instruction {
            name: "initialize_epoch".to_owned(),
            docs: Vec::new(),
            discriminator: vec![0; 8],
            accounts: vec![
                writable_account("channel", None),
                writable_account(
                    "epoch",
                    Some(Pda {
                        seeds: vec![
                            Seed::Constant(b"epoch".to_vec()),
                            Seed::Account {
                                path: "channel".to_owned(),
                                account_type: None,
                            },
                            Seed::Argument {
                                path: "epoch_counter".to_owned(),
                            },
                        ],
                        program: None,
                    }),
                ),
                writable_account(
                    "final_epoch",
                    Some(Pda {
                        seeds: vec![
                            Seed::Constant(b"final-epoch".to_vec()),
                            Seed::Account {
                                path: "epoch".to_owned(),
                                account_type: None,
                            },
                        ],
                        program: None,
                    }),
                ),
            ],
            args: vec![
                Field {
                    name: "epoch_counter".to_owned(),
                    docs: Vec::new(),
                    ty: Type::U32,
                },
                Field {
                    name: "amount".to_owned(),
                    docs: Vec::new(),
                    ty: Type::U64,
                },
            ],
            returns: None,
        }],
        accounts: Vec::new(),
        events: Vec::new(),
        errors: Vec::new(),
        types: Vec::new(),
        constants: Vec::new(),
    }
}

/// A PDA seeded by a 32-byte array inside a struct argument
/// (`args.identifier_ascii`). The program has no events.
pub(crate) fn nested_byte_array_seed_program() -> ProgramData {
    ProgramData {
        name: "Example".to_owned(),
        docs: Vec::new(),
        instructions: vec![Instruction {
            name: "initialize_fleet".to_owned(),
            docs: Vec::new(),
            discriminator: vec![1; 8],
            accounts: vec![writable_account(
                "fleet",
                Some(Pda {
                    seeds: vec![
                        Seed::Constant(b"fleet".to_vec()),
                        Seed::Argument {
                            path: "args.identifier_ascii".to_owned(),
                        },
                    ],
                    program: None,
                }),
            )],
            args: vec![Field {
                name: "args".to_owned(),
                docs: Vec::new(),
                ty: Type::Defined {
                    name: "InitializeFleetArgs".to_owned(),
                    generic_arguments: Vec::new(),
                },
            }],
            returns: None,
        }],
        accounts: Vec::new(),
        events: Vec::new(),
        errors: Vec::new(),
        types: vec![TypeDefinition {
            name: "InitializeFleetArgs".to_owned(),
            docs: Vec::new(),
            serialization: Serialization::Borsh,
            representation: None,
            generic_parameters: Vec::new(),
            kind: TypeDefinitionKind::Struct {
                fields: Some(DefinedFields::Named(vec![Field {
                    name: "identifier_ascii".to_owned(),
                    docs: Vec::new(),
                    ty: Type::Array(Box::new(Type::U8), ArrayLength::Value(32)),
                }])),
            },
        }],
        constants: Vec::new(),
    }
}

fn writable_account(name: &str, pda: Option<Pda>) -> InstructionAccountItem {
    InstructionAccountItem::Single(InstructionAccount {
        name: name.to_owned(),
        docs: Vec::new(),
        writable: true,
        signer: false,
        optional: false,
        address: None,
        pda,
        relations: Vec::new(),
    })
}
