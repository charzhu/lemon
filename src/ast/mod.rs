//! Abstract Syntax Tree definitions for Lemon
//!
//! The AST represents the parsed structure of a Lemon program.

use crate::lexer::Span;

/// A complete Lemon source file
#[derive(Debug, Clone)]
pub struct SourceFile {
    pub items: Vec<Item>,
    pub span: Span,
}

/// Top-level items in a source file
#[derive(Debug, Clone)]
pub enum Item {
    Function(Function),
    Struct(StructDef),
    Enum(EnumDef),
    Trait(TraitDef),
    Impl(ImplBlock),
    Schema(SchemaDef),
    Capability(CapabilityDef),
    Use(UseDecl),
    Mod(ModDecl),
    TypeAlias(TypeAlias),
    Const(ConstDef),
    Static(StaticDef),
    // OOP items
    Class(ClassDef),
    Interface(InterfaceDef),
}

/// Function definition
#[derive(Debug, Clone)]
pub struct Function {
    pub name: Ident,
    pub generics: Option<Generics>,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub effects: Vec<Ident>,
    pub capabilities: Vec<Ident>,
    pub where_clause: Option<WhereClause>,
    pub body: Option<Block>,
    pub is_async: bool,
    pub visibility: Visibility,
    pub attributes: Vec<Attribute>,
    pub span: Span,
}

/// Function parameter
#[derive(Debug, Clone)]
pub struct Param {
    pub pattern: Pattern,
    pub ty: Type,
    pub span: Span,
}

/// Struct definition
#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: Ident,
    pub generics: Option<Generics>,
    pub fields: Vec<Field>,
    pub visibility: Visibility,
    pub attributes: Vec<Attribute>,
    pub span: Span,
}

/// Struct/schema field
#[derive(Debug, Clone)]
pub struct Field {
    pub name: Ident,
    pub ty: Type,
    pub visibility: Visibility,
    pub attributes: Vec<Attribute>,
    pub span: Span,
}

/// Enum definition
#[derive(Debug, Clone)]
pub struct EnumDef {
    pub name: Ident,
    pub generics: Option<Generics>,
    pub variants: Vec<Variant>,
    pub visibility: Visibility,
    pub attributes: Vec<Attribute>,
    pub span: Span,
}

/// Enum variant
#[derive(Debug, Clone)]
pub struct Variant {
    pub name: Ident,
    pub fields: VariantFields,
    pub attributes: Vec<Attribute>,
    pub span: Span,
}

/// Enum variant field types
#[derive(Debug, Clone)]
pub enum VariantFields {
    Unit,
    Tuple(Vec<Type>),
    Struct(Vec<Field>),
}

/// Trait definition
#[derive(Debug, Clone)]
pub struct TraitDef {
    pub name: Ident,
    pub generics: Option<Generics>,
    pub super_traits: Vec<TypePath>,
    pub items: Vec<TraitItem>,
    pub visibility: Visibility,
    pub attributes: Vec<Attribute>,
    pub span: Span,
}

/// Item within a trait definition
#[derive(Debug, Clone)]
pub enum TraitItem {
    Function(Function),
    Type(AssocType),
    Const(ConstDef),
}

/// Associated type in a trait
#[derive(Debug, Clone)]
pub struct AssocType {
    pub name: Ident,
    pub bounds: Vec<TypePath>,
    pub default: Option<Type>,
    pub span: Span,
}

/// Implementation block
#[derive(Debug, Clone)]
pub struct ImplBlock {
    pub generics: Option<Generics>,
    pub trait_: Option<TypePath>,
    pub self_type: Type,
    pub where_clause: Option<WhereClause>,
    pub items: Vec<ImplItem>,
    pub attributes: Vec<Attribute>,
    pub span: Span,
}

/// Item within an impl block
#[derive(Debug, Clone)]
pub enum ImplItem {
    Function(Function),
    Type(TypeAlias),
    Const(ConstDef),
}

/// Schema definition (Lemon-specific structured data type)
#[derive(Debug, Clone)]
pub struct SchemaDef {
    pub name: Ident,
    pub generics: Option<Generics>,
    pub fields: Vec<SchemaField>,
    pub visibility: Visibility,
    pub attributes: Vec<Attribute>,
    pub span: Span,
}

/// Schema field with validation annotations
#[derive(Debug, Clone)]
pub struct SchemaField {
    pub name: Ident,
    pub ty: Type,
    pub annotations: Vec<Annotation>,
    pub span: Span,
}

