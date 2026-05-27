use crate::{
    expr::Mutability as ExprMutability, Abi, Arena, Attribute, BinaryOp, ElifArm, ExprId,
    ExternItem, FieldDef, FieldExpr, GenericParam, ImplItem, InterfaceItem, ItemId, Kwarg, Literal,
    MatchArm, Param, PatField, PatId, StmtId, TyId, UnaryOp, Variant, VariantFields,
};
use crate::{ExprKind, ItemKind, PatKind, StmtKind, TyKind};
use crate::{Module, Path};
use std::fmt::{self, Write};

/// Pretty-printer for AST nodes (used in tests and debugging).
pub struct AstPrinter<'a> {
    arena: &'a Arena,
    indent: usize,
    buf: String,
}

impl<'a> AstPrinter<'a> {
    pub fn new(arena: &'a Arena) -> Self {
        Self {
            arena,
            indent: 0,
            buf: String::new(),
        }
    }

    pub fn print_module(mut self, module: &Module) -> String {
        self.line("Module {");
        self.bump();
        for &item in &module.items {
            self.print_item(item);
        }
        self.unbump();
        self.line("}");
        self.buf
    }

    pub fn print_item(&mut self, id: ItemId) {
        let item = self.arena.item(id);
        self.line(&format!("Item @{} {{", id.index()));
        self.bump();
        match &item.kind {
            ItemKind::Function {
                name,
                generics,
                params,
                ret_ty,
                body,
                is_pub,
                attrs,
            } => {
                self.print_attrs(attrs);
                self.kv("vis", if *is_pub { "pub" } else { "private" });
                self.kv("kind", "Function");
                self.kv("name", name.as_str());
                self.print_generics(generics);
                self.print_params(params);
                if let Some(ret) = ret_ty {
                    self.print_ty("ret", *ret);
                }
                self.print_stmt_block("body", body);
            }
            ItemKind::Class {
                name,
                generics,
                bases,
                body,
                is_pub,
                attrs,
            } => {
                self.print_attrs(attrs);
                self.kv("vis", if *is_pub { "pub" } else { "private" });
                self.kv("kind", "Class");
                self.kv("name", name.as_str());
                self.print_generics(generics);
                self.line("bases [");
                self.bump();
                for &base in bases {
                    self.print_ty_node(base);
                }
                self.unbump();
                self.line("]");
                self.line("body [");
                self.bump();
                for &nested in body {
                    self.print_item(nested);
                }
                self.unbump();
                self.line("]");
            }
            ItemKind::Struct {
                name,
                generics,
                fields,
                is_pub,
                attrs,
            } => {
                self.print_attrs(attrs);
                self.kv("vis", if *is_pub { "pub" } else { "private" });
                self.kv("kind", "Struct");
                self.kv("name", name.as_str());
                self.print_generics(generics);
                self.print_field_defs(fields);
            }
            ItemKind::Enum {
                name,
                generics,
                variants,
                is_pub,
                attrs,
            } => {
                self.print_attrs(attrs);
                self.kv("vis", if *is_pub { "pub" } else { "private" });
                self.kv("kind", "Enum");
                self.kv("name", name.as_str());
                self.print_generics(generics);
                self.print_variants(variants);
            }
            ItemKind::Interface {
                name,
                generics,
                items,
                is_pub,
                attrs,
            } => {
                self.print_attrs(attrs);
                self.kv("vis", if *is_pub { "pub" } else { "private" });
                self.kv("kind", "Interface");
                self.kv("name", name.as_str());
                self.print_generics(generics);
                self.line("items [");
                self.bump();
                for item in items {
                    self.print_interface_item(item);
                }
                self.unbump();
                self.line("]");
            }
            ItemKind::Impl {
                generics,
                interface_ref,
                self_ty,
                items,
                attrs,
            } => {
                self.print_attrs(attrs);
                self.kv("kind", "Impl");
                self.print_generics(generics);
                if let Some(path) = interface_ref {
                    self.print_path("interface", path);
                }
                self.print_ty("self_ty", *self_ty);
                self.line("items [");
                self.bump();
                for item in items {
                    self.print_impl_item(item);
                }
                self.unbump();
                self.line("]");
            }
            ItemKind::Const {
                name,
                ty,
                value,
                is_pub,
            } => {
                self.kv("vis", if *is_pub { "pub" } else { "private" });
                self.kv("kind", "Const");
                self.kv("name", name.as_str());
                self.print_ty("ty", *ty);
                self.print_expr("value", *value);
            }
            ItemKind::Import { path, alias } => {
                self.kv("kind", "Import");
                self.print_path("path", path);
                if let Some(alias) = alias {
                    self.kv("alias", alias.as_str());
                }
            }
            ItemKind::ExternBlock { abi, items } => {
                self.kv("kind", "ExternBlock");
                self.kv("abi", &format_abi(abi));
                self.line("items [");
                self.bump();
                for item in items {
                    self.print_extern_item(item);
                }
                self.unbump();
                self.line("]");
            }
            ItemKind::Module { name, items } => {
                self.kv("kind", "Module");
                self.kv("name", name.as_str());
                self.line("items [");
                self.bump();
                for &nested in items {
                    self.print_item(nested);
                }
                self.unbump();
                self.line("]");
            }
        }
        self.unbump();
        self.line("}");
    }

