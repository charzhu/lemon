//! Type system for Lemon
//!
//! Handles type checking, inference, and effect/capability tracking.

pub mod check;

use crate::ast;
use crate::lexer::Span;
use std::collections::HashMap;

/// Represents a resolved type in the type system
#[derive(Debug, Clone, PartialEq)]
pub enum Ty {
    /// Primitive types
    Int(IntTy),
    Float(FloatTy),
    Bool,
    Char,
    String,
    Unit,
    Never,

    /// Compound types
    Tuple(Vec<Ty>),
    Array(Box<Ty>, usize),
    Slice(Box<Ty>),
    Reference { mutable: bool, ty: Box<Ty> },
    Function { params: Vec<Ty>, ret: Box<Ty>, effects: Vec<Effect> },

    /// Named types (structs, enums, traits, schemas)
    Named {
        name: String,
        generics: Vec<Ty>,
    },

    /// Type variable (for inference)
    Var(TypeVar),

    /// Error type (for recovery)
    Error,
}

/// Integer type variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntTy {
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    Int,  // Platform-native signed
    Uint, // Platform-native unsigned
}

/// Float type variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatTy {
    F32,
    F64,
}

/// Type variable for inference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeVar(pub u32);

/// Effects that a function can have
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Effect {
    Io,
    Random,
    Time,
    Panic,
    Custom(String),
}

/// Capabilities required by a function
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Capability {
    FileRead,
    FileWrite,
    NetworkAccess,
    ProcessSpawn,
    EnvAccess,
    TimeAccess,
    RandomAccess,
    Custom(String),
}

/// Type environment for type checking
#[derive(Debug, Default)]
pub struct TypeEnv {
    /// Variable bindings: name -> type
    variables: HashMap<String, Ty>,

    /// Type definitions: name -> type info
    types: HashMap<String, TypeDef>,

    /// Function signatures
    functions: HashMap<String, FunctionSig>,

    /// Trait definitions
    traits: HashMap<String, TraitDef>,

    /// Impl blocks
    impls: Vec<ImplDef>,

    /// Next type variable id
    next_var: u32,
}

/// Type definition
#[derive(Debug, Clone)]
pub struct TypeDef {
    pub name: String,
    pub generics: Vec<String>,
    pub kind: TypeDefKind,
}

/// Kind of type definition
#[derive(Debug, Clone)]
pub enum TypeDefKind {
    Struct(Vec<(String, Ty)>),
    Enum(Vec<(String, VariantDef)>),
    Schema(Vec<SchemaFieldDef>),
    Alias(Ty),
}

/// Enum variant definition
#[derive(Debug, Clone)]
pub enum VariantDef {
    Unit,
    Tuple(Vec<Ty>),
    Struct(Vec<(String, Ty)>),
}

/// Schema field with validation
#[derive(Debug, Clone)]
pub struct SchemaFieldDef {
    pub name: String,
    pub ty: Ty,
    pub required: bool,
    pub default: Option<ast::Expr>,
    pub validations: Vec<Validation>,
}

/// Field validation rules
#[derive(Debug, Clone)]
pub enum Validation {
    Range { min: Option<i64>, max: Option<i64> },
    Length { min: Option<usize>, max: Option<usize> },
    Pattern(String),
    Custom(String),
}

/// Function signature
#[derive(Debug, Clone)]
pub struct FunctionSig {
    pub name: String,
    pub generics: Vec<String>,
    pub params: Vec<(String, Ty)>,
    pub return_type: Ty,
    pub effects: Vec<Effect>,
    pub capabilities: Vec<Capability>,
    pub is_async: bool,
}

/// Trait definition
#[derive(Debug, Clone)]
pub struct TraitDef {
    pub name: String,
    pub generics: Vec<String>,
    pub super_traits: Vec<String>,
    pub methods: Vec<FunctionSig>,
    pub assoc_types: Vec<(String, Vec<String>)>, // (name, bounds)
}