/// Capability definition (Lemon-specific security primitive)
#[derive(Debug, Clone)]
pub struct CapabilityDef {
    pub name: Ident,
    pub composed_of: Vec<Ident>,
    pub visibility: Visibility,
    pub span: Span,
}

// ============================================================================
// OOP: Class-based Object-Oriented Programming
// ============================================================================

/// Class definition
///
/// Example:
/// ```lemon
/// class Animal {
///     protected name: String,
///
///     pub fn new(name: String) -> Self {
///         Self { name }
///     }
///
///     pub fn speak(this) -> String {
///         "..."
///     }
/// }
///
/// class Dog extends Animal implements Comparable {
///     delegate bark_handler: BarkBehavior,  // Delegation
///
///     pub override fn speak(this) -> String {
///         "Woof!"
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ClassDef {
    pub name: Ident,
    pub generics: Option<Generics>,
    pub extends: Option<TypePath>,          // Single inheritance
    pub implements: Vec<TypePath>,          // Multiple interfaces
    pub members: Vec<ClassMember>,
    pub visibility: Visibility,
    pub is_abstract: bool,
    pub is_final: bool,
    pub attributes: Vec<Attribute>,
    pub span: Span,
}

/// Member of a class
#[derive(Debug, Clone)]
pub enum ClassMember {
    /// Field: `name: String`
    Field(ClassField),

    /// Method: `fn speak(this) -> String { ... }`
    Method(Method),

    /// Constructor: `fn new(...) -> Self { ... }`
    Constructor(Method),

    /// Static method: `static fn create() -> Self { ... }`
    StaticMethod(Method),

    /// Delegate field: `delegate handler: EventHandler`
    /// Methods from the delegate type are forwarded automatically
    Delegate(DelegateField),
}

/// Class field
#[derive(Debug, Clone)]
pub struct ClassField {
    pub name: Ident,
    pub ty: Type,
    pub default: Option<Expr>,
    pub visibility: MemberVisibility,
    pub is_static: bool,
    pub is_final: bool,
    pub attributes: Vec<Attribute>,
    pub span: Span,
}

/// Delegate field (for composition/delegation pattern)
///
/// Example: `delegate logger: Logger`
/// This automatically forwards all Logger methods to this field
#[derive(Debug, Clone)]
pub struct DelegateField {
    pub name: Ident,
    pub ty: Type,
    pub visibility: MemberVisibility,
    pub span: Span,
}

/// Method definition (used in classes and interfaces)
#[derive(Debug, Clone)]
pub struct Method {
    pub name: Ident,
    pub generics: Option<Generics>,
    pub params: Vec<Param>,             // First param can be `this` or `mut this`
    pub return_type: Option<Type>,
    pub effects: Vec<Ident>,
    pub capabilities: Vec<Ident>,
    pub body: Option<Block>,            // None for abstract/interface methods
    pub visibility: MemberVisibility,
    pub is_async: bool,
    pub is_abstract: bool,
    pub is_override: bool,
    pub is_final: bool,
    pub attributes: Vec<Attribute>,
    pub span: Span,
}

/// Interface definition (like Java interfaces or Rust traits)
///
/// Example:
/// ```lemon
/// interface Drawable {
///     fn draw(this, canvas: Canvas);
///
///     fn bounds(this) -> Rect {
///         // Default implementation
///         Rect::zero()
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct InterfaceDef {
    pub name: Ident,
    pub generics: Option<Generics>,
    pub extends: Vec<TypePath>,         // Interface can extend multiple interfaces
    pub methods: Vec<Method>,
    pub visibility: Visibility,
    pub attributes: Vec<Attribute>,
    pub span: Span,
}

/// Member visibility (pub, protected, private)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MemberVisibility {
    #[default]
    Private,
    Protected,
    Public,
}

/// Use declaration (imports)
#[derive(Debug, Clone)]
pub struct UseDecl {
    pub tree: UseTree,
    pub visibility: Visibility,
    pub span: Span,
}

/// Use tree for nested imports
#[derive(Debug, Clone)]
pub enum UseTree {
    Path(Ident, Box<UseTree>),
    Name(Ident),
    Rename(Ident, Ident),
    Glob,
    Group(Vec<UseTree>),
}

/// Module declaration
#[derive(Debug, Clone)]
pub struct ModDecl {
    pub name: Ident,
    pub items: Option<Vec<Item>>,
    pub visibility: Visibility,
    pub span: Span,
}