    pub fn print_stmt(&mut self, id: StmtId) {
        let stmt = self.arena.stmt(id);
        self.line(&format!("Stmt @{} {{", id.index()));
        self.bump();
        match &stmt.kind {
            StmtKind::Expr(expr) => {
                self.kv("kind", "Expr");
                self.print_expr("expr", *expr);
            }
            StmtKind::Assign { targets, value } => {
                self.kv("kind", "Assign");
                self.line("targets [");
                self.bump();
                for &pat in targets {
                    self.print_pat(pat);
                }
                self.unbump();
                self.line("]");
                self.print_expr("value", *value);
            }
            StmtKind::AnnAssign { target, ty, value } => {
                self.kv("kind", "AnnAssign");
                self.print_pat(*target);
                self.print_ty("ty", *ty);
                if let Some(val) = value {
                    self.print_expr("value", *val);
                }
            }
            StmtKind::Return(expr) => {
                self.kv("kind", "Return");
                if let Some(expr) = expr {
                    self.print_expr("value", *expr);
                }
            }
            StmtKind::Raise(expr) => {
                self.kv("kind", "Raise");
                self.print_expr("exc", *expr);
            }
            StmtKind::Assert { test, msg } => {
                self.kv("kind", "Assert");
                self.print_expr("test", *test);
                if let Some(msg) = msg {
                    self.print_expr("msg", *msg);
                }
            }
            StmtKind::Pass => self.kv("kind", "Pass"),
            StmtKind::Break(label) => {
                self.kv("kind", "Break");
                if let Some(label) = label {
                    self.kv("label", label.0.as_str());
                }
            }
            StmtKind::Continue(label) => {
                self.kv("kind", "Continue");
                if let Some(label) = label {
                    self.kv("label", label.0.as_str());
                }
            }
            StmtKind::While { test, body } => {
                self.kv("kind", "While");
                self.print_expr("test", *test);
                self.print_stmt_block("body", body);
            }
            StmtKind::For { pat, iter, body } => {
                self.kv("kind", "For");
                self.print_pat(*pat);
                self.print_expr("iter", *iter);
                self.print_stmt_block("body", body);
            }
            StmtKind::If {
                test,
                then_body,
                elifs,
                else_body,
            } => {
                self.kv("kind", "If");
                self.print_expr("test", *test);
                self.print_stmt_block("then", then_body);
                for (i, elif) in elifs.iter().enumerate() {
                    self.print_elif(i, elif);
                }
                if let Some(body) = else_body {
                    self.print_stmt_block("else", body);
                }
            }
            StmtKind::Match { scrutinee, arms } => {
                self.kv("kind", "Match");
                self.print_expr("scrutinee", *scrutinee);
                self.line("arms [");
                self.bump();
                for arm in arms {
                    self.print_match_arm(arm);
                }
                self.unbump();
                self.line("]");
            }
        }
        self.unbump();
        self.line("}");
    }

