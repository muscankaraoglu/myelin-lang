//! `myelin-mir` — Flat Single Static Assignment (SSA) Intermediate Representation
//! and Linear Resource Verification for Myelin.

use std::collections::{HashMap, HashSet};
use std::fmt;

use myelin_ast::{BinOp, Expr, FnDecl, Item, Literal, Module, Stmt, UnaryOp};
use myelin_diag::{Diagnostic, Span};

/// Virtual register in Single Static Assignment (SSA) form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VReg(pub usize);

impl fmt::Display for VReg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "%{}", self.0)
    }
}

/// Basic block identifier in the control-flow graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BlockId(pub usize);

impl fmt::Display for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bb{}", self.0)
    }
}

/// An atomic SSA value or constant operand.
#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Copy(VReg),
    Constant(Literal),
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operand::Copy(reg) => write!(f, "{reg}"),
            Operand::Constant(lit) => match lit {
                Literal::Int(i) => write!(f, "{i}"),
                Literal::Float(fl) => write!(f, "{fl}"),
                Literal::Bool(b) => write!(f, "{b}"),
                Literal::Str(s) => write!(f, "\"{s}\""),
                Literal::None => write!(f, "none"),
            },
        }
    }
}

/// A computational operation producing a single value.
#[derive(Debug, Clone, PartialEq)]
pub enum Rvalue {
    Use(Operand),
    Unary(UnaryOp, Operand),
    Binary(BinOp, Operand, Operand),
    Call {
        callee: String,
        args: Vec<Operand>,
    },
    Field {
        base: Operand,
        field: String,
    },
    Index {
        base: Operand,
        index: Operand,
    },
    List(Vec<Operand>),
}

impl fmt::Display for Rvalue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Rvalue::Use(op) => write!(f, "{op}"),
            Rvalue::Unary(op, operand) => {
                let op_str = match op {
                    UnaryOp::Neg => "-",
                    UnaryOp::Not => "!",
                };
                write!(f, "{op_str}{operand}")
            }
            Rvalue::Binary(op, lhs, rhs) => {
                let op_str = match op {
                    BinOp::Add => "+",
                    BinOp::Sub => "-",
                    BinOp::Mul => "*",
                    BinOp::Div => "/",
                    BinOp::Rem => "%",
                    BinOp::Eq => "==",
                    BinOp::Ne => "!=",
                    BinOp::Lt => "<",
                    BinOp::Le => "<=",
                    BinOp::Gt => ">",
                    BinOp::Ge => ">=",
                    BinOp::And => "&&",
                    BinOp::Or => "||",
                    BinOp::Coalesce => "??",
                };
                write!(f, "{lhs} {op_str} {rhs}")
            }
            Rvalue::Call { callee, args } => {
                let arg_strs: Vec<String> = args.iter().map(|a| a.to_string()).collect();
                write!(f, "call {callee}({})", arg_strs.join(", "))
            }
            Rvalue::Field { base, field } => write!(f, "{base}.{field}"),
            Rvalue::Index { base, index } => write!(f, "{base}[{index}]"),
            Rvalue::List(items) => {
                let item_strs: Vec<String> = items.iter().map(|i| i.to_string()).collect();
                write!(f, "[{}]", item_strs.join(", "))
            }
        }
    }
}

/// Non-branching atomic instruction inside a basic block.
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Assign(VReg, Rvalue, Span),
    Drop(VReg),
    Comment(String),
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Statement::Assign(reg, rvalue, _) => write!(f, "    {reg} = {rvalue}"),
            Statement::Drop(reg) => write!(f, "    drop({reg})"),
            Statement::Comment(c) => write!(f, "    -- {c}"),
        }
    }
}

/// Block terminator defining outgoing control-flow edges.
#[derive(Debug, Clone, PartialEq)]
pub enum Terminator {
    Return(Option<Operand>),
    Goto {
        target: BlockId,
        args: Vec<Operand>,
    },
    Branch {
        cond: Operand,
        then_block: BlockId,
        then_args: Vec<Operand>,
        else_block: BlockId,
        else_args: Vec<Operand>,
    },
    Unreachable,
}

