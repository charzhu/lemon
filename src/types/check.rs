//! Type checker for Lemon
//!
//! Performs type inference and checking on the AST.

use super::*;
use crate::ast::{self, Block, Expr, Item, SourceFile, Stmt};

/// Type checker state
pub struct TypeChecker {
    env: TypeEnv,
    errors: Vec<TypeError>,
    current_effects: Vec<Effect>,
    current_capabilities: Vec<Capability>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            env: TypeEnv::new(),
            errors: Vec::new(),
            current_effects: Vec::new(),
            current_capabilities: Vec::new(),
        }
    }

    /// Check a complete source file
    pub fn check_file(&mut self, file: &SourceFile) -> Result<(), Vec<TypeError>> {
        // First pass: collect all type and function definitions
        for item in &file.items {
            self.collect_item(item);
        }

        // Second pass: type check function bodies
        for item in &file.items {
            self.check_item(item);
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }

    /// Collect type/function definitions (first pass)
    fn collect_item(&mut self, item: &Item) {
        match item {
            Item::Function(func) => {
                let sig = self.function_signature(func);
                self.env.define_function(sig);
            }
            Item::Struct(def) => {
                let type_def = self.struct_definition(def);
                self.env.define_type(type_def);
            }
            Item::Enum(def) => {
                let type_def = self.enum_definition(def);
                self.env.define_type(type_def);
            }
            Item::Schema(def) => {
                let type_def = self.schema_definition(def);
                self.env.define_type(type_def);
            }
            Item::TypeAlias(alias) => {
                let ty = self.resolve_type(&alias.ty);
                self.env.define_type(TypeDef {
                    name: alias.name.name.clone(),
                    generics: Vec::new(), // TODO: handle generics
                    kind: TypeDefKind::Alias(ty),
                });
            }
            _ => {}
        }
    }

    /// Type check an item (second pass)
    fn check_item(&mut self, item: &Item) {
        match item {
            Item::Function(func) => {
                if let Some(body) = &func.body {
                    self.check_function(func, body);
                }
            }
            Item::Impl(impl_block) => {
                for impl_item in &impl_block.items {
                    if let ast::ImplItem::Function(func) = impl_item {
                        if let Some(body) = &func.body {
                            self.check_function(func, body);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn function_signature(&mut self, func: &ast::Function) -> FunctionSig {
        let params: Vec<(String, Ty)> = func
            .params
            .iter()
            .map(|p| {
                let name = self.pattern_name(&p.pattern);
                let ty = self.resolve_type(&p.ty);
                (name, ty)
            })
            .collect();

        let return_type = func
            .return_type
            .as_ref()
            .map(|t| self.resolve_type(t))
            .unwrap_or(Ty::Unit);

        let effects = func
            .effects
            .iter()
            .map(|e| self.resolve_effect(&e.name))
            .collect();

        let capabilities = func
            .capabilities
            .iter()
            .map(|c| self.resolve_capability(&c.name))
            .collect();

        FunctionSig {
            name: func.name.name.clone(),
            generics: Vec::new(), // TODO
            params,
            return_type,
            effects,
            capabilities,
            is_async: func.is_async,
        }
    }

    fn struct_definition(&mut self, def: &ast::StructDef) -> TypeDef {
        let fields: Vec<(String, Ty)> = def
            .fields
            .iter()
            .map(|f| (f.name.name.clone(), self.resolve_type(&f.ty)))
            .collect();

        TypeDef {
            name: def.name.name.clone(),
            generics: Vec::new(), // TODO
            kind: TypeDefKind::Struct(fields),
        }
    }

    fn enum_definition(&mut self, def: &ast::EnumDef) -> TypeDef {
        let variants: Vec<(String, VariantDef)> = def
            .variants
            .iter()
            .map(|v| {
                let variant_def = match &v.fields {
                    ast::VariantFields::Unit => VariantDef::Unit,
                    ast::VariantFields::Tuple(types) => {
                        VariantDef::Tuple(types.iter().map(|t| self.resolve_type(t)).collect())
                    }
                    ast::VariantFields::Struct(fields) => VariantDef::Struct(
                        fields
                            .iter()
                            .map(|f| (f.name.name.clone(), self.resolve_type(&f.ty)))
                            .collect(),
                    ),
                };
                (v.name.name.clone(), variant_def)
            })
            .collect();

        TypeDef {
            name: def.name.name.clone(),
            generics: Vec::new(), // TODO
            kind: TypeDefKind::Enum(variants),
        }
    }

    fn schema_definition(&mut self, def: &ast::SchemaDef) -> TypeDef {
        let fields: Vec<SchemaFieldDef> = def
            .fields
            .iter()
            .map(|f| {
                let mut required = true;
                let mut validations = Vec::new();

                for ann in &f.annotations {
                    match ann.name.name.as_str() {
                        "required" => required = true,
                        "optional" => required = false,
                        "range" => {
                            // Parse range validation
                            if ann.args.len() >= 2 {
                                let min = self.expr_to_i64(&ann.args[0]);
                                let max = self.expr_to_i64(&ann.args[1]);
                                validations.push(Validation::Range { min, max });
                            }
                        }
                        "length" => {
                            if ann.args.len() >= 2 {
                                let min = self.expr_to_usize(&ann.args[0]);
                                let max = self.expr_to_usize(&ann.args[1]);
                                validations.push(Validation::Length { min, max });
                            }
                        }
                        "pattern" => {
                            if let Some(pattern) = ann.args.first().and_then(|e| self.expr_to_string(e)) {
                                validations.push(Validation::Pattern(pattern));
                            }
                        }
                        _ => {}
                    }
                }

                SchemaFieldDef {
                    name: f.name.name.clone(),
                    ty: self.resolve_type(&f.ty),
                    required,
                    default: None, // TODO
                    validations,
                }
            })
            .collect();

        TypeDef {
            name: def.name.name.clone(),
            generics: Vec::new(), // TODO
            kind: TypeDefKind::Schema(fields),
        }
    }

    fn check_function(&mut self, func: &ast::Function, body: &Block) {
        // Set up the function's effects and capabilities
        self.current_effects = func
            .effects
            .iter()
            .map(|e| self.resolve_effect(&e.name))
            .collect();

        self.current_capabilities = func
            .capabilities
            .iter()
            .map(|c| self.resolve_capability(&c.name))
            .collect();

        // Add parameters to environment
        for param in &func.params {
            let name = self.pattern_name(&param.pattern);
            let ty = self.resolve_type(&param.ty);
            self.env.define_var(name, ty);
        }

        // Check the body
        let body_ty = self.check_block(body);

        // Verify return type
        let expected_ret = func
            .return_type
            .as_ref()
            .map(|t| self.resolve_type(t))
            .unwrap_or(Ty::Unit);

        if body_ty != expected_ret && body_ty != Ty::Never && body_ty != Ty::Error {
            self.errors.push(TypeError::type_mismatch(
                expected_ret,
                body_ty,
                body.span,
            ));
        }
    }

    fn check_block(&mut self, block: &Block) -> Ty {
        let mut last_ty = Ty::Unit;

        for stmt in &block.stmts {
            last_ty = self.check_stmt(stmt);
        }

        last_ty
    }

    fn check_stmt(&mut self, stmt: &Stmt) -> Ty {
        match stmt {
            Stmt::Let { pattern, ty, value, span } => {
                let var_ty = if let Some(explicit_ty) = ty {
                    self.resolve_type(explicit_ty)
                } else if let Some(val) = value {
                    self.check_expr(val)
                } else {
                    self.env.fresh_var()
                };

                if let Some(val) = value {
                    let val_ty = self.check_expr(val);
                    if var_ty != val_ty && val_ty != Ty::Error {
                        self.errors.push(TypeError::type_mismatch(
                            var_ty.clone(),
                            val_ty,
                            *span,
                        ));
                    }
                }

                let name = self.pattern_name(pattern);
                self.env.define_var(name, var_ty);
                Ty::Unit
            }
            Stmt::Expr(expr) => self.check_expr(expr),
            Stmt::Item(item) => {
                self.check_item(item);
                Ty::Unit
            }
        }
    }

    fn check_expr(&mut self, expr: &Expr) -> Ty {
        match expr {
            Expr::Literal(lit, _) => self.literal_type(lit),

            Expr::Path(path, span) => {
                if path.segments.len() == 1 {
                    let name = &path.segments[0].name;
                    if let Some(ty) = self.env.lookup_var(name) {
                        ty.clone()
                    } else if let Some(ty) = Ty::from_primitive(name) {
                        ty
                    } else {
                        self.errors.push(TypeError::undefined_var(name, *span));
                        Ty::Error
                    }
                } else {
                    // Multi-segment path - could be module::item
                    Ty::Error // TODO
                }
            }

            Expr::Binary { op, left, right, span } => {
                let left_ty = self.check_expr(left);
                let right_ty = self.check_expr(right);
                self.check_binary_op(*op, left_ty, right_ty, *span)
            }

            Expr::Unary { op, expr, span } => {
                let inner_ty = self.check_expr(expr);
                self.check_unary_op(*op, inner_ty, *span)
            }

            Expr::Call { func, args, span } => {
                let func_ty = self.check_expr(func);
                let arg_tys: Vec<Ty> = args.iter().map(|a| self.check_expr(a)).collect();

                match func_ty {
                    Ty::Function { params, ret, effects } => {
                        // Check argument count
                        if params.len() != arg_tys.len() {
                            self.errors.push(TypeError {
                                message: format!(
                                    "expected {} arguments, found {}",
                                    params.len(),
                                    arg_tys.len()
                                ),
                                span: *span,
                                kind: TypeErrorKind::InvalidOperation,
                            });
                        }

                        // Check argument types
                        for (param, arg) in params.iter().zip(arg_tys.iter()) {
                            if param != arg && *arg != Ty::Error {
                                self.errors.push(TypeError::type_mismatch(
                                    param.clone(),
                                    arg.clone(),
                                    *span,
                                ));
                            }
                        }

                        // Check effects
                        for effect in &effects {
                            if !self.current_effects.contains(effect) {
                                self.errors.push(TypeError {
                                    message: format!("missing effect: {:?}", effect),
                                    span: *span,
                                    kind: TypeErrorKind::MissingEffect(effect.clone()),
                                });
                            }
                        }

                        *ret
                    }
                    Ty::Error => Ty::Error,
                    _ => {
                        self.errors.push(TypeError {
                            message: "not a function".to_string(),
                            span: *span,
                            kind: TypeErrorKind::InvalidOperation,
                        });
                        Ty::Error
                    }
                }
            }

            Expr::If { condition, then_branch, else_branch, span } => {
                let cond_ty = self.check_expr(condition);
                if cond_ty != Ty::Bool && cond_ty != Ty::Error {
                    self.errors.push(TypeError::type_mismatch(
                        Ty::Bool,
                        cond_ty,
                        condition.span(),
                    ));
                }

                let then_ty = self.check_block(then_branch);

                if let Some(else_expr) = else_branch {
                    let else_ty = self.check_expr(else_expr);
                    if then_ty != else_ty && then_ty != Ty::Error && else_ty != Ty::Error {
                        self.errors.push(TypeError::type_mismatch(
                            then_ty.clone(),
                            else_ty,
                            *span,
                        ));
                    }
                    then_ty
                } else {
                    Ty::Unit
                }
            }

            Expr::Block(block) => self.check_block(block),

            Expr::Return { value, .. } => {
                if let Some(val) = value {
                    self.check_expr(val);
                }
                Ty::Never
            }

            Expr::Loop { body, .. } => {
                self.check_block(body);
                Ty::Never // Unless there's a break with a value
            }

            Expr::While { condition, body, .. } => {
                let cond_ty = self.check_expr(condition);
                if cond_ty != Ty::Bool && cond_ty != Ty::Error {
                    self.errors.push(TypeError::type_mismatch(
                        Ty::Bool,
                        cond_ty,
                        condition.span(),
                    ));
                }
                self.check_block(body);
                Ty::Unit
            }

            Expr::For { pattern, iter, body, .. } => {
                let iter_ty = self.check_expr(iter);
                // TODO: Check that iter_ty implements Iterator
                let name = self.pattern_name(pattern);
                self.env.define_var(name, Ty::Error); // TODO: Get element type
                self.check_block(body);
                Ty::Unit
            }

            Expr::Tuple(exprs, _) => {
                let types: Vec<Ty> = exprs.iter().map(|e| self.check_expr(e)).collect();
                Ty::Tuple(types)
            }

            Expr::Array(exprs, span) => {
                if exprs.is_empty() {
                    return Ty::Array(Box::new(self.env.fresh_var()), 0);
                }

                let first_ty = self.check_expr(&exprs[0]);
                for expr in &exprs[1..] {
                    let ty = self.check_expr(expr);
                    if ty != first_ty && ty != Ty::Error {
                        self.errors.push(TypeError::type_mismatch(
                            first_ty.clone(),
                            ty,
                            *span,
                        ));
                    }
                }

                Ty::Array(Box::new(first_ty), exprs.len())
            }

            Expr::Field { expr, field, span } => {
                let expr_ty = self.check_expr(expr);
                self.check_field_access(expr_ty, &field.name, *span)
            }

            Expr::Try { expr, span } => {
                let inner_ty = self.check_expr(expr);
                // TODO: Check that inner_ty is Result<T, E> and return T
                // For now, just return the type
                inner_ty
            }

            Expr::Await { expr, span } => {
                let inner_ty = self.check_expr(expr);
                // TODO: Check that inner_ty is Future<Output = T> and return T
                inner_ty
            }

            _ => {
                // TODO: Implement remaining expressions
                Ty::Error
            }
        }
    }

    fn literal_type(&self, lit: &ast::Literal) -> Ty {
        match lit {
            ast::Literal::Int(_) => Ty::Int(IntTy::I64),
            ast::Literal::Float(_) => Ty::Float(FloatTy::F64),
            ast::Literal::String(_) => Ty::String,
            ast::Literal::Char(_) => Ty::Char,
            ast::Literal::Bool(_) => Ty::Bool,
        }
    }

    fn check_binary_op(&mut self, op: ast::BinOp, left: Ty, right: Ty, span: Span) -> Ty {
        use ast::BinOp::*;

        match op {
            Add | Sub | Mul | Div | Rem => {
                if left != right {
                    self.errors.push(TypeError::type_mismatch(left.clone(), right, span));
                }
                left
            }
            Eq | Ne | Lt | Le | Gt | Ge => {
                if left != right {
                    self.errors.push(TypeError::type_mismatch(left, right, span));
                }
                Ty::Bool
            }
            And | Or => {
                if left != Ty::Bool {
                    self.errors.push(TypeError::type_mismatch(Ty::Bool, left, span));
                }
                if right != Ty::Bool {
                    self.errors.push(TypeError::type_mismatch(Ty::Bool, right, span));
                }
                Ty::Bool
            }
            BitAnd | BitOr | BitXor | Shl | Shr => {
                if left != right {
                    self.errors.push(TypeError::type_mismatch(left.clone(), right, span));
                }
                left
            }
        }
    }

    fn check_unary_op(&mut self, op: ast::UnaryOp, inner: Ty, span: Span) -> Ty {
        use ast::UnaryOp::*;

        match op {
            Not => {
                if inner != Ty::Bool {
                    self.errors.push(TypeError::type_mismatch(Ty::Bool, inner, span));
                }
                Ty::Bool
            }
            Neg => inner,
            Deref => {
                match inner {
                    Ty::Reference { ty, .. } => *ty,
                    _ => {
                        self.errors.push(TypeError {
                            message: "cannot dereference non-reference type".to_string(),
                            span,
                            kind: TypeErrorKind::InvalidOperation,
                        });
                        Ty::Error
                    }
                }
            }
            Ref | RefMut => {
                Ty::Reference {
                    mutable: op == RefMut,
                    ty: Box::new(inner),
                }
            }
        }
    }

    fn check_field_access(&mut self, expr_ty: Ty, field: &str, span: Span) -> Ty {
        match expr_ty {
            Ty::Named { name, .. } => {
                if let Some(type_def) = self.env.lookup_type(&name) {
                    match &type_def.kind {
                        TypeDefKind::Struct(fields) => {
                            if let Some((_, ty)) = fields.iter().find(|(n, _)| n == field) {
                                return ty.clone();
                            }
                        }
                        TypeDefKind::Schema(fields) => {
                            if let Some(f) = fields.iter().find(|f| f.name == field) {
                                return f.ty.clone();
                            }
                        }
                        _ => {}
                    }
                }
                self.errors.push(TypeError {
                    message: format!("no field '{}' on type '{}'", field, name),
                    span,
                    kind: TypeErrorKind::InvalidOperation,
                });
                Ty::Error
            }
            Ty::Tuple(types) => {
                if let Ok(idx) = field.parse::<usize>() {
                    if idx < types.len() {
                        return types[idx].clone();
                    }
                }
                self.errors.push(TypeError {
                    message: format!("no field '{}' on tuple", field),
                    span,
                    kind: TypeErrorKind::InvalidOperation,
                });
                Ty::Error
            }
            Ty::Error => Ty::Error,
            _ => {
                self.errors.push(TypeError {
                    message: "field access on non-struct type".to_string(),
                    span,
                    kind: TypeErrorKind::InvalidOperation,
                });
                Ty::Error
            }
        }
    }

    // Helper methods

    fn resolve_type(&mut self, ty: &ast::Type) -> Ty {
        match ty {
            ast::Type::Path(path) => {
                if path.segments.len() == 1 {
                    let name = &path.segments[0].ident.name;
                    if let Some(prim) = Ty::from_primitive(name) {
                        return prim;
                    }
                    Ty::Named {
                        name: name.clone(),
                        generics: Vec::new(), // TODO
                    }
                } else {
                    // Multi-segment path
                    Ty::Named {
                        name: path.segments.last().unwrap().ident.name.clone(),
                        generics: Vec::new(),
                    }
                }
            }
            ast::Type::Tuple(types, _) => {
                Ty::Tuple(types.iter().map(|t| self.resolve_type(t)).collect())
            }
            ast::Type::Array(elem, size, _) => {
                let elem_ty = self.resolve_type(elem);
                // TODO: Evaluate const expr for size
                Ty::Array(Box::new(elem_ty), 0)
            }
            ast::Type::Slice(elem, _) => {
                Ty::Slice(Box::new(self.resolve_type(elem)))
            }
            ast::Type::Reference { mutable, ty, .. } => {
                Ty::Reference {
                    mutable: *mutable,
                    ty: Box::new(self.resolve_type(ty)),
                }
            }
            ast::Type::Function { params, return_type, .. } => {
                Ty::Function {
                    params: params.iter().map(|t| self.resolve_type(t)).collect(),
                    ret: Box::new(self.resolve_type(return_type)),
                    effects: Vec::new(),
                }
            }
            ast::Type::Unit(_) => Ty::Unit,
            ast::Type::Never(_) => Ty::Never,
            ast::Type::Infer(_) => self.env.fresh_var(),
        }
    }

    fn resolve_effect(&self, name: &str) -> Effect {
        match name {
            "io" => Effect::Io,
            "random" => Effect::Random,
            "time" => Effect::Time,
            "panic" => Effect::Panic,
            _ => Effect::Custom(name.to_string()),
        }
    }

    fn resolve_capability(&self, name: &str) -> Capability {
        match name {
            "FileRead" => Capability::FileRead,
            "FileWrite" => Capability::FileWrite,
            "NetworkAccess" => Capability::NetworkAccess,
            "ProcessSpawn" => Capability::ProcessSpawn,
            "EnvAccess" => Capability::EnvAccess,
            "TimeAccess" => Capability::TimeAccess,
            "RandomAccess" => Capability::RandomAccess,
            _ => Capability::Custom(name.to_string()),
        }
    }

    fn pattern_name(&self, pattern: &ast::Pattern) -> String {
        match pattern {
            ast::Pattern::Ident { name, .. } => name.name.clone(),
            ast::Pattern::Wildcard(_) => "_".to_string(),
            _ => "_".to_string(),
        }
    }

    fn expr_to_i64(&self, expr: &Expr) -> Option<i64> {
        if let Expr::Literal(ast::Literal::Int(n), _) = expr {
            Some(*n)
        } else {
            None
        }
    }

    fn expr_to_usize(&self, expr: &Expr) -> Option<usize> {
        self.expr_to_i64(expr).and_then(|n| usize::try_from(n).ok())
    }

    fn expr_to_string(&self, expr: &Expr) -> Option<String> {
        if let Expr::Literal(ast::Literal::String(s), _) = expr {
            Some(s.clone())
        } else {
            None
        }
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}
