//! Item parsing for Lemon (functions, structs, enums, traits, etc.)

use super::*;

impl Parser {
    /// Parse a top-level item
    pub fn parse_item(&mut self) -> ParseResult<Item> {
        let attrs = self.parse_attributes()?;
        let vis = self.parse_visibility();

        // Check for abstract/final modifiers for classes
        let is_abstract = self.match_token(&Token::Abstract);
        let is_final = if !is_abstract { self.match_token(&Token::Final) } else { false };

        match self.current_token() {
            Some(Token::Fn) | Some(Token::Async) => {
                let func = self.parse_function(attrs, vis)?;
                Ok(Item::Function(func))
            }
            Some(Token::Struct) => {
                let def = self.parse_struct(attrs, vis)?;
                Ok(Item::Struct(def))
            }
            Some(Token::Enum) => {
                let def = self.parse_enum(attrs, vis)?;
                Ok(Item::Enum(def))
            }
            Some(Token::Trait) => {
                let def = self.parse_trait(attrs, vis)?;
                Ok(Item::Trait(def))
            }
            Some(Token::Impl) => {
                let block = self.parse_impl(attrs)?;
                Ok(Item::Impl(block))
            }
            Some(Token::Schema) => {
                let def = self.parse_schema(attrs, vis)?;
                Ok(Item::Schema(def))
            }
            Some(Token::Capability) => {
                let def = self.parse_capability(vis)?;
                Ok(Item::Capability(def))
            }
            Some(Token::Use) => {
                let decl = self.parse_use(vis)?;
                Ok(Item::Use(decl))
            }
            Some(Token::Mod) => {
                let decl = self.parse_mod(vis)?;
                Ok(Item::Mod(decl))
            }
            Some(Token::Type) => {
                let alias = self.parse_type_alias(attrs, vis)?;
                Ok(Item::TypeAlias(alias))
            }
            // OOP: class and interface
            Some(Token::Class) => {
                let def = self.parse_class(attrs, vis, is_abstract, is_final)?;
                Ok(Item::Class(def))
            }
            Some(Token::Interface) => {
                let def = self.parse_interface(attrs, vis)?;
                Ok(Item::Interface(def))
            }
            Some(Token::Let) => {
                let def = self.parse_const(vis)?;
                Ok(Item::Const(def))
            }
            Some(t) => Err(ParseError::UnexpectedToken {
                expected: "item (fn, struct, enum, trait, impl, schema, capability, class, interface, use, mod, type, let)".to_string(),
                found: t.clone(),
                span: self.current_span(),
            }),
            None => Err(ParseError::UnexpectedEof {
                span: self.prev_span(),
            }),
        }
    }

    fn parse_function(&mut self, attrs: Vec<Attribute>, vis: Visibility) -> ParseResult<Function> {
        let is_async = self.match_token(&Token::Async);
        let start = self.consume(&Token::Fn)?;
        let name = self.consume_ident()?;
        let generics = self.parse_generics()?;

        self.consume(&Token::LParen)?;
        let params = self.parse_params()?;
        self.consume(&Token::RParen)?;

        let return_type = if self.match_token(&Token::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };

        // Parse effects
        let effects = if self.match_token(&Token::Effects) {
            self.parse_effect_list()?
        } else {
            Vec::new()
        };

        // Parse required capabilities
        let capabilities = if self.match_token(&Token::Requires) {
            self.parse_capability_list()?
        } else {
            Vec::new()
        };

        let where_clause = self.parse_where_clause()?;

        let body = if self.check(&Token::LBrace) {
            Some(self.parse_block()?)
        } else {
            self.consume(&Token::Semi)?;
            None
        };

        let end = self.prev_span();
        Ok(Function {
            name,
            generics,
            params,
            return_type,
            effects,
            capabilities,
            where_clause,
            body,
            is_async,
            visibility: vis,
            attributes: attrs,
            span: start.merge(end),
        })
    }

    fn parse_params(&mut self) -> ParseResult<Vec<Param>> {
        let mut params = Vec::new();

        // Handle self parameter
        if self.check(&Token::Amp) || self.check(&Token::SelfLower) || self.check(&Token::Mut) {
            if let Some(param) = self.try_parse_self_param()? {
                params.push(param);
                if !self.match_token(&Token::Comma) {
                    return Ok(params);
                }
            }
        }

        while !self.check(&Token::RParen) {
            params.push(self.parse_param()?);
            if !self.match_token(&Token::Comma) {
                break;
            }
        }

        Ok(params)
    }

