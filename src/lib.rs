//! Lemon Programming Language
//!
//! An AI-native, human-friendly programming language with:
//! - Dual execution (JIT script mode + native compilation)
//! - Capability-based security
//! - Effect tracking
//! - Deterministic execution mode

pub mod ast;
pub mod codegen;
pub mod lexer;
pub mod parser;
pub mod runtime;
pub mod types;

pub use ast::*;
pub use lexer::Token;
