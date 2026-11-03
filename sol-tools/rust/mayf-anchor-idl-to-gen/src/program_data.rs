//! Language-neutral description of a program's client interface.
//!
//! These types contain no Anchor or target-language types. An input adapter
//! produces [`ProgramData`], and each language materializer consumes it.

/// Everything a materializer needs to produce a client for one program.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgramData {
    /// Client-facing program name.
    pub name: String,
    /// Program-level documentation.
    pub docs: Vec<String>,
    /// Callable instructions, in source order.
    pub instructions: Vec<Instruction>,
    /// Program-owned account types and their discriminators.
    pub accounts: Vec<ProgramAccount>,
    /// Emitted event types and their discriminators.
    pub events: Vec<ProgramEvent>,
    /// Custom program errors.
    pub errors: Vec<ProgramError>,
    /// Named data types used throughout the interface.
    pub types: Vec<TypeDefinition>,
    /// Declared constants.
    pub constants: Vec<Constant>,
}

impl ProgramData {
    /// Remove every PDA recipe, including those inside account groups.
    ///
    /// Generated clients then take those accounts as caller-supplied addresses.
    pub fn clear_pdas(&mut self) {
        fn clear(items: &mut [InstructionAccountItem]) {
            for item in items {
                match item {
                    InstructionAccountItem::Single(account) => account.pda = None,
                    InstructionAccountItem::Composite(group) => clear(&mut group.accounts),
                }
            }
        }

        for instruction in &mut self.instructions {
            clear(&mut instruction.accounts);
        }
    }
}

/// One callable program instruction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Instruction {
    /// Instruction name in snake_case, as written in the program.
    pub name: String,
    pub docs: Vec<String>,
    /// Bytes that prefix the instruction data and select this instruction.
    pub discriminator: Vec<u8>,
    /// Accounts in the order the program expects them.
    pub accounts: Vec<InstructionAccountItem>,
    /// Arguments in the order they are serialized.
    pub args: Vec<Field>,
    /// Type of the value the instruction returns, if any.
    pub returns: Option<Type>,
}

/// A single account or a named group of accounts in an instruction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InstructionAccountItem {
    /// One account.
    Single(InstructionAccount),
    /// A named group, from a nested Anchor accounts struct.
    Composite(InstructionAccountGroup),
}

/// A named group of instruction accounts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstructionAccountGroup {
    pub name: String,
    pub accounts: Vec<InstructionAccountItem>,
}

/// One account required by an instruction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstructionAccount {
    pub name: String,
    pub docs: Vec<String>,
    pub writable: bool,
    pub signer: bool,
    /// The caller may omit this account. Anchor then expects the program's own
    /// address in its place.
    pub optional: bool,
    /// A fixed address, such as the system program's.
    pub address: Option<[u8; 32]>,
    /// The recipe that derives this account's address.
    pub pda: Option<Pda>,
    /// Other accounts of the instruction whose data stores this account's
    /// address, through Anchor's `has_one` constraint.
    pub relations: Vec<String>,
}

/// Rules for deriving a program address.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pda {
    /// Seeds in derivation order.
    pub seeds: Vec<Seed>,
    /// A non-`None` value selects a program other than the generated program.
    pub program: Option<Seed>,
}

/// One component of a PDA derivation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Seed {
    /// Fixed bytes.
    Constant(Vec<u8>),
    /// An instruction argument. A dotted path, such as `args.id`, selects a
    /// field inside a struct argument.
    Argument { path: String },
    /// An account address, or a field of that account's data when the path is
    /// dotted.
    Account {
        path: String,
        /// The account's type name, when the seed reads its data.
        account_type: Option<String>,
    },
}

/// A program-owned account type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgramAccount {
    pub name: String,
    pub discriminator: Vec<u8>,
}

/// A program event type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgramEvent {
    pub name: String,
    pub discriminator: Vec<u8>,
}

/// A custom program error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgramError {
    /// The custom error code. Anchor numbers these from 6000.
    pub code: u32,
    pub name: String,
    pub message: Option<String>,
}

/// A named constant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Constant {
    pub name: String,
    pub docs: Vec<String>,
    pub ty: Type,
    /// Source representation of the value, interpreted according to [`Self::ty`].
    pub value: String,
}

/// A named data type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeDefinition {
    pub name: String,
    pub docs: Vec<String>,
    pub serialization: Serialization,
    pub representation: Option<Representation>,
    pub generic_parameters: Vec<GenericParameter>,
    pub kind: TypeDefinitionKind,
}

/// Wire serialization declared for a type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Serialization {
    Borsh,
    Bytemuck,
    BytemuckUnsafe,
    Custom(String),
}

/// In-memory representation declared for a type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Representation {
    Rust(RepresentationModifier),
    C(RepresentationModifier),
    Transparent,
}

/// Packing and alignment modifiers on a type representation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepresentationModifier {
    pub packed: bool,
    pub align: Option<usize>,
}

/// A generic type or constant parameter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GenericParameter {
    Type { name: String },
    Constant { name: String, ty: String },
}

/// The shape of a named data type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeDefinitionKind {
    Struct { fields: Option<DefinedFields> },
    Enum { variants: Vec<EnumVariant> },
    Alias(Type),
}

/// One variant of an enum.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnumVariant {
    pub name: String,
    pub fields: Option<DefinedFields>,
}

/// Named or positional fields.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DefinedFields {
    Named(Vec<Field>),
    Tuple(Vec<Type>),
}

/// A named, documented value in a struct or instruction payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Field {
    pub name: String,
    pub docs: Vec<String>,
    pub ty: Type,
}

/// A language-neutral wire type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    Bool,
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    F32,
    U64,
    I64,
    F64,
    U128,
    I128,
    U256,
    I256,
    Bytes,
    String,
    PublicKey,
    Option(Box<Type>),
    Vec(Box<Type>),
    Array(Box<Type>, ArrayLength),
    Defined {
        name: String,
        generic_arguments: Vec<GenericArgument>,
    },
    Generic(String),
}

/// A fixed or generic array length.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArrayLength {
    Value(usize),
    Generic(String),
}

/// A type or constant supplied to a generic type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GenericArgument {
    Type(Type),
    Constant(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::sample_program;

    fn pda_count(items: &[InstructionAccountItem]) -> usize {
        items
            .iter()
            .map(|item| match item {
                InstructionAccountItem::Single(account) => usize::from(account.pda.is_some()),
                InstructionAccountItem::Composite(group) => pda_count(&group.accounts),
            })
            .sum()
    }

    fn program_pda_count(program: &ProgramData) -> usize {
        program
            .instructions
            .iter()
            .map(|instruction| pda_count(&instruction.accounts))
            .sum()
    }

    #[test]
    fn clear_pdas_removes_every_recipe_including_grouped_accounts() {
        let mut program = sample_program();
        // vault, receipt, and the config account inside the `common` group.
        assert_eq!(program_pda_count(&program), 3);

        program.clear_pdas();

        assert_eq!(program_pda_count(&program), 0);
    }
}
