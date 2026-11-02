//! Deterministic LL(1) + Pratt expression parser for Myelin (`.myl`).

use myelin_ast::*;
use myelin_diag::{Diagnostic, Span};
use myelin_lexer::{Token, TokenKind, tokenize};

pub struct ParseOutput {
    pub module: Module,
    pub diagnostics: Vec<Diagnostic>,
}

/// Tokenizes and parses a `.myl` source buffer into a [`Module`] AST and diagnostics.
pub fn parse_source(src: &str) -> ParseOutput {
    let lex = tokenize(src);
    let mut parser = Parser::new(lex.tokens, lex.diagnostics);
    let module = parser.parse_module();
    ParseOutput {
        module,
        diagnostics: parser.diagnostics,
    }
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    diagnostics: Vec<Diagnostic>,
}

impl Parser {
    fn new(tokens: Vec<Token>, diagnostics: Vec<Diagnostic>) -> Self {
        Self {
            tokens,
            pos: 0,
            diagnostics,
        }
    }

    #[inline]
    fn peek(&self) -> &Token {
        self.tokens
            .get(self.pos)
            .unwrap_or_else(|| self.tokens.last().expect("token stream ends with Eof"))
    }

    #[inline]
    fn peek_at(&self, offset: usize) -> &Token {
        self.tokens
            .get(self.pos + offset)
            .unwrap_or_else(|| self.tokens.last().expect("token stream ends with Eof"))
    }

    #[inline]
    fn at(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(kind)
    }

    #[inline]
    fn advance(&mut self) -> Token {
        let tok = self.peek().clone();
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
        tok
    }