    pub fn print_expr(&mut self, label: &str, id: ExprId) {
        let expr = self.arena.expr(id);
        self.line(&format!("{label} @{} {{", id.index()));
        self.bump();
        match &expr.kind {
            ExprKind::Literal(lit) => {
                self.kv("kind", "Literal");
                self.kv("value", &format_literal(lit));
            }
            ExprKind::Path(path) => {
                self.kv("kind", "Path");
                self.print_path_inline(path);
            }
            ExprKind::Call { func, args, kwargs } => {
                self.kv("kind", "Call");
                self.print_expr("func", *func);
                self.line("args [");
                self.bump();
                for &arg in args {
                    self.print_expr_node(arg);
                }
                self.unbump();
                self.line("]");
                if !kwargs.is_empty() {
                    self.line("kwargs [");
                    self.bump();
                    for kw in kwargs {
                        self.print_kwarg(kw);
                    }
                    self.unbump();
                    self.line("]");
                }
            }
            ExprKind::MethodCall {
                receiver,
                method,
                args,
            } => {
                self.kv("kind", "MethodCall");
                self.print_expr("receiver", *receiver);
                self.kv("method", method.as_str());
                self.line("args [");
                self.bump();
                for &arg in args {
                    self.print_expr_node(arg);
                }
                self.unbump();
                self.line("]");
            }
            ExprKind::Field { base, field } => {
                self.kv("kind", "Field");
                self.print_expr("base", *base);
                self.kv("field", field.as_str());
            }
            ExprKind::Index { base, index } => {
                self.kv("kind", "Index");
                self.print_expr("base", *base);
                self.print_expr("index", *index);
            }
            ExprKind::Unary { op, operand } => {
                self.kv("kind", "Unary");
                self.kv("op", format_unary_op(*op));
                self.print_expr("operand", *operand);
            }
            ExprKind::Binary { op, left, right } => {
                self.kv("kind", "Binary");
                self.kv("op", format_binary_op(*op));
                self.print_expr("left", *left);
                self.print_expr("right", *right);
            }
            ExprKind::Tuple(elems) | ExprKind::List(elems) => {
                self.kv(
                    "kind",
                    if matches!(expr.kind, ExprKind::Tuple(_)) {
                        "Tuple"
                    } else {
                        "List"
                    },
                );
                self.line("elems [");
                self.bump();
                for &elem in elems {
                    self.print_expr_node(elem);
                }
                self.unbump();
                self.line("]");
            }
            ExprKind::Struct { path, fields } => {
                self.kv("kind", "Struct");
                self.print_path("path", path);
                self.line("fields [");
                self.bump();
                for field in fields {
                    self.print_field_expr(field);
                }
                self.unbump();
                self.line("]");
            }
            ExprKind::If {
                test,
                then,
                else_branch,
            } => {
                self.kv("kind", "If");
                self.print_expr("test", *test);
                self.print_expr("then", *then);
                self.print_expr("else", *else_branch);
            }
            ExprKind::Block(stmts) => {
                self.kv("kind", "Block");
                self.print_stmt_block("stmts", stmts);
            }
            ExprKind::Lambda { params, body } => {
                self.kv("kind", "Lambda");
                self.line("params [");
                self.bump();
                for param in params {
                    self.line(&format!("{}: {:?}", param.name, param.ty));
                }
                self.unbump();
                self.line("]");
                self.print_expr("body", *body);
            }
            ExprKind::Cast { expr, ty } => {
                self.kv("kind", "Cast");
                self.print_expr("expr", *expr);
                self.print_ty("ty", *ty);
            }
            ExprKind::Ref { mutability, expr } => {
                self.kv("kind", "Ref");
                self.kv("mut", format_expr_mut(*mutability));
                self.print_expr("expr", *expr);
            }
            ExprKind::Deref(expr) => {
                self.kv("kind", "Deref");
                self.print_expr("expr", *expr);
            }
        }
        self.unbump();
        self.line("}");
    }