/// Type alias
#[derive(Debug, Clone)]
pub struct TypeAlias {
    pub name: Ident,
    pub generics: Option<Generics>,
    pub ty: Type,
    pub visibility: Visibility,
    pub span: Span,
}

/// Constant definition
#[derive(Debug, Clone)]
pub struct ConstDef {
    pub name: Ident,
    pub ty: Type,
    pub value: Expr,
    pub visibility: Visibility,
    pub span: Span,
}

/// Static variable definition
#[derive(Debug, Clone)]
pub struct StaticDef {
    pub name: Ident,
    pub ty: Type,
    pub value: Expr,
    pub is_mut: bool,
    pub visibility: Visibility,
    pub span: Span,
}

/// Generic parameters
#[derive(Debug, Clone)]
pub struct Generics {
    pub params: Vec<GenericParam>,
    pub span: Span,
}

/// A single generic parameter
#[derive(Debug, Clone)]
pub enum GenericParam {
    Type(TypeParam),
    Const(ConstParam),
}

/// Type parameter (e.g., `T: Clone`)
#[derive(Debug, Clone)]
pub struct TypeParam {
    pub name: Ident,
    pub bounds: Vec<TypePath>,
    pub default: Option<Type>,
    pub span: Span,
}

/// Const generic parameter (e.g., `const N: usize`)
#[derive(Debug, Clone)]
pub struct ConstParam {
    pub name: Ident,
    pub ty: Type,
    pub default: Option<Expr>,
    pub span: Span,
}

/// Where clause for additional bounds
#[derive(Debug, Clone)]
pub struct WhereClause {
    pub predicates: Vec<WherePredicate>,
    pub span: Span,
}

/// A single predicate in a where clause
#[derive(Debug, Clone)]
pub struct WherePredicate {
    pub ty: Type,
    pub bounds: Vec<TypePath>,
    pub span: Span,
}

/// Type representation
#[derive(Debug, Clone)]
pub enum Type {
    /// Simple path type (e.g., `String`, `Vec<T>`)
    Path(TypePath),

    /// Tuple type (e.g., `(A, B, C)`)
    Tuple(Vec<Type>, Span),

    /// Array type (e.g., `[T; N]`)
    Array(Box<Type>, Box<Expr>, Span),

    /// Slice type (e.g., `[T]`)
    Slice(Box<Type>, Span),

    /// Reference type (e.g., `&T`, `&mut T`)
    Reference {
        mutable: bool,
        ty: Box<Type>,
        span: Span,
    },

    /// Function type (e.g., `fn(A, B) -> C`)
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
        span: Span,
    },

    /// Inferred type (`_`)
    Infer(Span),

    /// Never type (`!` or `never`)
    Never(Span),

    /// Unit type (`()`)
    Unit(Span),
}

/// Type path (e.g., `std::collections::HashMap<K, V>`)
#[derive(Debug, Clone)]
pub struct TypePath {
    pub segments: Vec<PathSegment>,
    pub span: Span,
}

/// Segment in a type path
#[derive(Debug, Clone)]
pub struct PathSegment {
    pub ident: Ident,
    pub generics: Option<Vec<GenericArg>>,
    pub span: Span,
}

/// Generic argument in a type application
#[derive(Debug, Clone)]
pub enum GenericArg {
    Type(Type),
    Const(Expr),
}

/// Block of statements
#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

/// Statement
#[derive(Debug, Clone)]
pub enum Stmt {
    /// Let binding
    Let {
        pattern: Pattern,
        ty: Option<Type>,
        value: Option<Expr>,
        span: Span,
    },

    /// Expression statement
    Expr(Expr),

    /// Item definition (function, struct, etc.)
    Item(Box<Item>),
}

/// Expression
#[derive(Debug, Clone)]
pub enum Expr {
    /// Literal value
    Literal(Literal, Span),

    /// Variable or path reference
    Path(ExprPath, Span),

    /// Binary operation
    Binary {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
        span: Span,
    },