    fn try_parse_self_param(&mut self) -> ParseResult<Option<Param>> {
        let start = self.current_span();

        // &self or &mut self
        if self.match_token(&Token::Amp) {
            let mutable = self.match_token(&Token::Mut);
            if self.match_token(&Token::SelfLower) {
                let end = self.prev_span();
                let pattern = Pattern::Ident {
                    mutable: false,
                    name: Ident::new("self", end),
                    span: end,
                };
                let ty = Type::Reference {
                    mutable,
                    ty: Box::new(Type::Path(
                        TypePath {
                            segments: vec![PathSegment {
                                ident: Ident::new("Self", end),
                                generics: None,
                                span: end,
                            }],
                            span: end,
                        }
                    )),
                    span: start.merge(end),
                };
                return Ok(Some(Param {
                    pattern,
                    ty,
                    span: start.merge(end),
                }));
            }
            // Not a self param, backtrack would be needed - for now error
            return Err(ParseError::UnexpectedToken {
                expected: "self".to_string(),
                found: self.current_token().cloned().unwrap_or(Token::Error),
                span: self.current_span(),
            });
        }

        // mut self
        if self.check(&Token::Mut) && self.peek(1) == Some(&Token::SelfLower) {
            self.advance(); // mut
            self.advance(); // self
            let end = self.prev_span();
            let pattern = Pattern::Ident {
                mutable: true,
                name: Ident::new("self", end),
                span: end,
            };
            let ty = Type::Path(
                TypePath {
                    segments: vec![PathSegment {
                        ident: Ident::new("Self", end),
                        generics: None,
                        span: end,
                    }],
                    span: end,
                }
            );
            return Ok(Some(Param {
                pattern,
                ty,
                span: start.merge(end),
            }));
        }

        // self
        if self.match_token(&Token::SelfLower) {
            let end = self.prev_span();
            let pattern = Pattern::Ident {
                mutable: false,
                name: Ident::new("self", end),
                span: end,
            };
            let ty = Type::Path(
                TypePath {
                    segments: vec![PathSegment {
                        ident: Ident::new("Self", end),
                        generics: None,
                        span: end,
                    }],
                    span: end,
                }
            );
            return Ok(Some(Param {
                pattern,
                ty,
                span: start.merge(end),
            }));
        }

        Ok(None)
    }

    fn parse_param(&mut self) -> ParseResult<Param> {
        let pattern = self.parse_pattern()?;
        let start = pattern.span();

        self.consume(&Token::Colon)?;
        let ty = self.parse_type()?;

        let end = ty.span();
        Ok(Param {
            pattern,
            ty,
            span: start.merge(end),
        })
    }

    fn parse_effect_list(&mut self) -> ParseResult<Vec<Ident>> {
        self.consume(&Token::LBracket)?;

        let mut effects = Vec::new();
        if !self.check(&Token::RBracket) {
            effects.push(self.consume_ident()?);
            while self.match_token(&Token::Comma) {
                if self.check(&Token::RBracket) {
                    break;
                }
                effects.push(self.consume_ident()?);
            }
        }

        self.consume(&Token::RBracket)?;
        Ok(effects)
    }

    fn parse_capability_list(&mut self) -> ParseResult<Vec<Ident>> {
        self.consume(&Token::LBracket)?;

        let mut caps = Vec::new();
        if !self.check(&Token::RBracket) {
            caps.push(self.consume_ident()?);
            while self.match_token(&Token::Comma) {
                if self.check(&Token::RBracket) {
                    break;
                }
                caps.push(self.consume_ident()?);
            }
        }

        self.consume(&Token::RBracket)?;
        Ok(caps)
    }