    pub fn print_pat(&mut self, id: PatId) {
        let pat = self.arena.pat(id);
        self.line(&format!("Pat @{} {{", id.index()));
        self.bump();
        match &pat.kind {
            PatKind::Wild => self.kv("kind", "Wild"),
            PatKind::Ident {
                name,
                mutability,
                subpat,
            } => {
                self.kv("kind", "Ident");
                self.kv("name", name.as_str());
                self.kv("mut", format_pat_mut(*mutability));
                if let Some(sub) = subpat {
                    self.print_pat(*sub);
                }
            }
            PatKind::Literal(lit) => {
                self.kv("kind", "Literal");
                self.kv("value", &format_literal(lit));
            }
            PatKind::Tuple(pats) => {
                self.kv("kind", "Tuple");
                self.line("pats [");
                self.bump();
                for &p in pats {
                    self.print_pat(p);
                }
                self.unbump();
                self.line("]");
            }
            PatKind::Struct { path, fields } => {
                self.kv("kind", "Struct");
                self.print_path("path", path);
                self.line("fields [");
                self.bump();
                for field in fields {
                    self.print_pat_field(field);
                }
                self.unbump();
                self.line("]");
            }
            PatKind::Enum {
                path,
                variant,
                subpats,
            } => {
                self.kv("kind", "Enum");
                self.print_path("path", path);
                self.kv("variant", variant.as_str());
                self.line("subpats [");
                self.bump();
                for &p in subpats {
                    self.print_pat(p);
                }
                self.unbump();
                self.line("]");
            }
            PatKind::Or(pats) => {
                self.kv("kind", "Or");
                self.line("pats [");
                self.bump();
                for &p in pats {
                    self.print_pat(p);
                }
                self.unbump();
                self.line("]");
            }
        }
        self.unbump();
        self.line("}");
    }

    pub fn print_ty(&mut self, label: &str, id: TyId) {
        let ty = self.arena.ty(id);
        self.line(&format!("{label} @{} {{", id.index()));
        self.bump();
        self.print_ty_kind(&ty.kind);
        self.unbump();
        self.line("}");
    }

    fn print_ty_node(&mut self, id: TyId) {
        self.print_ty("ty", id);
    }

    fn print_ty_kind(&mut self, kind: &TyKind) {
        match kind {
            TyKind::Path(path) => {
                self.kv("kind", "Path");
                self.print_path_inline(path);
            }
            TyKind::Tuple(elems) => {
                self.kv("kind", "Tuple");
                self.line("elems [");
                self.bump();
                for &elem in elems {
                    self.print_ty_node(elem);
                }
                self.unbump();
                self.line("]");
            }
            TyKind::Array { elem, len } => {
                self.kv("kind", "Array");
                self.print_ty_node(*elem);
                if let Some(len) = len {
                    self.kv("len", &len.to_string());
                }
            }
            TyKind::Slice { elem } => {
                self.kv("kind", "Slice");
                self.print_ty_node(*elem);
            }
            TyKind::Ref { mutability, inner } => {
                self.kv("kind", "Ref");
                self.kv("mut", format_ty_mut(*mutability));
                self.print_ty_node(*inner);
            }
            TyKind::Fn { params, ret } => {
                self.kv("kind", "Fn");
                self.line("params [");
                self.bump();
                for &param in params {
                    self.print_ty_node(param);
                }
                self.unbump();
                self.line("]");
                if let Some(ret) = ret {
                    self.print_ty("ret", *ret);
                }
            }
            TyKind::GenericParam { name } => {
                self.kv("kind", "GenericParam");
                self.kv("name", name.as_str());
            }
        }
    }

    fn print_stmt_block(&mut self, label: &str, stmts: &[StmtId]) {
        self.line(&format!("{label} ["));
        self.bump();
        for &stmt in stmts {
            self.print_stmt(stmt);
        }
        self.unbump();
        self.line("]");
    }

    fn print_expr_node(&mut self, id: ExprId) {
        self.print_expr("expr", id);
    }

    fn print_attrs(&mut self, attrs: &[Attribute]) {
        if attrs.is_empty() {
            return;
        }
        self.line("attrs [");
        self.bump();
        for attr in attrs {
            self.line(&format!("@{}(...)", attr.name));
        }
        self.unbump();
        self.line("]");
    }