impl fmt::Display for Terminator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Terminator::Return(Some(op)) => write!(f, "    return {op}"),
            Terminator::Return(None) => write!(f, "    return"),
            Terminator::Goto { target, args } => {
                if args.is_empty() {
                    write!(f, "    goto {target}")
                } else {
                    let a_strs: Vec<String> = args.iter().map(|a| a.to_string()).collect();
                    write!(f, "    goto {target}({})", a_strs.join(", "))
                }
            }
            Terminator::Branch {
                cond,
                then_block,
                else_block,
                ..
            } => {
                write!(f, "    branch {cond} ? {then_block} : {else_block}")
            }
            Terminator::Unreachable => write!(f, "    unreachable"),
        }
    }
}

/// Basic block consisting of argument variables, statements, and one terminator.
#[derive(Debug, Clone, PartialEq)]
pub struct BasicBlock {
    pub id: BlockId,
    pub args: Vec<VReg>,
    pub statements: Vec<Statement>,
    pub terminator: Terminator,
}

impl BasicBlock {
    pub fn new(id: BlockId, args: Vec<VReg>) -> Self {
        Self {
            id,
            args,
            statements: Vec::new(),
            terminator: Terminator::Unreachable,
        }
    }
}

/// Lowered Flat SSA function.
#[derive(Debug, Clone, PartialEq)]
pub struct MirFunction {
    pub name: String,
    pub params: Vec<(String, VReg)>,
    pub declared_caps: Vec<String>,
    pub blocks: Vec<BasicBlock>,
    pub vreg_count: usize,
}

impl MirFunction {
    pub fn print_mir(&self) -> String {
        let mut out = String::new();
        let param_strs: Vec<String> = self
            .params
            .iter()
            .map(|(name, reg)| format!("{name}: {reg}"))
            .collect();
        let cap_str = if self.declared_caps.is_empty() {
            String::new()
        } else {
            format!(" {}", self.declared_caps.join(" "))
        };
        out.push_str(&format!(
            "#mir fn {} [{}]{}\n",
            self.name,
            param_strs.join(", "),
            cap_str
        ));

        for block in &self.blocks {
            if block.args.is_empty() {
                out.push_str(&format!("{}:\n", block.id));
            } else {
                let a_strs: Vec<String> = block.args.iter().map(|a| a.to_string()).collect();
                out.push_str(&format!("{}({}):\n", block.id, a_strs.join(", ")));
            }
            for stmt in &block.statements {
                out.push_str(&format!("{stmt}\n"));
            }
            out.push_str(&format!("{}\n", block.terminator));
        }
        out
    }
}

/// Complete module in Flat SSA form.
#[derive(Debug, Clone, PartialEq)]
pub struct MirModule {
    pub functions: Vec<MirFunction>,
}

impl MirModule {
    pub fn print_mir(&self) -> String {
        let mut out = String::new();
        for f in &self.functions {
            out.push_str(&f.print_mir());
            out.push('\n');
        }
        out
    }
}

/// Builder that lowers AST declarations into Flat SSA representation.
pub struct MirBuilder {
    next_vreg: usize,
    next_block: usize,
    blocks: Vec<BasicBlock>,
    current_block: BlockId,
    var_map: HashMap<String, VReg>,
    linear_resources: HashSet<VReg>,
    consumed_resources: HashSet<VReg>,
    diagnostics: Vec<Diagnostic>,
}

impl MirBuilder {
    pub fn new() -> Self {
        let entry_id = BlockId(0);
        let entry_block = BasicBlock::new(entry_id, Vec::new());
        Self {
            next_vreg: 0,
            next_block: 1,
            blocks: vec![entry_block],
            current_block: entry_id,
            var_map: HashMap::new(),
            linear_resources: HashSet::new(),
            consumed_resources: HashSet::new(),
            diagnostics: Vec::new(),
        }
    }