    /// Unary operation
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
        span: Span,
    },

    /// Function call
    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
        span: Span,
    },

    /// Method call
    MethodCall {
        receiver: Box<Expr>,
        method: Ident,
        generics: Option<Vec<GenericArg>>,
        args: Vec<Expr>,
        span: Span,
    },

    /// Field access
    Field {
        expr: Box<Expr>,
        field: Ident,
        span: Span,
    },

    /// Index access
    Index {
        expr: Box<Expr>,
        index: Box<Expr>,
        span: Span,
    },

    /// Tuple expression
    Tuple(Vec<Expr>, Span),

    /// Array expression
    Array(Vec<Expr>, Span),

    /// Struct literal
    Struct {
        path: TypePath,
        fields: Vec<FieldInit>,
        rest: Option<Box<Expr>>,
        span: Span,
    },

    /// Block expression
    Block(Block),

    /// If expression
    If {
        condition: Box<Expr>,
        then_branch: Block,
        else_branch: Option<Box<Expr>>,
        span: Span,
    },

    /// Match expression
    Match {
        expr: Box<Expr>,
        arms: Vec<MatchArm>,
        span: Span,
    },

    /// Loop expression
    Loop {
        body: Block,
        label: Option<Ident>,
        span: Span,
    },

    /// While loop
    While {
        condition: Box<Expr>,
        body: Block,
        label: Option<Ident>,
        span: Span,
    },

    /// For loop
    For {
        pattern: Pattern,
        iter: Box<Expr>,
        body: Block,
        label: Option<Ident>,
        span: Span,
    },

    /// Break expression
    Break {
        label: Option<Ident>,
        value: Option<Box<Expr>>,
        span: Span,
    },

    /// Continue expression
    Continue {
        label: Option<Ident>,
        span: Span,
    },

    /// Return expression
    Return {
        value: Option<Box<Expr>>,
        span: Span,
    },

    /// Closure expression
    Closure {
        params: Vec<ClosureParam>,
        return_type: Option<Type>,
        body: Box<Expr>,
        is_async: bool,
        span: Span,
    },

    /// Await expression
    Await {
        expr: Box<Expr>,
        span: Span,
    },

    /// Try expression (?)
    Try {
        expr: Box<Expr>,
        span: Span,
    },

    /// Assignment
    Assign {
        target: Box<Expr>,
        value: Box<Expr>,
        span: Span,
    },

    /// Compound assignment (+=, -=, etc.)
    AssignOp {
        op: BinOp,
        target: Box<Expr>,
        value: Box<Expr>,
        span: Span,
    },

    /// Range expression
    Range {
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
        inclusive: bool,
        span: Span,
    },

    /// Cast expression
    Cast {
        expr: Box<Expr>,
        ty: Type,
        span: Span,
    },

    /// Spawn expression (Lemon-specific)
    Spawn {
        body: Block,
        span: Span,
    },

    /// Scope expression (Lemon-specific structured concurrency)
    Scope {
        body: Block,
        span: Span,
    },

    /// Select expression (Lemon-specific async)
    Select {
        arms: Vec<SelectArm>,
        span: Span,
    },

    // === OOP Expressions ===

    /// New expression: `new ClassName(args)` or `new ClassName { field: value }`
    New {
        class: TypePath,
        args: NewArgs,
        span: Span,
    },

    /// This reference: `this` or `this.field`
    This { span: Span },

    /// Super call: `super.method()` or `super(args)` in constructor
    Super { span: Span },
}

/// Arguments for the `new` expression
#[derive(Debug, Clone)]
pub enum NewArgs {
    /// Constructor-style: `new Foo(arg1, arg2)`
    Args(Vec<Expr>),
    /// Struct-style: `new Foo { field: value }`
    Fields(Vec<FieldInit>),
}

/// Literal value
#[derive(Debug, Clone)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(String),
    Char(char),
    Bool(bool),
}

/// Binary operator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

/// Unary operator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    Neg,
    Deref,
    Ref,
    RefMut,
}

/// Expression path (e.g., `std::io::Result`)
#[derive(Debug, Clone)]
pub struct ExprPath {
    pub segments: Vec<Ident>,
}

/// Field initialization in struct literal
#[derive(Debug, Clone)]
pub struct FieldInit {
    pub name: Ident,
    pub value: Option<Expr>,
    pub span: Span,
}

/// Match arm
#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Box<Expr>>,
    pub body: Expr,
    pub span: Span,
}

/// Select arm (for async select)
#[derive(Debug, Clone)]
pub struct SelectArm {
    pub pattern: Pattern,
    pub future: Expr,
    pub body: Expr,
    pub span: Span,
}

/// Closure parameter
#[derive(Debug, Clone)]
pub struct ClosureParam {
    pub pattern: Pattern,
    pub ty: Option<Type>,
    pub span: Span,
}

/// Pattern for matching
#[derive(Debug, Clone)]
pub enum Pattern {
    /// Wildcard pattern (`_`)
    Wildcard(Span),

    /// Identifier binding
    Ident {
        mutable: bool,
        name: Ident,
        span: Span,
    },

    /// Literal pattern
    Literal(Literal, Span),