    fn print_generics(&mut self, generics: &[GenericParam]) {
        if generics.is_empty() {
            return;
        }
        self.line("generics [");
        self.bump();
        for g in generics {
            self.line(&format!("{}", g.name));
        }
        self.unbump();
        self.line("]");
    }

    fn print_params(&mut self, params: &[Param]) {
        self.line("params [");
        self.bump();
        for p in params {
            self.line(&format!("{}: {:?}", p.name, p.ty));
        }
        self.unbump();
        self.line("]");
    }

    fn print_field_defs(&mut self, fields: &[FieldDef]) {
        self.line("fields [");
        self.bump();
        for f in fields {
            self.line(&format!("{}:", f.name));
            self.bump();
            self.print_ty_node(f.ty);
            self.unbump();
        }
        self.unbump();
        self.line("]");
    }

    fn print_variants(&mut self, variants: &[Variant]) {
        self.line("variants [");
        self.bump();
        for v in variants {
            self.line(&format!("{} {{", v.name));
            self.bump();
            match &v.fields {
                VariantFields::Unit => self.kv("fields", "Unit"),
                VariantFields::Tuple(tys) => {
                    self.line("tuple [");
                    self.bump();
                    for &ty in tys {
                        self.print_ty_node(ty);
                    }
                    self.unbump();
                    self.line("]");
                }
                VariantFields::Struct(defs) => self.print_field_defs(defs),
            }
            self.unbump();
            self.line("}");
        }
        self.unbump();
        self.line("]");
    }

    fn print_interface_item(&mut self, item: &InterfaceItem) {
        match item {
            InterfaceItem::Function {
                name,
                generics,
                params,
                ret_ty,
                default_body,
                ..
            } => {
                self.line(&format!("fn {}(...)", name));
                self.bump();
                self.print_generics(generics);
                self.print_params(params);
                if let Some(ret) = ret_ty {
                    self.print_ty("ret", *ret);
                }
                if let Some(body) = default_body {
                    self.print_stmt_block("default", body);
                }
                self.unbump();
            }
            InterfaceItem::Type { name, ty, .. } => {
                self.line(&format!("type {}", name));
                if let Some(ty) = ty {
                    self.print_ty("ty", *ty);
                }
            }
        }
    }

    fn print_impl_item(&mut self, item: &ImplItem) {
        match item {
            ImplItem::Function {
                name,
                generics,
                params,
                ret_ty,
                body,
                ..
            } => {
                self.line(&format!("fn {}(...)", name));
                self.bump();
                self.print_generics(generics);
                self.print_params(params);
                if let Some(ret) = ret_ty {
                    self.print_ty("ret", *ret);
                }
                self.print_stmt_block("body", body);
                self.unbump();
            }
            ImplItem::Const {
                name, ty, value, ..
            } => {
                self.line(&format!("const {}", name));
                self.print_ty("ty", *ty);
                self.print_expr("value", *value);
            }
            ImplItem::Type { name, ty, .. } => {
                self.line(&format!("type {}", name));
                self.print_ty("ty", *ty);
            }
        }
    }

    fn print_extern_item(&mut self, item: &ExternItem) {
        match item {
            ExternItem::Function { name, .. } => self.line(&format!("extern fn {}", name)),
            ExternItem::Static { name, .. } => self.line(&format!("extern static {}", name)),
        }
    }

    fn print_elif(&mut self, i: usize, elif: &ElifArm) {
        self.line(&format!("elif[{i}] {{"));
        self.bump();
        self.print_expr("test", elif.test);
        self.print_stmt_block("body", &elif.body);
        self.unbump();
        self.line("}");
    }

    fn print_match_arm(&mut self, arm: &MatchArm) {
        self.line("arm {");
        self.bump();
        self.print_pat(arm.pat);
        if let Some(guard) = arm.guard {
            self.print_expr("guard", guard);
        }
        self.print_stmt_block("body", &arm.body);
        self.unbump();
        self.line("}");
    }

    fn print_kwarg(&mut self, kw: &Kwarg) {
        self.line(&format!("{} =", kw.name));
        self.bump();
        self.print_expr_node(kw.value);
        self.unbump();
    }