/// Impl definition
#[derive(Debug, Clone)]
pub struct ImplDef {
    pub generics: Vec<String>,
    pub trait_: Option<String>,
    pub self_type: Ty,
    pub methods: Vec<FunctionSig>,
}

impl TypeEnv {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a fresh type variable
    pub fn fresh_var(&mut self) -> Ty {
        let var = TypeVar(self.next_var);
        self.next_var += 1;
        Ty::Var(var)
    }

    /// Define a variable in the environment
    pub fn define_var(&mut self, name: String, ty: Ty) {
        self.variables.insert(name, ty);
    }

    /// Look up a variable
    pub fn lookup_var(&self, name: &str) -> Option<&Ty> {
        self.variables.get(name)
    }

    /// Define a type
    pub fn define_type(&mut self, def: TypeDef) {
        self.types.insert(def.name.clone(), def);
    }

    /// Look up a type
    pub fn lookup_type(&self, name: &str) -> Option<&TypeDef> {
        self.types.get(name)
    }

    /// Define a function
    pub fn define_function(&mut self, sig: FunctionSig) {
        self.functions.insert(sig.name.clone(), sig);
    }

    /// Look up a function
    pub fn lookup_function(&self, name: &str) -> Option<&FunctionSig> {
        self.functions.get(name)
    }
}

impl Ty {
    /// Check if this type is a copy type
    pub fn is_copy(&self) -> bool {
        matches!(
            self,
            Ty::Int(_)
                | Ty::Float(_)
                | Ty::Bool
                | Ty::Char
                | Ty::Unit
                | Ty::Never
                | Ty::Reference { mutable: false, .. }
        )
    }

    /// Get the primitive type from a name
    pub fn from_primitive(name: &str) -> Option<Ty> {
        Some(match name {
            "i8" => Ty::Int(IntTy::I8),
            "i16" => Ty::Int(IntTy::I16),
            "i32" => Ty::Int(IntTy::I32),
            "i64" => Ty::Int(IntTy::I64),
            "i128" => Ty::Int(IntTy::I128),
            "u8" => Ty::Int(IntTy::U8),
            "u16" => Ty::Int(IntTy::U16),
            "u32" => Ty::Int(IntTy::U32),
            "u64" => Ty::Int(IntTy::U64),
            "u128" => Ty::Int(IntTy::U128),
            "int" => Ty::Int(IntTy::Int),
            "uint" => Ty::Int(IntTy::Uint),
            "f32" => Ty::Float(FloatTy::F32),
            "f64" => Ty::Float(FloatTy::F64),
            "float" => Ty::Float(FloatTy::F64),
            "bool" => Ty::Bool,
            "char" => Ty::Char,
            "String" => Ty::String,
            "str" => Ty::Slice(Box::new(Ty::Char)),
            "()" => Ty::Unit,
            "!" | "never" => Ty::Never,
            _ => return None,
        })
    }
}

/// Type checking error
#[derive(Debug, Clone)]
pub struct TypeError {
    pub message: String,
    pub span: Span,
    pub kind: TypeErrorKind,
}

/// Kind of type error
#[derive(Debug, Clone)]
pub enum TypeErrorKind {
    TypeMismatch { expected: Ty, found: Ty },
    UndefinedVariable(String),
    UndefinedType(String),
    UndefinedFunction(String),
    MissingEffect(Effect),
    MissingCapability(Capability),
    CannotInfer,
    InvalidOperation,
}

impl TypeError {
    pub fn type_mismatch(expected: Ty, found: Ty, span: Span) -> Self {
        Self {
            message: format!("type mismatch: expected {:?}, found {:?}", expected, found),
            span,
            kind: TypeErrorKind::TypeMismatch { expected, found },
        }
    }

    pub fn undefined_var(name: &str, span: Span) -> Self {
        Self {
            message: format!("undefined variable: {}", name),
            span,
            kind: TypeErrorKind::UndefinedVariable(name.to_string()),
        }
    }
}
