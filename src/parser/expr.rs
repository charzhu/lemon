//! Expression parsing for Lemon

use super::*;

impl Parser {
    /// Parse an expression (entry point)
    pub fn parse_expr(&mut self) -> ParseResult<Expr> {
        self.parse_expr_assignment()
    }

    fn parse_expr_assignment(&mut self) -> ParseResult<Expr> {
        let expr = self.parse_expr_or()?;

        if self.match_token(&Token::Eq) {
            let value = self.parse_expr_assignment()?;
            let span = expr.span().merge(value.span());
            return Ok(Expr::Assign {
                target: Box::new(expr),
                value: Box::new(value),
                span,
            });
        }

        // Compound assignment operators
        let op = match self.current_token() {
            Some(Token::Plus) if self.peek(1) == Some(&Token::Eq) => Some(BinOp::Add),
            Some(Token::Minus) if self.peek(1) == Some(&Token::Eq) => Some(BinOp::Sub),
            Some(Token::Star) if self.peek(1) == Some(&Token::Eq) => Some(BinOp::Mul),
            Some(Token::Slash) if self.peek(1) == Some(&Token::Eq) => Some(BinOp::Div),
            Some(Token::Percent) if self.peek(1) == Some(&Token::Eq) => Some(BinOp::Rem),
            _ => None,
        };

        if let Some(op) = op {
            self.advance(); // Skip operator
            self.advance(); // Skip =
            let value = self.parse_expr_assignment()?;
            let span = expr.span().merge(value.span());
            return Ok(Expr::AssignOp {
                op,
                target: Box::new(expr),
                value: Box::new(value),
                span,
            });
        }

        Ok(expr)
    }