    fn eat(&mut self, expected: &TokenKind) -> bool {
        if self.at(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, expected: &TokenKind, fix: &str) -> Option<Token> {
        if self.at(expected) {
            Some(self.advance())
        } else {
            let found = self.peek();
            self.diagnostics.push(Diagnostic::error(
                "E011",
                found.span,
                format!(
                    "expected '{}' found '{}'",
                    expected.describe(),
                    found.kind.describe()
                ),
                fix,
            ));
            None
        }
    }

    fn expect_ident(&mut self, fix: &str) -> Option<(String, Span)> {
        match &self.peek().kind {
            TokenKind::Ident(_) => {
                let tok = self.advance();
                if let TokenKind::Ident(s) = tok.kind {
                    Some((s, tok.span))
                } else {
                    unreachable!()
                }
            }
            _ => {
                let found = self.peek();
                self.diagnostics.push(Diagnostic::error(
                    "E012",
                    found.span,
                    format!("expected identifier found '{}'", found.kind.describe()),
                    fix,
                ));
                None
            }
        }
    }

    fn skip_newlines(&mut self) {
        while self.eat(&TokenKind::Newline) {}
    }

    fn recover_to_newline(&mut self) {
        while !matches!(
            self.peek().kind,
            TokenKind::Newline | TokenKind::Dedent | TokenKind::Eof
        ) {
            self.advance();
        }
        let _ = self.eat(&TokenKind::Newline);
    }

    fn parse_module(&mut self) -> Module {
        let start_span = self.peek().span;
        let mut items = Vec::new();
        self.skip_newlines();

        while !self.at(&TokenKind::Eof) {
            if let Some(item) = self.parse_item() {
                items.push(item);
            } else {
                self.recover_to_newline();
                // Drain stray indent/dedent blocks during top-level error recovery
                while matches!(self.peek().kind, TokenKind::Indent | TokenKind::Dedent) {
                    self.advance();
                }
            }
            self.skip_newlines();
        }

        let end_span = self.peek().span;
        Module {
            items,
            span: start_span.merge(end_span),
        }
    }

    fn parse_item(&mut self) -> Option<Item> {
        match self.peek().kind {
            TokenKind::HashMod => self.parse_mod_decl().map(Item::Mod),
            TokenKind::HashUse => self.parse_use_decl().map(Item::Use),
            TokenKind::HashRec => self.parse_rec_decl().map(Item::Rec),
            TokenKind::HashSum => self.parse_sum_decl().map(Item::Sum),
            TokenKind::HashExt => self.parse_ext_decl().map(Item::Ext),
            TokenKind::HashFn => self.parse_fn_decl().map(Item::Fn),
            TokenKind::HashTest => self.parse_test_decl().map(Item::Test),
            _ => {
                let bad = self.peek();
                self.diagnostics.push(Diagnostic::error(
                    "E013",
                    bad.span,
                    format!("expected top_level_directive found '{}'", bad.kind.describe()),
                    "start top-level declarations at column 0 with #mod, #use, #rec, #sum, #ext, #fn, or #test",
                ));
                None
            }
        }
    }

    fn parse_path(&mut self, fix: &str) -> Option<Path> {
        let (first, mut span) = self.expect_ident(fix)?;
        let mut segments = vec![first];
        while self.eat(&TokenKind::Dot) {
            let (seg, seg_span) = self.expect_ident("provide identifier segment after '.'")?;
            span = span.merge(seg_span);
            segments.push(seg);
        }
        Some(Path { segments, span })
    }

    fn parse_capabilities(&mut self) -> Vec<Capability> {
        let mut caps = Vec::new();
        while self.at(&TokenKind::Bang) {
            let bang = self.advance();
            if let Some(path) =
                self.parse_path("write capability name after '!', e.g. !db.rw or !fs.r")
            {
                let span = bang.span.merge(path.span);
                caps.push(Capability { path, span });
            } else {
                break;
            }
        }
        caps
    }

    fn parse_mod_decl(&mut self) -> Option<ModDecl> {
        let start = self.advance().span;
        let path = self.parse_path("write module path after #mod, e.g. #mod bank.ledger")?;
        let _ = self.expect(&TokenKind::Newline, "end #mod declaration with a newline");
        Some(ModDecl {
            span: start.merge(path.span),
            path,
        })
    }

    fn parse_use_decl(&mut self) -> Option<UseDecl> {
        let start = self.advance().span;
        let path = self.parse_path("write import path after #use, e.g. #use std.db")?;
        let _ = self.expect(&TokenKind::Newline, "end #use declaration with a newline");
        Some(UseDecl {
            span: start.merge(path.span),
            path,
        })
    }

    fn parse_rec_decl(&mut self) -> Option<RecDecl> {
        let start = self.advance().span;
        let (name, _) =
            self.expect_ident("write record type name after #rec, e.g. #rec Account [...]")?;
        self.expect(&TokenKind::LBracket, "enclose record fields in '[' ... ']'")?;

        let mut fields = Vec::new();
        while !self.at(&TokenKind::RBracket) && !self.at(&TokenKind::Eof) {
            let _ = self.eat(&TokenKind::Comma);
            let (fname, fstart) = self.expect_ident("write field name as 'field:type'")?;
            self.expect(&TokenKind::Colon, "separate field name and type with ':'")?;
            let ty = self.parse_type()?;
            fields.push(Param {
                name: fname,
                span: fstart.merge(ty.span()),
                ty,
            });
        }
        let end = self
            .expect(&TokenKind::RBracket, "close record field list with ']'")?
            .span;
        let _ = self.expect(&TokenKind::Newline, "end #rec declaration with a newline");
        Some(RecDecl {
            name,
            fields,
            span: start.merge(end),
        })
    }

    fn parse_sum_decl(&mut self) -> Option<SumDecl> {
        let start = self.advance().span;
        let (name, _) =
            self.expect_ident("write sum type name after #sum, e.g. #sum TxErr [...]")?;
        self.expect(&TokenKind::LBracket, "enclose sum variants in '[' ... ']'")?;

        let mut variants = Vec::new();
        while !self.at(&TokenKind::RBracket) && !self.at(&TokenKind::Eof) {
            let _ = self.eat(&TokenKind::Comma);
            let (vname, vstart) = self.expect_ident("write variant name inside #sum [...]")?;
            let mut vend = vstart;
            let mut payload = Vec::new();
            if self.eat(&TokenKind::LParen) {
                while !self.at(&TokenKind::RParen) && !self.at(&TokenKind::Eof) {
                    let _ = self.eat(&TokenKind::Comma);
                    payload.push(self.parse_type()?);
                }
                vend = self
                    .expect(&TokenKind::RParen, "close variant payload types with ')'")?
                    .span;
            }
            variants.push(Variant {
                name: vname,
                payload,
                span: vstart.merge(vend),
            });
        }
        let end = self
            .expect(&TokenKind::RBracket, "close #sum variant list with ']'")?
            .span;
        let _ = self.expect(&TokenKind::Newline, "end #sum declaration with a newline");
        Some(SumDecl {
            name,
            variants,
            span: start.merge(end),
        })
    }

    fn parse_signature(&mut self) -> Option<Signature> {
        let start = self
            .expect(
                &TokenKind::LBracket,
                "enclose signature in '[param:type ... -> ret]'",
            )?
            .span;
        let mut params = Vec::new();
        while !self.at(&TokenKind::Arrow)
            && !self.at(&TokenKind::RBracket)
            && !self.at(&TokenKind::Eof)
        {
            let _ = self.eat(&TokenKind::Comma);
            let (pname, pstart) = self.expect_ident("write parameter as 'name:type'")?;
            self.expect(
                &TokenKind::Colon,
                "separate parameter name and type with ':'",
            )?;
            let ty = self.parse_type()?;
            params.push(Param {
                name: pname,
                span: pstart.merge(ty.span()),
                ty,
            });
        }
        let ret = if self.eat(&TokenKind::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };
        let end = self
            .expect(&TokenKind::RBracket, "close signature with ']'")?
            .span;
        Some(Signature {
            params,
            ret,
            span: start.merge(end),
        })
    }

    fn parse_ext_decl(&mut self) -> Option<ExtDecl> {
        let start = self.advance().span;
        let abi = match &self.peek().kind {
            TokenKind::StrLit(_) => {
                if let TokenKind::StrLit(s) = self.advance().kind {
                    s
                } else {
                    unreachable!()
                }
            }
            _ => {
                self.diagnostics.push(Diagnostic::error(
                    "E014",
                    self.peek().span,
                    "expected_abi_string",
                    "specify ABI string after #ext, e.g. #ext \"c\" fn_name [-> u64] !io",
                ));
                return None;
            }
        };
        let (name, _) = self.expect_ident("write external symbol name after ABI string")?;
        let sig = self.parse_signature()?;
        let caps = self.parse_capabilities();
        let end = caps.last().map(|c| c.span).unwrap_or(sig.span);
        let _ = self.expect(&TokenKind::Newline, "end #ext declaration with a newline");
        Some(ExtDecl {
            abi,
            name,
            sig,
            caps,
            span: start.merge(end),
        })
    }

    fn parse_fn_decl(&mut self) -> Option<FnDecl> {
        let start = self.advance().span;
        let (name, _) = self.expect_ident("write function name after #fn")?;
        let sig = self.parse_signature()?;
        let caps = self.parse_capabilities();
        self.expect(&TokenKind::Newline, "put function body on indented lines")?;
        self.expect(
            &TokenKind::Indent,
            "indent #fn body by 2 spaces under #fn header",
        )?;

        let mut contracts = Vec::new();
        while matches!(self.peek().kind, TokenKind::KwReq | TokenKind::KwEns) {
            let kw = self.advance();
            let kind = if kw.kind == TokenKind::KwReq {
                ContractKind::Req
            } else {
                ContractKind::Ens
            };
            let expr = self.parse_expr()?;
            let span = kw.span.merge(expr.span());
            let _ = self.expect(&TokenKind::Newline, "place each contract on its own line");
            contracts.push(Contract { kind, expr, span });
        }

        let body = self.parse_stmt_list_until_dedent()?;
        let end = self.peek().span;
        Some(FnDecl {
            name,
            sig,
            caps,
            contracts,
            body,
            span: start.merge(end),
        })
    }

    fn parse_test_decl(&mut self) -> Option<TestDecl> {
        let start = self.advance().span;
        let name = match &self.peek().kind {
            TokenKind::StrLit(_) => {
                if let TokenKind::StrLit(s) = self.advance().kind {
                    s
                } else {
                    unreachable!()
                }
            }
            _ => {
                self.diagnostics.push(Diagnostic::error(
                    "E014",
                    self.peek().span,
                    "expected_test_name_string",
                    "write test description string after #test, e.g. #test \"adds numbers\"",
                ));
                return None;
            }
        };
        let caps = self.parse_capabilities();
        self.expect(&TokenKind::Newline, "put #test body on indented lines")?;
        self.expect(&TokenKind::Indent, "indent #test body by 2 spaces")?;
        let body = self.parse_stmt_list_until_dedent()?;
        let end = self.peek().span;
        Some(TestDecl {
            name,
            caps,
            body,
            span: start.merge(end),
        })
    }

    fn parse_type(&mut self) -> Option<TypeExpr> {
        let base = match &self.peek().kind {
            TokenKind::Question => {
                let q = self.advance().span;
                let inner = self.parse_base_type()?;
                let span = q.merge(inner.span());
                TypeExpr::Optional {
                    inner: Box::new(inner),
                    span,
                }
            }
            _ => self.parse_base_type()?,
        };

        if self.eat(&TokenKind::Bang) {
            let err = self.parse_base_type()?;
            let span = base.span().merge(err.span());
            Ok::<TypeExpr, ()>(TypeExpr::Result {
                ok: Box::new(base),
                err: Box::new(err),
                span,
            })
            .ok()
        } else {
            Some(base)
        }
    }

    fn parse_base_type(&mut self) -> Option<TypeExpr> {
        match &self.peek().kind {
            TokenKind::LBracket => {
                let start = self.advance().span;
                if self.at(&TokenKind::RBracket) {
                    let _ = self.advance();
                    let elem = self.parse_base_type()?;
                    let span = start.merge(elem.span());
                    Some(TypeExpr::Slice {
                        elem: Box::new(elem),
                        span,
                    })
                } else {
                    let key = self.parse_type()?;
                    self.expect(&TokenKind::Colon, "write map type as '[Key:Val]'")?;
                    let val = self.parse_type()?;
                    let end = self
                        .expect(&TokenKind::RBracket, "close map type with ']'")?
                        .span;
                    Some(TypeExpr::Map {
                        key: Box::new(key),
                        val: Box::new(val),
                        span: start.merge(end),
                    })
                }
            }
            TokenKind::Star => {
                let start = self.advance().span;
                let pointee = self.parse_base_type()?;
                let span = start.merge(pointee.span());
                Some(TypeExpr::Pointer {
                    pointee: Box::new(pointee),
                    span,
                })
            }
            _ => {
                let path = self.parse_path("expected type name (e.g. i64, str, []T, ?T)")?;
                let span = path.span;
                Some(TypeExpr::Named { path, span })
            }
        }
    }

    fn parse_indented_block(&mut self) -> Option<Vec<Stmt>> {
        self.expect(
            &TokenKind::Newline,
            "expected newline before indented block",
        )?;
        self.expect(&TokenKind::Indent, "indent nested block by 2 spaces")?;
        self.parse_stmt_list_until_dedent()
    }

    fn parse_stmt_list_until_dedent(&mut self) -> Option<Vec<Stmt>> {
        let mut stmts = Vec::new();
        self.skip_newlines();
        while !self.at(&TokenKind::Dedent) && !self.at(&TokenKind::Eof) {
            if let Some(stmt) = self.parse_stmt() {
                stmts.push(stmt);
            } else {
                self.recover_to_newline();
            }
            self.skip_newlines();
        }
        let _ = self.eat(&TokenKind::Dedent);
        Some(stmts)
    }

    fn parse_stmt(&mut self) -> Option<Stmt> {
        match &self.peek().kind {
            TokenKind::Eq => {
                let start = self.advance().span;
                if self.at(&TokenKind::Newline) || self.at(&TokenKind::Dedent) {
                    let _ = self.eat(&TokenKind::Newline);
                    Some(Stmt::Return {
                        value: None,
                        span: start,
                    })
                } else {
                    let val = self.parse_expr()?;
                    let span = start.merge(val.span());
                    let _ = self.eat(&TokenKind::Newline);
                    Some(Stmt::Return {
                        value: Some(val),
                        span,
                    })
                }
            }
            TokenKind::Caret => {
                let start = self.advance().span;
                let err = self.parse_expr()?;
                let span = start.merge(err.span());
                let _ = self.eat(&TokenKind::Newline);
                Some(Stmt::Fail { error: err, span })
            }
            TokenKind::Question => {
                let start = self.advance().span;
                let cond = self.parse_expr()?;
                let mut end = cond.span();
                let otherwise = if self.eat(&TokenKind::Pipe) {
                    let act = self.parse_guard_action()?;
                    end = self.peek().span;
                    Some(act)
                } else {
                    None
                };
                let _ = self.eat(&TokenKind::Newline);
                Some(Stmt::Guard {
                    cond,
                    otherwise,
                    span: start.merge(end),
                })
            }
            TokenKind::AtNext => {
                let span = self.advance().span;
                let _ = self.eat(&TokenKind::Newline);
                Some(Stmt::Next { span })
            }
            TokenKind::AtBrk => {
                let span = self.advance().span;
                let _ = self.eat(&TokenKind::Newline);
                Some(Stmt::Break { span })
            }
            TokenKind::AtIf => self.parse_if_stmt(),
            TokenKind::AtFor => self.parse_for_stmt(),
            TokenKind::AtLoop => {
                let start = self.advance().span;
                let body = self.parse_indented_block()?;
                Some(Stmt::Loop { body, span: start })
            }
            TokenKind::AtMatch => self.parse_match_stmt(),
            TokenKind::Ident(word) if matches!(word.as_str(), "let" | "var" | "const") => {
                let bad = self.advance();
                if let TokenKind::Ident(kw) = &bad.kind {
                    self.diagnostics.push(Diagnostic::error(
                        "E015",
                        bad.span,
                        format!("forbidden_keyword '{kw}'"),
                        "remove 'let'/'var'/'const'; write 'name = expr' directly",
                    ));
                }
                self.parse_stmt()
            }
            TokenKind::Ident(word) if word == "return" => {
                let bad = self.advance();
                self.diagnostics.push(Diagnostic::error(
                    "E016",
                    bad.span,
                    "forbidden_keyword 'return'",
                    "replace 'return expr' with '= expr' (or '^ expr' for error return)",
                ));
                let val = if !self.at(&TokenKind::Newline) {
                    Some(self.parse_expr()?)
                } else {
                    None
                };
                let _ = self.eat(&TokenKind::Newline);
                Some(Stmt::Return {
                    value: val,
                    span: bad.span,
                })
            }
            TokenKind::Ident(_) if self.peek_at(1).kind == TokenKind::Eq => {
                let (target, start) = self.expect_ident("expected binding name")?;
                let _ = self.advance(); // '='
                let value = self.parse_expr()?;
                let span = start.merge(value.span());
                let _ = self.eat(&TokenKind::Newline);
                Some(Stmt::Bind {
                    target,
                    value,
                    span,
                })
            }
            TokenKind::KwReq | TokenKind::KwEns => {
                let bad = self.advance();
                self.diagnostics.push(Diagnostic::error(
                    "E017",
                    bad.span,
                    "misplaced_contract",
                    "place 'req' and 'ens' contracts immediately at the top of #fn before statements",
                ));
                let _ = self.parse_expr();
                let _ = self.eat(&TokenKind::Newline);
                None
            }
            _ => {
                let expr = self.parse_expr()?;
                let span = expr.span();
                let _ = self.eat(&TokenKind::Newline);
                Some(Stmt::Expr { expr, span })
            }
        }
    }

    fn parse_guard_action(&mut self) -> Option<GuardAction> {
        match &self.peek().kind {
            TokenKind::Caret => {
                self.advance();
                Some(GuardAction::Fail(self.parse_expr()?))
            }
            TokenKind::Eq => {
                self.advance();
                if self.at(&TokenKind::Newline) {
                    Some(GuardAction::Return(None))
                } else {
                    Some(GuardAction::Return(Some(self.parse_expr()?)))
                }
            }
            TokenKind::AtNext => {
                self.advance();
                Some(GuardAction::Next)
            }
            TokenKind::AtBrk => {
                self.advance();
                Some(GuardAction::Break)
            }
            _ => {
                let bad = self.peek();
                self.diagnostics.push(Diagnostic::error(
                    "E018",
                    bad.span,
                    "invalid_guard_action",
                    "after '|', write '^Err', '= val', '@next', or '@brk'",
                ));
                None
            }
        }
    }

    fn parse_if_stmt(&mut self) -> Option<Stmt> {
        let start = self.advance().span;
        let cond = self.parse_expr()?;
        let then_body = self.parse_indented_block()?;
        let mut branches = vec![(cond, then_body)];
        self.skip_newlines();

        while self.eat(&TokenKind::AtElif) {
            let elif_cond = self.parse_expr()?;
            let elif_body = self.parse_indented_block()?;
            branches.push((elif_cond, elif_body));
            self.skip_newlines();
        }

        let else_body = if self.eat(&TokenKind::AtEl) {
            Some(self.parse_indented_block()?)
        } else {
            None
        };

        Some(Stmt::If {
            branches,
            else_body,
            span: start,
        })
    }

    fn parse_for_stmt(&mut self) -> Option<Stmt> {
        let start = self.advance().span;
        let (binding, _) = self.expect_ident("write loop variable after @for, e.g. @for x : xs")?;
        self.expect(
            &TokenKind::Colon,
            "use ':' between loop variable and iterable",
        )?;
        let iter = self.parse_expr()?;
        let body = self.parse_indented_block()?;
        Some(Stmt::For {
            binding,
            iter,
            body,
            span: start,
        })
    }

    fn parse_match_stmt(&mut self) -> Option<Stmt> {
        let start = self.advance().span;
        let scrutinee = self.parse_expr()?;
        self.expect(&TokenKind::Newline, "put @match arms on indented lines")?;
        self.expect(&TokenKind::Indent, "indent @match arms by 2 spaces")?;

        let mut arms = Vec::new();
        self.skip_newlines();
        while !self.at(&TokenKind::Dedent) && !self.at(&TokenKind::Eof) {
            let pat = self.parse_pattern()?;
            self.expect(&TokenKind::FatArrow, "use '=>' after @match pattern")?;
            let body = if self.at(&TokenKind::Newline) {
                self.parse_indented_block()?
            } else {
                vec![self.parse_stmt()?]
            };
            arms.push(MatchArm {
                span: pat.span(),
                pattern: pat,
                body,
            });
            self.skip_newlines();
        }
        let _ = self.eat(&TokenKind::Dedent);

        Some(Stmt::Match {
            scrutinee,
            arms,
            span: start,
        })
    }

    fn parse_pattern(&mut self) -> Option<Pattern> {
        match &self.peek().kind {
            TokenKind::Ident(s) if s == "_" => {
                let tok = self.advance();
                Some(Pattern::Wildcard(tok.span))
            }
            TokenKind::IntLit(_)
            | TokenKind::FloatLit(_)
            | TokenKind::StrLit(_)
            | TokenKind::KwTrue
            | TokenKind::KwFalse
            | TokenKind::KwNone => {
                let tok = self.advance();
                let lit = match tok.kind {
                    TokenKind::IntLit(v) => Literal::Int(v),
                    TokenKind::FloatLit(v) => Literal::Float(v),
                    TokenKind::StrLit(s) => Literal::Str(s),
                    TokenKind::KwTrue => Literal::Bool(true),
                    TokenKind::KwFalse => Literal::Bool(false),
                    TokenKind::KwNone => Literal::None,
                    _ => unreachable!(),
                };
                Some(Pattern::Literal(lit, tok.span))
            }
            TokenKind::Ident(_) => {
                let path = self.parse_path("expected pattern identifier or variant")?;
                let mut span = path.span;
                let mut args = Vec::new();
                if self.eat(&TokenKind::LParen) {
                    while !self.at(&TokenKind::RParen) && !self.at(&TokenKind::Eof) {
                        let _ = self.eat(&TokenKind::Comma);
                        args.push(self.parse_pattern()?);
                    }
                    let end = self
                        .expect(&TokenKind::RParen, "close variant pattern with ')'")?
                        .span;
                    span = span.merge(end);
                }
                Some(Pattern::Variant { path, args, span })
            }
            _ => {
                let bad = self.peek();
                self.diagnostics.push(Diagnostic::error(
                    "E019",
                    bad.span,
                    "invalid_pattern",
                    "write '_', a literal, a binding name, or Variant(inner)",
                ));
                None
            }
        }
    }

    // --- Expressions (Pratt Precedence + Space-Separated Command Calls + `?` Suffix) ---

    fn parse_expr(&mut self) -> Option<Expr> {
        let mut lhs = self.parse_binary_expr(0)?;

        // Space-separated command call: `db.get_acc uid` or `str.fmt "ok:{}" v`
        if matches!(lhs, Expr::Ident { .. } | Expr::Field { .. }) && self.is_command_arg_start() {
            let start = lhs.span();
            let mut args = Vec::new();
            while self.is_command_arg_start() {
                args.push(self.parse_postfix()?);
            }
            let end = args.last().map(Expr::span).unwrap_or(start);
            lhs = Expr::Call {
                callee: Box::new(lhs),
                args,
                span: start.merge(end),
            };
        }

        // Error propagation suffix: `expr ?` or `expr ? ^ErrExpr`
        if self.at(&TokenKind::Question) {
            let q = self.advance();
            let mut end = q.span;
            let map_err = if self.eat(&TokenKind::Caret) {
                let err_expr = self.parse_expr()?;
                end = err_expr.span();
                Some(Box::new(err_expr))
            } else {
                None
            };
            let span = lhs.span().merge(end);
            lhs = Expr::Try {
                expr: Box::new(lhs),
                map_err,
                span,
            };
        }

        Some(lhs)
    }

    fn is_command_arg_start(&self) -> bool {
        matches!(
            self.peek().kind,
            TokenKind::Ident(_)
                | TokenKind::IntLit(_)
                | TokenKind::FloatLit(_)
                | TokenKind::StrLit(_)
                | TokenKind::KwTrue
                | TokenKind::KwFalse
                | TokenKind::KwNone
                | TokenKind::LParen
                | TokenKind::LBracket
        )
    }

    fn parse_binary_expr(&mut self, min_bp: u8) -> Option<Expr> {
        let mut lhs = self.parse_unary()?;

        while let Some((op, l_bp, r_bp)) = self.peek_bin_op() {
            if l_bp < min_bp {
                break;
            }
            self.advance();
            let rhs = self.parse_binary_expr(r_bp)?;
            let span = lhs.span().merge(rhs.span());
            lhs = Expr::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
                span,
            };
        }

        Some(lhs)
    }