    fn print_field_expr(&mut self, field: &FieldExpr) {
        self.line(&format!("{}:", field.name));
        self.bump();
        self.print_expr_node(field.expr);
        self.unbump();
    }

    fn print_pat_field(&mut self, field: &PatField) {
        self.line(&format!("{}:", field.name));
        self.bump();
        self.print_pat(field.pat);
        self.unbump();
    }

    fn print_path(&mut self, label: &str, path: &Path) {
        self.line(&format!("{label}: {}", format_path(path)));
    }

    fn print_path_inline(&mut self, path: &Path) {
        self.kv("path", &format_path(path));
    }

    fn line(&mut self, s: &str) {
        for _ in 0..self.indent {
            self.buf.push_str("  ");
        }
        self.buf.push_str(s);
        self.buf.push('\n');
    }

    fn kv(&mut self, key: &str, value: &str) {
        self.line(&format!("{key}: {value}"));
    }

    fn bump(&mut self) {
        self.indent += 1;
    }

    fn unbump(&mut self) {
        self.indent -= 1;
    }
}

/// Format a module to a string.
pub fn format_module(module: &Module, arena: &Arena) -> String {
    AstPrinter::new(arena).print_module(module)
}

impl<'a> fmt::Display for AstPrinter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.buf)
    }
}

fn format_path(path: &Path) -> String {
    let mut out = String::new();
    for (i, seg) in path.segments.iter().enumerate() {
        if i > 0 {
            out.push('.');
        }
        out.push_str(seg.ident.as_str());
        if !seg.args.is_empty() {
            let _ = write!(out, "[{} type args]", seg.args.len());
        }
    }
    out
}

fn format_literal(lit: &Literal) -> String {
    match lit {
        Literal::Int(n) => n.to_string(),
        Literal::Float(x) => x.to_string(),
        Literal::String(s) => format!("{s:?}"),
        Literal::Bytes(b) => format!("b{bytes:?}", bytes = String::from_utf8_lossy(b)),
        Literal::Bool(b) => b.to_string(),
        Literal::Char(c) => format!("'{c}'"),
        Literal::None => "None".to_string(),
    }
}

fn format_unary_op(op: UnaryOp) -> &'static str {
    match op {
        UnaryOp::Not => "not",
        UnaryOp::Neg => "-",
        UnaryOp::Pos => "+",
        UnaryOp::BitNot => "~",
    }
}

fn format_binary_op(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+",
        BinaryOp::Sub => "-",
        BinaryOp::Mul => "*",
        BinaryOp::Div => "/",
        BinaryOp::Mod => "%",
        BinaryOp::FloorDiv => "//",
        BinaryOp::Pow => "**",
        BinaryOp::Eq => "==",
        BinaryOp::NotEq => "!=",
        BinaryOp::Lt => "<",
        BinaryOp::LtEq => "<=",
        BinaryOp::Gt => ">",
        BinaryOp::GtEq => ">=",
        BinaryOp::And => "and",
        BinaryOp::Or => "or",
        BinaryOp::BitAnd => "&",
        BinaryOp::BitOr => "|",
        BinaryOp::BitXor => "^",
        BinaryOp::Shl => "<<",
        BinaryOp::Shr => ">>",
        BinaryOp::Is => "is",
        BinaryOp::In => "in",
    }
}

fn format_expr_mut(m: ExprMutability) -> &'static str {
    match m {
        ExprMutability::Imm => "imm",
        ExprMutability::Mut => "mut",
    }
}

fn format_pat_mut(m: crate::pat::Mutability) -> &'static str {
    match m {
        crate::pat::Mutability::Imm => "imm",
        crate::pat::Mutability::Mut => "mut",
    }
}

fn format_ty_mut(m: crate::ty::Mutability) -> &'static str {
    match m {
        crate::ty::Mutability::Imm => "imm",
        crate::ty::Mutability::Mut => "mut",
    }
}

fn format_abi(abi: &Abi) -> String {
    match abi {
        Abi::C => "C".to_string(),
        Abi::RPython => "rpython".to_string(),
        Abi::Other(s) => s.to_string(),
    }
}

/// Convenience alias used in tests.
pub type PrettyPrinter<'a> = AstPrinter<'a>;