    fn parse_expr_or(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_expr_and()?;

        while self.match_token(&Token::PipePipe) {
            let right = self.parse_expr_and()?;
            let span = left.span().merge(right.span());
            left = Expr::Binary {
                op: BinOp::Or,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }

        Ok(left)
    }

    fn parse_expr_and(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_expr_comparison()?;

        while self.match_token(&Token::AmpAmp) {
            let right = self.parse_expr_comparison()?;
            let span = left.span().merge(right.span());
            left = Expr::Binary {
                op: BinOp::And,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }

        Ok(left)
    }

    fn parse_expr_comparison(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_expr_bitor()?;

        loop {
            let op = match self.current_token() {
                Some(Token::EqEq) => BinOp::Eq,
                Some(Token::BangEq) => BinOp::Ne,
                Some(Token::Lt) => BinOp::Lt,
                Some(Token::LtEq) => BinOp::Le,
                Some(Token::Gt) => BinOp::Gt,
                Some(Token::GtEq) => BinOp::Ge,
                _ => break,
            };

            self.advance();
            let right = self.parse_expr_bitor()?;
            let span = left.span().merge(right.span());
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }

        Ok(left)
    }

    fn parse_expr_bitor(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_expr_bitxor()?;

        while self.match_token(&Token::Pipe) {
            let right = self.parse_expr_bitxor()?;
            let span = left.span().merge(right.span());
            left = Expr::Binary {
                op: BinOp::BitOr,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }

        Ok(left)
    }

    fn parse_expr_bitxor(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_expr_bitand()?;

        while self.match_token(&Token::Caret) {
            let right = self.parse_expr_bitand()?;
            let span = left.span().merge(right.span());
            left = Expr::Binary {
                op: BinOp::BitXor,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }

        Ok(left)
    }

    fn parse_expr_bitand(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_expr_shift()?;

        while self.match_token(&Token::Amp) {
            let right = self.parse_expr_shift()?;
            let span = left.span().merge(right.span());
            left = Expr::Binary {
                op: BinOp::BitAnd,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }

        Ok(left)
    }

    fn parse_expr_shift(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_expr_additive()?;

        loop {
            let op = match self.current_token() {
                Some(Token::LtLt) => BinOp::Shl,
                Some(Token::GtGt) => BinOp::Shr,
                _ => break,
            };

            self.advance();
            let right = self.parse_expr_additive()?;
            let span = left.span().merge(right.span());
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }

        Ok(left)
    }

    fn parse_expr_additive(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_expr_multiplicative()?;

        loop {
            let op = match self.current_token() {
                // Don't consume + or - if followed by = (compound assignment)
                Some(Token::Plus) if self.peek(1) != Some(&Token::Eq) => BinOp::Add,
                Some(Token::Minus) if self.peek(1) != Some(&Token::Eq) => BinOp::Sub,
                _ => break,
            };

            self.advance();
            let right = self.parse_expr_multiplicative()?;
            let span = left.span().merge(right.span());
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }

        Ok(left)
    }

    fn parse_expr_multiplicative(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_expr_unary()?;

        loop {
            let op = match self.current_token() {
                // Don't consume *, /, % if followed by = (compound assignment)
                Some(Token::Star) if self.peek(1) != Some(&Token::Eq) => BinOp::Mul,
                Some(Token::Slash) if self.peek(1) != Some(&Token::Eq) => BinOp::Div,
                Some(Token::Percent) if self.peek(1) != Some(&Token::Eq) => BinOp::Rem,
                _ => break,
            };

            self.advance();
            let right = self.parse_expr_unary()?;
            let span = left.span().merge(right.span());
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }

        Ok(left)
    }

    fn parse_expr_unary(&mut self) -> ParseResult<Expr> {
        let span = self.current_span();

        let op = match self.current_token() {
            Some(Token::Bang) => Some(UnaryOp::Not),
            Some(Token::Minus) => Some(UnaryOp::Neg),
            Some(Token::Star) => Some(UnaryOp::Deref),
            Some(Token::Amp) => {
                self.advance();
                let mutable = self.match_token(&Token::Mut);
                let expr = self.parse_expr_unary()?;
                let end = expr.span();
                return Ok(Expr::Unary {
                    op: if mutable { UnaryOp::RefMut } else { UnaryOp::Ref },
                    expr: Box::new(expr),
                    span: span.merge(end),
                });
            }
            _ => None,
        };

        if let Some(op) = op {
            self.advance();
            let expr = self.parse_expr_unary()?;
            let end = expr.span();
            return Ok(Expr::Unary {
                op,
                expr: Box::new(expr),
                span: span.merge(end),
            });
        }

        self.parse_expr_postfix()
    }

    fn parse_expr_postfix(&mut self) -> ParseResult<Expr> {
        let mut expr = self.parse_expr_primary()?;

        loop {
            if self.match_token(&Token::Dot) {
                // Check for await
                if self.check(&Token::Await) {
                    self.advance();
                    let span = expr.span().merge(self.prev_span());
                    expr = Expr::Await {
                        expr: Box::new(expr),
                        span,
                    };
                    continue;
                }

                // Field access or method call
                let field = self.consume_ident()?;

                if self.check(&Token::LParen) {
                    // Method call
                    let args = self.parse_call_args()?;
                    let span = expr.span().merge(self.prev_span());
                    expr = Expr::MethodCall {
                        receiver: Box::new(expr),
                        method: field,
                        generics: None,
                        args,
                        span,
                    };
                } else {
                    // Field access
                    let span = expr.span().merge(field.span);
                    expr = Expr::Field {
                        expr: Box::new(expr),
                        field,
                        span,
                    };
                }
            } else if self.check(&Token::LParen) {
                // Function call
                let args = self.parse_call_args()?;
                let span = expr.span().merge(self.prev_span());
                expr = Expr::Call {
                    func: Box::new(expr),
                    args,
                    span,
                };
            } else if self.check(&Token::LBracket) {
                // Index access
                self.advance();
                let index = self.parse_expr()?;
                let end = self.consume(&Token::RBracket)?;
                let span = expr.span().merge(end);
                expr = Expr::Index {
                    expr: Box::new(expr),
                    index: Box::new(index),
                    span,
                };
            } else if self.match_token(&Token::Question) {
                // Try operator
                let span = expr.span().merge(self.prev_span());
                expr = Expr::Try {
                    expr: Box::new(expr),
                    span,
                };
            } else if self.match_token(&Token::As) {
                // Cast expression
                let ty = self.parse_type()?;
                let span = expr.span().merge(ty.span());
                expr = Expr::Cast {
                    expr: Box::new(expr),
                    ty,
                    span,
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_call_args(&mut self) -> ParseResult<Vec<Expr>> {
        self.consume(&Token::LParen)?;

        let mut args = Vec::new();

        if !self.check(&Token::RParen) {
            args.push(self.parse_expr()?);
            while self.match_token(&Token::Comma) {
                if self.check(&Token::RParen) {
                    break;
                }
                args.push(self.parse_expr()?);
            }
        }

        self.consume(&Token::RParen)?;
        Ok(args)
    }

    fn parse_expr_primary(&mut self) -> ParseResult<Expr> {
        let span = self.current_span();

        // Literals
        match self.current_token() {
            Some(Token::Int(n)) => {
                let n = *n;
                self.advance();
                return Ok(Expr::Literal(Literal::Int(n), span));
            }
            Some(Token::HexInt(n)) | Some(Token::BinInt(n)) => {
                let n = *n;
                self.advance();
                return Ok(Expr::Literal(Literal::Int(n), span));
            }
            Some(Token::Float(n)) => {
                let n = *n;
                self.advance();
                return Ok(Expr::Literal(Literal::Float(n), span));
            }
            Some(Token::String(s)) => {
                let s = s.clone();
                self.advance();
                return Ok(Expr::Literal(Literal::String(s), span));
            }
            Some(Token::Char(c)) => {
                let c = *c;
                self.advance();
                return Ok(Expr::Literal(Literal::Char(c), span));
            }
            Some(Token::True) => {
                self.advance();
                return Ok(Expr::Literal(Literal::Bool(true), span));
            }
            Some(Token::False) => {
                self.advance();
                return Ok(Expr::Literal(Literal::Bool(false), span));
            }
            _ => {}
        }

        // Parenthesized expression or tuple
        if self.check(&Token::LParen) {
            return self.parse_paren_or_tuple();
        }

        // Array literal
        if self.check(&Token::LBracket) {
            return self.parse_array_expr();
        }

        // Block expression
        if self.check(&Token::LBrace) {
            let block = self.parse_block()?;
            return Ok(Expr::Block(block));
        }

        // If expression
        if self.check(&Token::If) {
            return self.parse_if_expr();
        }

        // Match expression
        if self.check(&Token::Match) {
            return self.parse_match_expr();
        }

        // Loop expressions
        if self.check(&Token::Loop) {
            return self.parse_loop_expr();
        }

        if self.check(&Token::While) {
            return self.parse_while_expr();
        }

        if self.check(&Token::For) {
            return self.parse_for_expr();
        }

        // Return expression
        if self.match_token(&Token::Return) {
            let value = if self.current_token().map(|t| t.can_start_expr()).unwrap_or(false) {
                Some(Box::new(self.parse_expr()?))
            } else {
                None
            };
            return Ok(Expr::Return {
                value,
                span: span.merge(self.prev_span()),
            });
        }

        // Break expression
        if self.match_token(&Token::Break) {
            let value = if self.current_token().map(|t| t.can_start_expr()).unwrap_or(false) {
                Some(Box::new(self.parse_expr()?))
            } else {
                None
            };
            return Ok(Expr::Break {
                label: None,
                value,
                span: span.merge(self.prev_span()),
            });
        }

        // Continue expression
        if self.match_token(&Token::Continue) {
            return Ok(Expr::Continue {
                label: None,
                span,
            });
        }

        // Spawn expression
        if self.match_token(&Token::Spawn) {
            let body = self.parse_block()?;
            return Ok(Expr::Spawn {
                body,
                span: span.merge(self.prev_span()),
            });
        }

        // Scope expression
        if self.match_token(&Token::Scope) {
            let body = self.parse_block()?;
            return Ok(Expr::Scope {
                body,
                span: span.merge(self.prev_span()),
            });
        }

        // Select expression
        if self.check(&Token::Select) {
            return self.parse_select_expr();
        }

        // Closure
        if self.check(&Token::Pipe) || self.check(&Token::PipePipe) {
            return self.parse_closure();
        }

        // Async closure
        if self.check(&Token::Async) {
            return self.parse_async_closure();
        }

        // Path expression (including struct literals)
        if let Some(Token::Ident(_)) = self.current_token() {
            return self.parse_path_or_struct_expr();
        }

        // Self type used as expression (e.g., Self { ... })
        if self.check(&Token::SelfUpper) {
            return self.parse_path_or_struct_expr();
        }

        // Self
        if self.match_token(&Token::SelfLower) {
            return Ok(Expr::Path(
                ExprPath {
                    segments: vec![Ident::new("self", span)],
                },
                span,
            ));
        }

        // OOP: this expression
        if self.match_token(&Token::This) {
            return Ok(Expr::This { span });
        }

        // OOP: super expression (super.method() or super::Type)
        if self.match_token(&Token::Super) {
            // Check for super::method or super::Type syntax
            if self.match_token(&Token::ColonColon) {
                let method = self.consume_ident()?;
                // This becomes a method call on super: super::method(args)
                if self.check(&Token::LParen) {
                    let args = self.parse_call_args()?;
                    let end = self.prev_span();
                    return Ok(Expr::MethodCall {
                        receiver: Box::new(Expr::Super { span }),
                        method,
                        generics: None,
                        args,
                        span: span.merge(end),
                    });
                }
                // super::Type path reference
                let end = self.prev_span();
                return Ok(Expr::Path(
                    ExprPath { segments: vec![Ident::new("super", span), method] },
                    span.merge(end),
                ));
            }
            return Ok(Expr::Super { span });
        }

        // OOP: new expression (new ClassName(...) or new ClassName { field: value })
        if self.match_token(&Token::New) {
            let type_path = self.parse_type_path()?;

            // Check for struct-style initialization: new Foo { field: value }
            if self.check(&Token::LBrace) {
                // Look ahead for field initialization
                let is_struct = self.peek(1).map(|t| matches!(t, Token::Ident(_))).unwrap_or(false)
                    && self.peek(2) == Some(&Token::Colon);

                if is_struct {
                    self.consume(&Token::LBrace)?;
                    let mut fields = Vec::new();

                    while !self.check(&Token::RBrace) && !self.is_at_end() {
                        let name = self.consume_ident()?;
                        let field_span = name.span;

                        let value = if self.match_token(&Token::Colon) {
                            Some(self.parse_expr()?)
                        } else {
                            None
                        };

                        fields.push(FieldInit {
                            name,
                            value,
                            span: field_span,
                        });

                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                    }

                    let end = self.consume(&Token::RBrace)?;
                    return Ok(Expr::New {
                        class: type_path,
                        args: NewArgs::Fields(fields),
                        span: span.merge(end),
                    });
                }
            }

            // Constructor-style: new Foo(arg1, arg2)
            let args = self.parse_call_args()?;
            let end = self.prev_span();
            return Ok(Expr::New {
                class: type_path,
                args: NewArgs::Args(args),
                span: span.merge(end),
            });
        }

        Err(ParseError::UnexpectedToken {
            expected: "expression".to_string(),
            found: self.current_token().cloned().unwrap_or(Token::Error),
            span,
        })
    }

    fn parse_paren_or_tuple(&mut self) -> ParseResult<Expr> {
        let start = self.consume(&Token::LParen)?;

        if self.match_token(&Token::RParen) {
            // Unit literal
            return Ok(Expr::Tuple(Vec::new(), start.merge(self.prev_span())));
        }

        let first = self.parse_expr()?;

        if self.match_token(&Token::RParen) {
            // Parenthesized expression
            return Ok(first);
        }

        // Tuple
        self.consume(&Token::Comma)?;
        let mut exprs = vec![first];

        if !self.check(&Token::RParen) {
            exprs.push(self.parse_expr()?);
            while self.match_token(&Token::Comma) {
                if self.check(&Token::RParen) {
                    break;
                }
                exprs.push(self.parse_expr()?);
            }
        }

        let end = self.consume(&Token::RParen)?;
        Ok(Expr::Tuple(exprs, start.merge(end)))
    }

    fn parse_array_expr(&mut self) -> ParseResult<Expr> {
        let start = self.consume(&Token::LBracket)?;

        let mut exprs = Vec::new();

        if !self.check(&Token::RBracket) {
            exprs.push(self.parse_expr()?);
            while self.match_token(&Token::Comma) {
                if self.check(&Token::RBracket) {
                    break;
                }
                exprs.push(self.parse_expr()?);
            }
        }

        let end = self.consume(&Token::RBracket)?;
        Ok(Expr::Array(exprs, start.merge(end)))
    }

    fn parse_if_expr(&mut self) -> ParseResult<Expr> {
        let start = self.consume(&Token::If)?;
        let condition = self.parse_expr()?;
        let then_branch = self.parse_block()?;

        let else_branch = if self.match_token(&Token::Else) {
            if self.check(&Token::If) {
                Some(Box::new(self.parse_if_expr()?))
            } else {
                let block = self.parse_block()?;
                Some(Box::new(Expr::Block(block)))
            }
        } else {
            None
        };

        let end = self.prev_span();
        Ok(Expr::If {
            condition: Box::new(condition),
            then_branch,
            else_branch,
            span: start.merge(end),
        })
    }

    fn parse_match_expr(&mut self) -> ParseResult<Expr> {
        let start = self.consume(&Token::Match)?;
        let expr = self.parse_expr()?;
        self.consume(&Token::LBrace)?;

        let mut arms = Vec::new();

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            arms.push(self.parse_match_arm()?);

            // Optional comma
            self.match_token(&Token::Comma);
        }

        let end = self.consume(&Token::RBrace)?;
        Ok(Expr::Match {
            expr: Box::new(expr),
            arms,
            span: start.merge(end),
        })
    }

    fn parse_match_arm(&mut self) -> ParseResult<MatchArm> {
        let pattern = self.parse_pattern()?;
        let start = pattern.span();

        let guard = if self.match_token(&Token::If) {
            Some(Box::new(self.parse_expr()?))
        } else {
            None
        };

        self.consume(&Token::FatArrow)?;
        let body = self.parse_expr()?;

        let end = body.span();
        Ok(MatchArm {
            pattern,
            guard,
            body,
            span: start.merge(end),
        })
    }

    fn parse_loop_expr(&mut self) -> ParseResult<Expr> {
        let start = self.consume(&Token::Loop)?;
        let body = self.parse_block()?;

        Ok(Expr::Loop {
            body,
            label: None,
            span: start.merge(self.prev_span()),
        })
    }

    fn parse_while_expr(&mut self) -> ParseResult<Expr> {
        let start = self.consume(&Token::While)?;
        let condition = self.parse_expr()?;
        let body = self.parse_block()?;

        Ok(Expr::While {
            condition: Box::new(condition),
            body,
            label: None,
            span: start.merge(self.prev_span()),
        })
    }

    fn parse_for_expr(&mut self) -> ParseResult<Expr> {
        let start = self.consume(&Token::For)?;
        let pattern = self.parse_pattern()?;
        self.consume(&Token::In)?;
        let iter = self.parse_expr()?;
        let body = self.parse_block()?;

        Ok(Expr::For {
            pattern,
            iter: Box::new(iter),
            body,
            label: None,
            span: start.merge(self.prev_span()),
        })
    }

    fn parse_select_expr(&mut self) -> ParseResult<Expr> {
        let start = self.consume(&Token::Select)?;
        self.consume(&Token::LBrace)?;

        let mut arms = Vec::new();

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            let pattern = self.parse_pattern()?;
            self.consume(&Token::Eq)?;
            let future = self.parse_expr()?;
            self.consume(&Token::FatArrow)?;
            let body = self.parse_expr()?;

            let span = pattern.span().merge(body.span());
            arms.push(SelectArm {
                pattern,
                future,
                body,
                span,
            });

            self.match_token(&Token::Comma);
        }

        let end = self.consume(&Token::RBrace)?;
        Ok(Expr::Select {
            arms,
            span: start.merge(end),
        })
    }

    fn parse_closure(&mut self) -> ParseResult<Expr> {
        let start = self.current_span();

        // Handle || for no params
        if self.match_token(&Token::PipePipe) {
            let body = self.parse_expr()?;
            let body_span = body.span();
            return Ok(Expr::Closure {
                params: Vec::new(),
                return_type: None,
                body: Box::new(body),
                is_async: false,
                span: start.merge(body_span),
            });
        }

        self.consume(&Token::Pipe)?;
        let params = self.parse_closure_params()?;
        self.consume(&Token::Pipe)?;

        let return_type = if self.match_token(&Token::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let body = self.parse_expr()?;

        Ok(Expr::Closure {
            params,
            return_type,
            body: Box::new(body),
            is_async: false,
            span: start.merge(self.prev_span()),
        })
    }

    fn parse_async_closure(&mut self) -> ParseResult<Expr> {
        let start = self.consume(&Token::Async)?;

        // Handle async || for no params
        if self.match_token(&Token::PipePipe) {
            let body = self.parse_expr()?;
            let body_span = body.span();
            return Ok(Expr::Closure {
                params: Vec::new(),
                return_type: None,
                body: Box::new(body),
                is_async: true,
                span: start.merge(body_span),
            });
        }

        self.consume(&Token::Pipe)?;
        let params = self.parse_closure_params()?;
        self.consume(&Token::Pipe)?;

        let return_type = if self.match_token(&Token::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let body = self.parse_expr()?;

        Ok(Expr::Closure {
            params,
            return_type,
            body: Box::new(body),
            is_async: true,
            span: start.merge(self.prev_span()),
        })
    }

    fn parse_closure_params(&mut self) -> ParseResult<Vec<ClosureParam>> {
        let mut params = Vec::new();

        if !self.check(&Token::Pipe) {
            params.push(self.parse_closure_param()?);
            while self.match_token(&Token::Comma) {
                if self.check(&Token::Pipe) {
                    break;
                }
                params.push(self.parse_closure_param()?);
            }
        }

        Ok(params)
    }

    fn parse_closure_param(&mut self) -> ParseResult<ClosureParam> {
        let pattern = self.parse_pattern()?;
        let start = pattern.span();

        let ty = if self.match_token(&Token::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let end = self.prev_span();
        Ok(ClosureParam {
            pattern,
            ty,
            span: start.merge(end),
        })
    }

    fn parse_path_or_struct_expr(&mut self) -> ParseResult<Expr> {
        let start = self.current_span();
        let mut segments = vec![self.consume_ident()?];

        while self.match_token(&Token::ColonColon) {
            segments.push(self.consume_ident()?);
        }

        // Check for struct literal
        if self.check(&Token::LBrace) {
            // Look ahead to distinguish block from struct literal
            // It's a struct if:
            // 1. We see `ident:` after `{` (explicit field), OR
            // 2. We see `ident,` or `ident}` after `{` (shorthand field), OR
            // 3. The path has multiple segments (e.g., Foo::Bar { })
            // 4. `}` immediately after `{` (empty struct)
            let peek1 = self.peek(1);
            let peek2 = self.peek(2);

            let is_struct = match peek1 {
                // Empty struct: `Self {}`
                Some(Token::RBrace) => true,
                // Explicit field: `Self { field: value }`
                Some(Token::Ident(_)) if peek2 == Some(&Token::Colon) => true,
                // Shorthand field: `Self { field }` or `Self { field, ... }`
                Some(Token::Ident(_)) if peek2 == Some(&Token::Comma) || peek2 == Some(&Token::RBrace) => true,
                // Multiple path segments imply struct
                _ => segments.len() > 1,
            };

            if is_struct {
                return self.parse_struct_literal(segments, start);
            }
        }

        let end = self.prev_span();
        Ok(Expr::Path(
            ExprPath { segments },
            start.merge(end),
        ))
    }

    fn parse_struct_literal(&mut self, segments: Vec<Ident>, start: Span) -> ParseResult<Expr> {
        let path = TypePath {
            segments: segments
                .into_iter()
                .map(|ident| PathSegment {
                    span: ident.span,
                    ident,
                    generics: None,
                })
                .collect(),
            span: start.merge(self.prev_span()),
        };

        self.consume(&Token::LBrace)?;

        let mut fields = Vec::new();
        let mut rest = None;

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            // Check for ..rest
            if self.match_token(&Token::DotDot) {
                rest = Some(Box::new(self.parse_expr()?));
                self.match_token(&Token::Comma);
                break;
            }

            let name = self.consume_ident()?;
            let field_span = name.span;

            let value = if self.match_token(&Token::Colon) {
                Some(self.parse_expr()?)
            } else {
                None
            };

            fields.push(FieldInit {
                name,
                value,
                span: field_span,
            });

            if !self.match_token(&Token::Comma) {
                break;
            }
        }

        let end = self.consume(&Token::RBrace)?;
        Ok(Expr::Struct {
            path,
            fields,
            rest,
            span: start.merge(end),
        })
    }
}