    fn parse_struct(&mut self, attrs: Vec<Attribute>, vis: Visibility) -> ParseResult<StructDef> {
        let start = self.consume(&Token::Struct)?;
        let name = self.consume_ident()?;
        let generics = self.parse_generics()?;

        self.consume(&Token::LBrace)?;

        let mut fields = Vec::new();
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            let field_attrs = self.parse_attributes()?;
            let field_vis = self.parse_visibility();
            let field_name = self.consume_ident()?;
            self.consume(&Token::Colon)?;
            let field_ty = self.parse_type()?;

            let field_span = field_name.span.merge(field_ty.span());
            fields.push(Field {
                name: field_name,
                ty: field_ty,
                visibility: field_vis,
                attributes: field_attrs,
                span: field_span,
            });

            if !self.match_token(&Token::Comma) {
                break;
            }
        }

        let end = self.consume(&Token::RBrace)?;
        Ok(StructDef {
            name,
            generics,
            fields,
            visibility: vis,
            attributes: attrs,
            span: start.merge(end),
        })
    }

    fn parse_enum(&mut self, attrs: Vec<Attribute>, vis: Visibility) -> ParseResult<EnumDef> {
        let start = self.consume(&Token::Enum)?;
        let name = self.consume_ident()?;
        let generics = self.parse_generics()?;

        self.consume(&Token::LBrace)?;

        let mut variants = Vec::new();
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            let var_attrs = self.parse_attributes()?;
            let var_name = self.consume_ident()?;
            let var_start = var_name.span;

            let fields = if self.check(&Token::LParen) {
                self.consume(&Token::LParen)?;
                let mut types = Vec::new();
                if !self.check(&Token::RParen) {
                    types.push(self.parse_type()?);
                    while self.match_token(&Token::Comma) {
                        if self.check(&Token::RParen) {
                            break;
                        }
                        types.push(self.parse_type()?);
                    }
                }
                self.consume(&Token::RParen)?;
                VariantFields::Tuple(types)
            } else if self.check(&Token::LBrace) {
                self.consume(&Token::LBrace)?;
                let mut struct_fields = Vec::new();
                while !self.check(&Token::RBrace) && !self.is_at_end() {
                    let field_name = self.consume_ident()?;
                    self.consume(&Token::Colon)?;
                    let field_ty = self.parse_type()?;
                    let field_span = field_name.span.merge(field_ty.span());
                    struct_fields.push(Field {
                        name: field_name,
                        ty: field_ty,
                        visibility: Visibility::Private,
                        attributes: Vec::new(),
                        span: field_span,
                    });
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
                self.consume(&Token::RBrace)?;
                VariantFields::Struct(struct_fields)
            } else {
                VariantFields::Unit
            };

            let var_end = self.prev_span();
            variants.push(Variant {
                name: var_name,
                fields,
                attributes: var_attrs,
                span: var_start.merge(var_end),
            });

            if !self.match_token(&Token::Comma) {
                break;
            }
        }

        let end = self.consume(&Token::RBrace)?;
        Ok(EnumDef {
            name,
            generics,
            variants,
            visibility: vis,
            attributes: attrs,
            span: start.merge(end),
        })
    }

    fn parse_trait(&mut self, attrs: Vec<Attribute>, vis: Visibility) -> ParseResult<TraitDef> {
        let start = self.consume(&Token::Trait)?;
        let name = self.consume_ident()?;
        let generics = self.parse_generics()?;

        let super_traits = if self.match_token(&Token::Colon) {
            let mut traits = vec![self.parse_type_path()?];
            while self.match_token(&Token::Plus) {
                traits.push(self.parse_type_path()?);
            }
            traits
        } else {
            Vec::new()
        };

        self.consume(&Token::LBrace)?;

        let mut items = Vec::new();
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            let item_attrs = self.parse_attributes()?;

            if self.check(&Token::Fn) || self.check(&Token::Async) {
                let func = self.parse_function(item_attrs, Visibility::Public)?;
                items.push(TraitItem::Function(func));
            } else if self.match_token(&Token::Type) {
                let assoc_name = self.consume_ident()?;
                let assoc_start = assoc_name.span;

                let bounds = if self.match_token(&Token::Colon) {
                    let mut b = vec![self.parse_type_path()?];
                    while self.match_token(&Token::Plus) {
                        b.push(self.parse_type_path()?);
                    }
                    b
                } else {
                    Vec::new()
                };

                let default = if self.match_token(&Token::Eq) {
                    Some(self.parse_type()?)
                } else {
                    None
                };

                self.consume(&Token::Semi)?;

                items.push(TraitItem::Type(AssocType {
                    name: assoc_name,
                    bounds,
                    default,
                    span: assoc_start.merge(self.prev_span()),
                }));
            } else {
                return Err(ParseError::UnexpectedToken {
                    expected: "fn or type".to_string(),
                    found: self.current_token().cloned().unwrap_or(Token::Error),
                    span: self.current_span(),
                });
            }
        }

        let end = self.consume(&Token::RBrace)?;
        Ok(TraitDef {
            name,
            generics,
            super_traits,
            items,
            visibility: vis,
            attributes: attrs,
            span: start.merge(end),
        })
    }

    fn parse_impl(&mut self, attrs: Vec<Attribute>) -> ParseResult<ImplBlock> {
        let start = self.consume(&Token::Impl)?;
        let generics = self.parse_generics()?;

        // Parse the first type
        let first_type = self.parse_type()?;

        // Check if this is a trait impl (impl Trait for Type)
        let (trait_, self_type) = if self.match_token(&Token::For) {
            let self_type = self.parse_type()?;
            let trait_path = match first_type {
                Type::Path(p) => p,
                _ => {
                    return Err(ParseError::Custom {
                        message: "expected trait path".to_string(),
                        span: first_type.span(),
                    })
                }
            };
            (Some(trait_path), self_type)
        } else {
            (None, first_type)
        };

        let where_clause = self.parse_where_clause()?;

        self.consume(&Token::LBrace)?;

        let mut items = Vec::new();
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            let item_attrs = self.parse_attributes()?;
            let item_vis = self.parse_visibility();

            if self.check(&Token::Fn) || self.check(&Token::Async) {
                let func = self.parse_function(item_attrs, item_vis)?;
                items.push(ImplItem::Function(func));
            } else if self.match_token(&Token::Type) {
                let name = self.consume_ident()?;
                let type_generics = self.parse_generics()?;
                self.consume(&Token::Eq)?;
                let ty = self.parse_type()?;
                self.consume(&Token::Semi)?;

                items.push(ImplItem::Type(TypeAlias {
                    name: name.clone(),
                    generics: type_generics,
                    ty,
                    visibility: item_vis,
                    span: name.span.merge(self.prev_span()),
                }));
            } else {
                return Err(ParseError::UnexpectedToken {
                    expected: "fn or type".to_string(),
                    found: self.current_token().cloned().unwrap_or(Token::Error),
                    span: self.current_span(),
                });
            }
        }

        let end = self.consume(&Token::RBrace)?;
        Ok(ImplBlock {
            generics,
            trait_,
            self_type,
            where_clause,
            items,
            attributes: attrs,
            span: start.merge(end),
        })
    }

    fn parse_schema(&mut self, attrs: Vec<Attribute>, vis: Visibility) -> ParseResult<SchemaDef> {
        let start = self.consume(&Token::Schema)?;
        let name = self.consume_ident()?;
        let generics = self.parse_generics()?;

        self.consume(&Token::LBrace)?;

        let mut fields = Vec::new();
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            // Parse annotations (@required, @optional, etc.)
            let mut annotations = Vec::new();
            while self.match_token(&Token::At) {
                let ann_name = self.consume_ident()?;
                let ann_start = ann_name.span;

                let args = if self.match_token(&Token::LParen) {
                    let mut a = Vec::new();
                    if !self.check(&Token::RParen) {
                        a.push(self.parse_expr()?);
                        while self.match_token(&Token::Comma) {
                            if self.check(&Token::RParen) {
                                break;
                            }
                            a.push(self.parse_expr()?);
                        }
                    }
                    self.consume(&Token::RParen)?;
                    a
                } else {
                    Vec::new()
                };

                annotations.push(Annotation {
                    name: ann_name,
                    args,
                    span: ann_start.merge(self.prev_span()),
                });
            }

            let field_name = self.consume_ident()?;
            self.consume(&Token::Colon)?;
            let field_ty = self.parse_type()?;

            let field_span = field_name.span.merge(field_ty.span());
            fields.push(SchemaField {
                name: field_name,
                ty: field_ty,
                annotations,
                span: field_span,
            });

            if !self.match_token(&Token::Comma) {
                break;
            }
        }

        let end = self.consume(&Token::RBrace)?;
        Ok(SchemaDef {
            name,
            generics,
            fields,
            visibility: vis,
            attributes: attrs,
            span: start.merge(end),
        })
    }

    fn parse_capability(&mut self, vis: Visibility) -> ParseResult<CapabilityDef> {
        let start = self.consume(&Token::Capability)?;
        let name = self.consume_ident()?;

        let composed_of = if self.match_token(&Token::Eq) {
            let mut caps = vec![self.consume_ident()?];
            while self.match_token(&Token::Plus) {
                caps.push(self.consume_ident()?);
            }
            caps
        } else {
            Vec::new()
        };

        self.consume(&Token::Semi)?;

        Ok(CapabilityDef {
            name,
            composed_of,
            visibility: vis,
            span: start.merge(self.prev_span()),
        })
    }

    fn parse_use(&mut self, vis: Visibility) -> ParseResult<UseDecl> {
        let start = self.consume(&Token::Use)?;
        let tree = self.parse_use_tree()?;
        self.consume(&Token::Semi)?;

        Ok(UseDecl {
            tree,
            visibility: vis,
            span: start.merge(self.prev_span()),
        })
    }

    fn parse_use_tree(&mut self) -> ParseResult<UseTree> {
        // Handle glob (*)
        if self.match_token(&Token::Star) {
            return Ok(UseTree::Glob);
        }

        // Handle group ({...})
        if self.check(&Token::LBrace) {
            self.consume(&Token::LBrace)?;
            let mut items = Vec::new();
            if !self.check(&Token::RBrace) {
                items.push(self.parse_use_tree()?);
                while self.match_token(&Token::Comma) {
                    if self.check(&Token::RBrace) {
                        break;
                    }
                    items.push(self.parse_use_tree()?);
                }
            }
            self.consume(&Token::RBrace)?;
            return Ok(UseTree::Group(items));
        }

        let ident = self.consume_ident()?;

        // Check for rename (as)
        if self.match_token(&Token::As) {
            let alias = self.consume_ident()?;
            return Ok(UseTree::Rename(ident, alias));
        }

        // Check for path continuation (::)
        if self.match_token(&Token::ColonColon) {
            let rest = self.parse_use_tree()?;
            return Ok(UseTree::Path(ident, Box::new(rest)));
        }

        Ok(UseTree::Name(ident))
    }

    fn parse_mod(&mut self, vis: Visibility) -> ParseResult<ModDecl> {
        let start = self.consume(&Token::Mod)?;
        let name = self.consume_ident()?;

        let items = if self.check(&Token::LBrace) {
            self.consume(&Token::LBrace)?;
            let mut mod_items = Vec::new();
            while !self.check(&Token::RBrace) && !self.is_at_end() {
                mod_items.push(self.parse_item()?);
            }
            self.consume(&Token::RBrace)?;
            Some(mod_items)
        } else {
            self.consume(&Token::Semi)?;
            None
        };

        Ok(ModDecl {
            name,
            items,
            visibility: vis,
            span: start.merge(self.prev_span()),
        })
    }

    fn parse_type_alias(
        &mut self,
        _attrs: Vec<Attribute>,
        vis: Visibility,
    ) -> ParseResult<TypeAlias> {
        let start = self.consume(&Token::Type)?;
        let name = self.consume_ident()?;
        let generics = self.parse_generics()?;
        self.consume(&Token::Eq)?;
        let ty = self.parse_type()?;
        self.consume(&Token::Semi)?;

        Ok(TypeAlias {
            name,
            generics,
            ty,
            visibility: vis,
            span: start.merge(self.prev_span()),
        })
    }

    /// Parse a module-level constant definition
    ///
    /// Syntax: `[pub] let NAME [: Type] = value;`
    fn parse_const(&mut self, vis: Visibility) -> ParseResult<ConstDef> {
        let start = self.consume(&Token::Let)?;
        let name = self.consume_ident()?;

        // Optional type annotation
        let ty = if self.match_token(&Token::Colon) {
            self.parse_type()?
        } else {
            // Default to inferred type (we'll use Int as placeholder)
            Type::Path(TypePath {
                segments: vec![PathSegment {
                    ident: Ident::new("_", name.span),
                    generics: None,
                    span: name.span,
                }],
                span: name.span,
            })
        };

        self.consume(&Token::Eq)?;
        let value = self.parse_expr()?;
        self.consume(&Token::Semi)?;

        Ok(ConstDef {
            name,
            ty,
            value,
            visibility: vis,
            span: start.merge(self.prev_span()),
        })
    }

    // =========================================================================
    // OOP: Class and Interface Parsing
    // =========================================================================

    /// Parse class definition
    ///
    /// Syntax:
    /// ```text
    /// [abstract|final] class Name<T> extends Parent implements Interface1, Interface2 {
    ///     [pub|protected|private] field: Type,
    ///     delegate handler: EventHandler,
    ///
    ///     fn new(...) -> Self { ... }
    ///     [pub|protected] [override|final] fn method(this) -> T { ... }
    ///     static fn create() -> Self { ... }
    /// }
    /// ```
    fn parse_class(
        &mut self,
        attrs: Vec<Attribute>,
        vis: Visibility,
        is_abstract: bool,
        is_final: bool,
    ) -> ParseResult<ClassDef> {
        let start = self.consume(&Token::Class)?;
        let name = self.consume_ident()?;
        let generics = self.parse_generics()?;

        // Parse extends (single inheritance)
        let extends = if self.match_token(&Token::Extends) {
            Some(self.parse_type_path()?)
        } else {
            None
        };

        // Parse implements (multiple interfaces)
        let mut implements = Vec::new();
        if self.match_token(&Token::Implements) {
            implements.push(self.parse_type_path()?);
            while self.match_token(&Token::Comma) {
                implements.push(self.parse_type_path()?);
            }
        }

        self.consume(&Token::LBrace)?;

        let mut members = Vec::new();
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            members.push(self.parse_class_member()?);
        }

        let end = self.consume(&Token::RBrace)?;

        Ok(ClassDef {
            name,
            generics,
            extends,
            implements,
            members,
            visibility: vis,
            is_abstract,
            is_final,
            attributes: attrs,
            span: start.merge(end),
        })
    }

    /// Parse a class member (field, method, delegate)
    fn parse_class_member(&mut self) -> ParseResult<ClassMember> {
        let attrs = self.parse_attributes()?;
        let member_vis = self.parse_member_visibility();

        // Check for delegate
        if self.match_token(&Token::Delegate) {
            let name = self.consume_ident()?;
            self.consume(&Token::Colon)?;
            let ty = self.parse_type()?;
            self.match_token(&Token::Semi);

            return Ok(ClassMember::Delegate(DelegateField {
                name: name.clone(),
                ty,
                visibility: member_vis,
                span: name.span.merge(self.prev_span()),
            }));
        }

        // Check for static
        let is_static = self.match_token(&Token::Static);

        // Check for abstract/override/final
        let is_abstract = self.match_token(&Token::Abstract);
        let is_override = if !is_abstract { self.match_token(&Token::Override) } else { false };
        let is_final = if !is_abstract && !is_override {
            self.match_token(&Token::Final)
        } else {
            false
        };

        // Check for async
        let is_async = self.match_token(&Token::Async);

        // Method or field?
        if self.check(&Token::Fn) {
            let method = self.parse_method(attrs, member_vis, is_async, is_abstract, is_override, is_final)?;

            // Determine method type
            if method.name.name == "new" {
                Ok(ClassMember::Constructor(method))
            } else if is_static {
                Ok(ClassMember::StaticMethod(method))
            } else {
                Ok(ClassMember::Method(method))
            }
        } else {
            // Field (static or instance)
            let is_final = self.match_token(&Token::Final);
            let name = self.consume_ident()?;
            self.consume(&Token::Colon)?;
            let ty = self.parse_type()?;

            let default = if self.match_token(&Token::Eq) {
                Some(self.parse_expr()?)
            } else {
                None
            };

            self.match_token(&Token::Semi);

            Ok(ClassMember::Field(ClassField {
                name: name.clone(),
                ty,
                default,
                visibility: member_vis,
                is_static,
                is_final,
                attributes: attrs,
                span: name.span.merge(self.prev_span()),
            }))
        }
    }

    /// Parse a method (used in classes and interfaces)
    fn parse_method(
        &mut self,
        attrs: Vec<Attribute>,
        vis: MemberVisibility,
        is_async: bool,
        is_abstract: bool,
        is_override: bool,
        is_final: bool,
    ) -> ParseResult<Method> {
        let start = self.consume(&Token::Fn)?;
        let name = self.consume_ident()?;
        let generics = self.parse_generics()?;

        self.consume(&Token::LParen)?;
        let params = self.parse_method_params()?;
        self.consume(&Token::RParen)?;

        let return_type = if self.match_token(&Token::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };

        // Parse effects
        let effects = if self.match_token(&Token::Effects) {
            self.parse_effect_list()?
        } else {
            Vec::new()
        };

        // Parse capabilities
        let capabilities = if self.match_token(&Token::Requires) {
            self.parse_capability_list()?
        } else {
            Vec::new()
        };

        // Body (optional for abstract/interface methods)
        let body = if self.check(&Token::LBrace) {
            Some(self.parse_block()?)
        } else {
            self.match_token(&Token::Semi);
            None
        };

        Ok(Method {
            name,
            generics,
            params,
            return_type,
            effects,
            capabilities,
            body,
            visibility: vis,
            is_async,
            is_abstract,
            is_override,
            is_final,
            attributes: attrs,
            span: start.merge(self.prev_span()),
        })
    }

    /// Parse method parameters (handles `this` and `mut this`)
    fn parse_method_params(&mut self) -> ParseResult<Vec<Param>> {
        let mut params = Vec::new();

        // Check for this parameter
        if self.check(&Token::This) || self.check(&Token::Mut) {
            let start = self.current_span();
            let mutable = self.match_token(&Token::Mut);

            if self.match_token(&Token::This) {
                let end = self.prev_span();
                // Create implicit self parameter
                let pattern = Pattern::Ident {
                    mutable,
                    name: Ident::new("this", end),
                    span: end,
                };
                let ty = Type::Path(TypePath {
                    segments: vec![PathSegment {
                        ident: Ident::new("Self", end),
                        generics: None,
                        span: end,
                    }],
                    span: end,
                });
                params.push(Param {
                    pattern,
                    ty,
                    span: start.merge(end),
                });

                if !self.match_token(&Token::Comma) {
                    return Ok(params);
                }
            }
        }

        // Regular parameters
        while !self.check(&Token::RParen) && !self.is_at_end() {
            params.push(self.parse_param()?);
            if !self.match_token(&Token::Comma) {
                break;
            }
        }

        Ok(params)
    }

    /// Parse interface definition
    fn parse_interface(
        &mut self,
        attrs: Vec<Attribute>,
        vis: Visibility,
    ) -> ParseResult<InterfaceDef> {
        let start = self.consume(&Token::Interface)?;
        let name = self.consume_ident()?;
        let generics = self.parse_generics()?;

        // Parse extends (interfaces can extend multiple interfaces)
        let mut extends = Vec::new();
        if self.match_token(&Token::Extends) {
            extends.push(self.parse_type_path()?);
            while self.match_token(&Token::Comma) {
                extends.push(self.parse_type_path()?);
            }
        }

        self.consume(&Token::LBrace)?;

        let mut methods = Vec::new();
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            let method_attrs = self.parse_attributes()?;
            let is_async = self.match_token(&Token::Async);

            let method = self.parse_method(
                method_attrs,
                MemberVisibility::Public, // Interface methods are always public
                is_async,
                false, // Not abstract (interface methods are implicitly abstract if no body)
                false, // Not override
                false, // Not final
            )?;
            methods.push(method);
        }

        let end = self.consume(&Token::RBrace)?;

        Ok(InterfaceDef {
            name,
            generics,
            extends,
            methods,
            visibility: vis,
            attributes: attrs,
            span: start.merge(end),
        })
    }

    /// Parse member visibility (pub, protected, private)
    fn parse_member_visibility(&mut self) -> MemberVisibility {
        if self.match_token(&Token::Pub) {
            MemberVisibility::Public
        } else if self.match_token(&Token::Protected) {
            MemberVisibility::Protected
        } else if self.match_token(&Token::Private) {
            MemberVisibility::Private
        } else {
            MemberVisibility::Private // Default to private
        }
    }
}