    /// Tuple pattern
    Tuple(Vec<Pattern>, Span),

    /// Struct pattern
    Struct {
        path: TypePath,
        fields: Vec<FieldPattern>,
        rest: bool,
        span: Span,
    },

    /// Tuple struct/enum variant pattern
    TupleStruct {
        path: TypePath,
        fields: Vec<Pattern>,
        span: Span,
    },

    /// Path pattern (unit variant)
    Path(TypePath, Span),

    /// Or pattern
    Or(Vec<Pattern>, Span),

    /// Reference pattern
    Ref {
        mutable: bool,
        pattern: Box<Pattern>,
        span: Span,
    },

    /// Range pattern
    Range {
        start: Option<Box<Pattern>>,
        end: Option<Box<Pattern>>,
        inclusive: bool,
        span: Span,
    },

    /// Slice pattern
    Slice {
        patterns: Vec<Pattern>,
        span: Span,
    },
}

/// Field pattern in struct pattern
#[derive(Debug, Clone)]
pub struct FieldPattern {
    pub name: Ident,
    pub pattern: Option<Pattern>,
    pub span: Span,
}

/// Attribute (e.g., `#[test]`, `#[inline]`)
#[derive(Debug, Clone)]
pub struct Attribute {
    pub path: Vec<Ident>,
    pub args: Option<AttrArgs>,
    pub span: Span,
}

/// Attribute arguments
#[derive(Debug, Clone)]
pub enum AttrArgs {
    /// Simple parenthesized tokens
    Delimited(Vec<AttrToken>),

    /// Key-value style (e.g., `#[cfg(target_os = "linux")]`)
    Eq(AttrToken),
}

/// Token inside attribute
#[derive(Debug, Clone)]
pub enum AttrToken {
    Ident(String),
    Literal(Literal),
    Punct(char),
    Group(Vec<AttrToken>),
}

/// Annotation for schema fields (e.g., `@required`, `@range(0, 100)`)
#[derive(Debug, Clone)]
pub struct Annotation {
    pub name: Ident,
    pub args: Vec<Expr>,
    pub span: Span,
}

/// Visibility modifier
#[derive(Debug, Clone, Default)]
pub enum Visibility {
    #[default]
    Private,
    Public,
    Restricted(TypePath),
}

/// Identifier with span
#[derive(Debug, Clone)]
pub struct Ident {
    pub name: String,
    pub span: Span,
}

impl Ident {
    pub fn new(name: impl Into<String>, span: Span) -> Self {
        Self {
            name: name.into(),
            span,
        }
    }
}

impl Type {
    pub fn span(&self) -> Span {
        match self {
            Type::Path(p) => p.span,
            Type::Tuple(_, span) => *span,
            Type::Array(_, _, span) => *span,
            Type::Slice(_, span) => *span,
            Type::Reference { span, .. } => *span,
            Type::Function { span, .. } => *span,
            Type::Infer(span) => *span,
            Type::Never(span) => *span,
            Type::Unit(span) => *span,
        }
    }
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Literal(_, span) => *span,
            Expr::Path(_, span) => *span,
            Expr::Binary { span, .. } => *span,
            Expr::Unary { span, .. } => *span,
            Expr::Call { span, .. } => *span,
            Expr::MethodCall { span, .. } => *span,
            Expr::Field { span, .. } => *span,
            Expr::Index { span, .. } => *span,
            Expr::Tuple(_, span) => *span,
            Expr::Array(_, span) => *span,
            Expr::Struct { span, .. } => *span,
            Expr::Block(b) => b.span,
            Expr::If { span, .. } => *span,
            Expr::Match { span, .. } => *span,
            Expr::Loop { span, .. } => *span,
            Expr::While { span, .. } => *span,
            Expr::For { span, .. } => *span,
            Expr::Break { span, .. } => *span,
            Expr::Continue { span, .. } => *span,
            Expr::Return { span, .. } => *span,
            Expr::Closure { span, .. } => *span,
            Expr::Await { span, .. } => *span,
            Expr::Try { span, .. } => *span,
            Expr::Assign { span, .. } => *span,
            Expr::AssignOp { span, .. } => *span,
            Expr::Range { span, .. } => *span,
            Expr::Cast { span, .. } => *span,
            Expr::Spawn { span, .. } => *span,
            Expr::Scope { span, .. } => *span,
            Expr::Select { span, .. } => *span,
            // OOP expressions
            Expr::New { span, .. } => *span,
            Expr::This { span } => *span,
            Expr::Super { span } => *span,
        }
    }
}