    fn peek_bin_op(&self) -> Option<(BinOp, u8, u8)> {
        let op = match self.peek().kind {
            TokenKind::QuestionQuestion => (BinOp::Coalesce, 1, 2),
            TokenKind::PipePipe => (BinOp::Or, 3, 4),
            TokenKind::AmpAmp => (BinOp::And, 5, 6),
            TokenKind::EqEq => (BinOp::Eq, 7, 8),
            TokenKind::BangEq => (BinOp::Ne, 7, 8),
            TokenKind::Lt => (BinOp::Lt, 9, 10),
            TokenKind::Le => (BinOp::Le, 9, 10),
            TokenKind::Gt => (BinOp::Gt, 9, 10),
            TokenKind::Ge => (BinOp::Ge, 9, 10),
            TokenKind::Plus => (BinOp::Add, 11, 12),
            TokenKind::Minus => (BinOp::Sub, 11, 12),
            TokenKind::Star => (BinOp::Mul, 13, 14),
            TokenKind::Slash => (BinOp::Div, 13, 14),
            TokenKind::Percent => (BinOp::Rem, 13, 14),
            _ => return None,
        };
        Some(op)
    }

    fn parse_unary(&mut self) -> Option<Expr> {
        match self.peek().kind {
            TokenKind::Minus => {
                let op_tok = self.advance();
                let operand = self.parse_unary()?;
                let span = op_tok.span.merge(operand.span());
                Some(Expr::Unary {
                    op: UnaryOp::Neg,
                    expr: Box::new(operand),
                    span,
                })
            }
            TokenKind::Bang => {
                let op_tok = self.advance();
                let operand = self.parse_unary()?;
                let span = op_tok.span.merge(operand.span());
                Some(Expr::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(operand),
                    span,
                })
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Option<Expr> {
        let mut expr = self.parse_primary()?;

        loop {
            match &self.peek().kind {
                TokenKind::Dot => {
                    self.advance();
                    let (field, fspan) =
                        self.expect_ident("write field or method name after '.'")?;
                    let span = expr.span().merge(fspan);
                    expr = Expr::Field {
                        base: Box::new(expr),
                        field,
                        span,
                    };
                }
                // Adjacent `callee(a, b)` with no intervening space
                TokenKind::LParen if expr.span().end == self.peek().span.start => {
                    self.advance();
                    let mut args = Vec::new();
                    while !self.at(&TokenKind::RParen) && !self.at(&TokenKind::Eof) {
                        let _ = self.eat(&TokenKind::Comma);
                        args.push(self.parse_expr()?);
                    }
                    let end = self
                        .expect(&TokenKind::RParen, "close call arguments with ')'")?
                        .span;
                    let span = expr.span().merge(end);
                    expr = Expr::Call {
                        callee: Box::new(expr),
                        args,
                        span,
                    };
                }
                // Adjacent `slice[idx]` with no intervening space
                TokenKind::LBracket if expr.span().end == self.peek().span.start => {
                    self.advance();
                    let index = self.parse_expr()?;
                    let end = self
                        .expect(&TokenKind::RBracket, "close index expression with ']'")?
                        .span;
                    let span = expr.span().merge(end);
                    expr = Expr::Index {
                        base: Box::new(expr),
                        index: Box::new(index),
                        span,
                    };
                }
                _ => break,
            }
        }

        Some(expr)
    }

    fn parse_primary(&mut self) -> Option<Expr> {
        let tok = self.peek().clone();
        match tok.kind {
            TokenKind::IntLit(v) => {
                self.advance();
                Some(Expr::Literal {
                    lit: Literal::Int(v),
                    span: tok.span,
                })
            }
            TokenKind::FloatLit(v) => {
                self.advance();
                Some(Expr::Literal {
                    lit: Literal::Float(v),
                    span: tok.span,
                })
            }
            TokenKind::StrLit(s) => {
                self.advance();
                Some(Expr::Literal {
                    lit: Literal::Str(s),
                    span: tok.span,
                })
            }
            TokenKind::KwTrue => {
                self.advance();
                Some(Expr::Literal {
                    lit: Literal::Bool(true),
                    span: tok.span,
                })
            }
            TokenKind::KwFalse => {
                self.advance();
                Some(Expr::Literal {
                    lit: Literal::Bool(false),
                    span: tok.span,
                })
            }
            TokenKind::KwNone => {
                self.advance();
                Some(Expr::Literal {
                    lit: Literal::None,
                    span: tok.span,
                })
            }
            TokenKind::Ident(name) => {
                self.advance();
                Some(Expr::Ident {
                    name,
                    span: tok.span,
                })
            }
            TokenKind::LParen => {
                let start = self.advance().span;
                let mut inner = self.parse_expr()?;
                let end = self
                    .expect(&TokenKind::RParen, "close grouped expression with ')'")?
                    .span;
                // Preserve full `( ... )` span so `foo (a + 1) b` keeps accurate boundaries.
                if let Expr::Binary { span, .. }
                | Expr::Unary { span, .. }
                | Expr::Call { span, .. } = &mut inner
                {
                    *span = start.merge(end);
                }
                Some(inner)
            }
            TokenKind::LBracket => {
                let start = self.advance().span;
                let mut items = Vec::new();
                while !self.at(&TokenKind::RBracket) && !self.at(&TokenKind::Eof) {
                    let _ = self.eat(&TokenKind::Comma);
                    items.push(self.parse_expr()?);
                }
                let end = self
                    .expect(&TokenKind::RBracket, "close list literal with ']'")?
                    .span;
                Some(Expr::List {
                    items,
                    span: start.merge(end),
                })
            }
            _ => {
                self.diagnostics.push(Diagnostic::error(
                    "E020",
                    tok.span,
                    format!("unexpected_token '{}'", tok.kind.describe()),
                    "write a valid expression (literal, variable, call, or '(...)')",
                ));
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_canonical_ledger_showcase_without_errors() {
        let src = include_str!("../../../examples/ledger.myl");
        let out = parse_source(src);
        assert!(
            out.diagnostics.is_empty(),
            "unexpected diagnostics: {:?}",
            out.diagnostics
        );
        assert_eq!(out.module.items.len(), 9);
    }

    #[test]
    fn recovers_and_advises_when_llm_leaks_let_or_return() {
        let src = "#fn add [a:i64 b:i64 -> i64]\n  let c = a + b\n  return c\n";
        let out = parse_source(src);
        let codes: Vec<_> = out.diagnostics.iter().map(|d| d.code).collect();
        assert_eq!(codes, vec!["E015", "E016"]);
    }
}