    pub fn alloc_vreg(&mut self) -> VReg {
        let reg = VReg(self.next_vreg);
        self.next_vreg += 1;
        reg
    }

    pub fn alloc_block(&mut self, args: Vec<VReg>) -> BlockId {
        let id = BlockId(self.next_block);
        self.next_block += 1;
        self.blocks.push(BasicBlock::new(id, args));
        id
    }

    pub fn current_bb(&mut self) -> &mut BasicBlock {
        let idx = self.current_block.0;
        &mut self.blocks[idx]
    }

    pub fn push_stmt(&mut self, stmt: Statement) {
        self.current_bb().statements.push(stmt);
    }

    pub fn set_terminator(&mut self, term: Terminator) {
        self.current_bb().terminator = term;
    }

    fn check_consumption(&mut self, expr: &Expr, span: Span) {
        if let Expr::Call { callee, args, .. } = expr {
            if let Expr::Field { base, field, .. } = &**callee {
                if let Expr::Ident { name, .. } = &**base {
                    if (name == "fs" && field == "close")
                        || (name == "net" && field == "close")
                        || (name == "gui" && field == "close")
                    {
                        if let Some(first) = args.first() {
                            if let Expr::Ident { name: var_name, .. } = first {
                                if let Some(&reg) = self.var_map.get(var_name) {
                                    if self.consumed_resources.contains(&reg) {
                                        self.diagnostics.push(Diagnostic::error(
                                            "L002",
                                            span,
                                            format!("linear resource '{var_name}' ({reg}) consumed more than once"),
                                            "remove duplicate close/consumption call",
                                        ));
                                    } else {
                                        self.consumed_resources.insert(reg);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn lower_fn(mut self, f: &FnDecl) -> (MirFunction, Vec<Diagnostic>) {
        let mut mir_params = Vec::new();
        for param in &f.sig.params {
            let reg = self.alloc_vreg();
            self.var_map.insert(param.name.clone(), reg);
            mir_params.push((param.name.clone(), reg));
        }

        for stmt in &f.body {
            self.lower_stmt(stmt);
        }

        // If current block doesn't have a terminator, default to return unit
        if matches!(self.current_bb().terminator, Terminator::Unreachable) {
            self.set_terminator(Terminator::Return(None));
        }

        // --- Linear Resource Verification (Austral Model) ---
        // Verify all opened linear resources were consumed
        for resource in &self.linear_resources {
            if !self.consumed_resources.contains(resource) {
                self.diagnostics.push(Diagnostic::error(
                    "L001",
                    f.sig.span,
                    format!("linear resource '{resource}' created but never consumed before return"),
                    "explicitly close, free, or pass the resource before function return",
                ));
            }
        }

        let declared_caps = f.caps.iter().map(|c| c.path.as_dotted()).collect();
        let mir_fn = MirFunction {
            name: f.name.clone(),
            params: mir_params,
            declared_caps,
            blocks: self.blocks,
            vreg_count: self.next_vreg,
        };

        (mir_fn, self.diagnostics)
    }

    fn lower_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Bind { target, value, span } => {
                self.check_consumption(value, *span);
                let rval = self.lower_expr_to_rvalue(value);
                let reg = self.alloc_vreg();
                self.push_stmt(Statement::Assign(reg, rval, *span));
                // Update SSA mapping: variable now points to this new SSA register!
                self.var_map.insert(target.clone(), reg);

                // Austral linear resource creation detection
                if let Expr::Call { callee, .. } = value {
                    if let Expr::Field { base, field, .. } = &**callee {
                        if let Expr::Ident { name, .. } = &**base {
                            if (name == "fs" && field == "open")
                                || (name == "net" && field == "connect")
                                || (name == "gui" && field == "window")
                            {
                                self.linear_resources.insert(reg);
                            }
                        }
                    }
                }
            }
            Stmt::Expr { expr, span } => {
                self.check_consumption(expr, *span);
                let rval = self.lower_expr_to_rvalue(expr);
                let reg = self.alloc_vreg();
                self.push_stmt(Statement::Assign(reg, rval, *span));
            }
            Stmt::Return { value, .. } => {
                let op = value.as_ref().map(|v| self.lower_expr_to_operand(v));
                self.set_terminator(Terminator::Return(op));
            }
            Stmt::If {
                branches,
                else_body,
                ..
            } => {
                let exit_block = self.alloc_block(Vec::new());

                for (cond_expr, body) in branches {
                    let cond_op = self.lower_expr_to_operand(cond_expr);
                    let then_block = self.alloc_block(Vec::new());
                    let next_test_block = self.alloc_block(Vec::new());

                    self.set_terminator(Terminator::Branch {
                        cond: cond_op,
                        then_block,
                        then_args: Vec::new(),
                        else_block: next_test_block,
                        else_args: Vec::new(),
                    });

                    // Build then block
                    self.current_block = then_block;
                    for s in body {
                        self.lower_stmt(s);
                    }
                    if matches!(self.current_bb().terminator, Terminator::Unreachable) {
                        self.set_terminator(Terminator::Goto {
                            target: exit_block,
                            args: Vec::new(),
                        });
                    }

                    self.current_block = next_test_block;
                }

                if let Some(el_stmts) = else_body {
                    for s in el_stmts {
                        self.lower_stmt(s);
                    }
                }

                if matches!(self.current_bb().terminator, Terminator::Unreachable) {
                    self.set_terminator(Terminator::Goto {
                        target: exit_block,
                        args: Vec::new(),
                    });
                }

                self.current_block = exit_block;
            }
            Stmt::Loop { body, .. } => {
                let loop_header = self.alloc_block(Vec::new());
                let loop_exit = self.alloc_block(Vec::new());

                self.set_terminator(Terminator::Goto {
                    target: loop_header,
                    args: Vec::new(),
                });

                self.current_block = loop_header;
                for s in body {
                    self.lower_stmt(s);
                }

                if matches!(self.current_bb().terminator, Terminator::Unreachable) {
                    self.set_terminator(Terminator::Goto {
                        target: loop_header,
                        args: Vec::new(),
                    });
                }

                self.current_block = loop_exit;
            }
            _ => {}
        }
    }

    fn lower_expr_to_operand(&mut self, expr: &Expr) -> Operand {
        match expr {
            Expr::Literal { lit, .. } => Operand::Constant(lit.clone()),
            Expr::Ident { name, span } => {
                if let Some(&reg) = self.var_map.get(name) {
                    Operand::Copy(reg)
                } else {
                    let reg = self.alloc_vreg();
                    self.push_stmt(Statement::Comment(format!("undefined identifier: {name}")));
                    self.push_stmt(Statement::Assign(
                        reg,
                        Rvalue::Use(Operand::Constant(Literal::None)),
                        *span,
                    ));
                    Operand::Copy(reg)
                }
            }
            complex => {
                let rval = self.lower_expr_to_rvalue(complex);
                let reg = self.alloc_vreg();
                self.push_stmt(Statement::Assign(reg, rval, complex.span()));
                Operand::Copy(reg)
            }
        }
    }

    fn lower_expr_to_rvalue(&mut self, expr: &Expr) -> Rvalue {
        match expr {
            Expr::Literal { lit, .. } => Rvalue::Use(Operand::Constant(lit.clone())),
            Expr::Ident { name, .. } => {
                if let Some(&reg) = self.var_map.get(name) {
                    Rvalue::Use(Operand::Copy(reg))
                } else {
                    Rvalue::Use(Operand::Constant(Literal::None))
                }
            }
            Expr::Unary { op, expr, .. } => {
                let operand = self.lower_expr_to_operand(expr);
                Rvalue::Unary(*op, operand)
            }
            Expr::Binary { op, lhs, rhs, .. } => {
                let l = self.lower_expr_to_operand(lhs);
                let r = self.lower_expr_to_operand(rhs);
                Rvalue::Binary(*op, l, r)
            }
            Expr::Call { callee, args, .. } => {
                let callee_name = match &**callee {
                    Expr::Field { base, field, .. } => {
                        if let Expr::Ident { name, .. } = &**base {
                            format!("{name}.{field}")
                        } else {
                            "indirect_field_call".to_string()
                        }
                    }
                    Expr::Ident { name, .. } => name.clone(),
                    _ => "indirect_call".to_string(),
                };
                let lowered_args: Vec<Operand> =
                    args.iter().map(|a| self.lower_expr_to_operand(a)).collect();
                Rvalue::Call {
                    callee: callee_name,
                    args: lowered_args,
                }
            }
            Expr::Field { base, field, .. } => {
                let b = self.lower_expr_to_operand(base);
                Rvalue::Field {
                    base: b,
                    field: field.clone(),
                }
            }
            Expr::Index { base, index, .. } => {
                let b = self.lower_expr_to_operand(base);
                let idx = self.lower_expr_to_operand(index);
                Rvalue::Index { base: b, index: idx }
            }
            Expr::List { items, .. } => {
                let list_items: Vec<Operand> =
                    items.iter().map(|i| self.lower_expr_to_operand(i)).collect();
                Rvalue::List(list_items)
            }
            Expr::Try { expr, .. } => self.lower_expr_to_rvalue(expr),
        }
    }
}

/// Lowers an entire AST module into Flat SSA MIR.
pub fn lower_module(module: &Module) -> (MirModule, Vec<Diagnostic>) {
    let mut functions = Vec::new();
    let mut all_diags = Vec::new();

    for item in &module.items {
        if let Item::Fn(f) = item {
            let builder = MirBuilder::new();
            let (mir_fn, diags) = builder.lower_fn(f);
            functions.push(mir_fn);
            all_diags.extend(diags);
        }
    }

    (MirModule { functions }, all_diags)
}

#[cfg(test)]
mod tests {
    use super::*;
    use myelin_parser::parse_source;

    #[test]
    fn test_ssa_rebinding_lowers_to_unique_vregs() {
        let src = r#"#mod main

#fn compute [-> i64]
  x = 10
  x = x + 5
  x = x * 2
  = x
"#;
        let parsed = parse_source(src);
        assert!(parsed.diagnostics.is_empty(), "parse diagnostics: {:?}", parsed.diagnostics);
        let (mir_mod, diags) = lower_module(&parsed.module);
        assert!(diags.is_empty());

        let mir_fn = &mir_mod.functions[0];
        let text = mir_fn.print_mir();
        // Verify that x was rebound to unique SSA registers: %0, %1, %2
        assert!(text.contains("%0 = 10"));
        assert!(text.contains("%1 = %0 + 5"));
        assert!(text.contains("%2 = %1 * 2"));
        assert!(text.contains("return %2"));
    }

    #[test]
    fn test_linear_resource_leak_detected() {
        let src = r#"#mod main

#fn bad_worker [] !fs
  file = fs.open "data.bin"
  = 42
"#;
        let parsed = parse_source(src);
        let (_, diags) = lower_module(&parsed.module);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "L001");
        assert!(diags[0].kind.contains("never consumed"));
    }

    #[test]
    fn test_linear_resource_valid_consumption() {
        let src = r#"#mod main

#fn good_worker [] !fs
  file = fs.open "data.bin"
  _ = fs.close file
  = 42
"#;
        let parsed = parse_source(src);
        let (_, diags) = lower_module(&parsed.module);
        assert!(diags.is_empty(), "expected zero diagnostics on valid close: {:?}", diags);
    }

    #[test]
    fn test_linear_resource_double_consumption_detected() {
        let src = r#"#mod main

#fn double_closer [] !fs
  file = fs.open "data.bin"
  _ = fs.close file
  _ = fs.close file
  = 42
"#;
        let parsed = parse_source(src);
        let (_, diags) = lower_module(&parsed.module);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "L002");
        assert!(diags[0].kind.contains("consumed more than once"));
    }
}
