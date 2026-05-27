use rpython_ast::{
    Attribute, FieldDef, GenericParam, ImplItem, InterfaceItem, ItemKind, Param,
    TyKind, Variant, VariantFields,
};
use rpython_ast::{ItemId, Path, TyId, TyMutability};
use rpython_syntax::TokenKind;
use smol_str::SmolStr;

use crate::parser::Parser;

impl Parser<'_> {
    pub fn parse_module_items(&mut self) -> Option<Vec<ItemId>> {
        let mut items = Vec::new();
        self.skip_stmt_separators();
        while !self.is_at_end() {
            self.skip_stmt_separators();
            if self.is_at_end() {
                break;
            }
            if let Some(item) = self.parse_item() {
                items.push(item);
            } else if self.handler.has_errors() {
                self.synchronize();
            } else {
                break;
            }
            self.skip_stmt_separators();
        }
        Some(items)
    }

    pub fn parse_item(&mut self) -> Option<ItemId> {
        let attrs = self.parse_attrs()?;
        let is_pub = self.eat(TokenKind::KwPub);
        self.skip_layout();

        let start = self.current().span;
        let kind = match self.current_kind() {
            TokenKind::KwDef => self.parse_function_item(is_pub, attrs)?,
            TokenKind::KwClass => self.parse_class_item(is_pub, attrs)?,
            TokenKind::KwStruct => self.parse_struct_item(is_pub, attrs)?,
            TokenKind::KwEnum => self.parse_enum_item(is_pub, attrs)?,
            TokenKind::KwInterface | TokenKind::KwTrait => {
                self.parse_interface_item(is_pub, attrs)?
            }
            TokenKind::KwImpl => self.parse_impl_item(attrs)?,
            TokenKind::KwImport => self.parse_import_item()?,
            TokenKind::KwFrom => self.parse_from_import_item()?,
            _ => {
                self.error(start, "expected item");
                return None;
            }
        };
        let span = self.span_from(start);
        Some(self.arena.alloc_item(kind, span))
    }

    fn parse_attrs(&mut self) -> Option<Vec<Attribute>> {
        let mut attrs = Vec::new();
        while self.eat(TokenKind::At) {
            let name = match self.current_kind() {
                TokenKind::Ident { name } => {
                    let name = name.clone();
                    self.bump();
                    name
                }
                _ => {
                    self.error(self.current().span, "expected attribute name");
                    return None;
                }
            };
            let span = self.tokens[self.pos - 1].span;
            attrs.push(Attribute {
                name,
                args: Vec::new(),
                span,
            });
            self.skip_layout();
        }
        Some(attrs)
    }

    fn parse_generics(&mut self) -> Option<Vec<GenericParam>> {
        if !self.eat(TokenKind::LBracket) {
            return Some(Vec::new());
        }
        self.skip_layout();
        let mut params = Vec::new();
        if !self.eat(TokenKind::RBracket) {
            loop {
                let start = self.current().span;
                let name = self.parse_ident_name()?;
                params.push(GenericParam {
                    name,
                    bounds: Vec::new(),
                    span: self.span_from(start),
                });
                self.skip_layout();
                if self.eat(TokenKind::RBracket) {
                    break;
                }
                if !self.eat(TokenKind::Comma) {
                    self.error(self.current().span, "expected ',' or ']'");
                    return None;
                }
            }
        }
        Some(params)
    }

    fn parse_function_item(
        &mut self,
        is_pub: bool,
        attrs: Vec<Attribute>,
    ) -> Option<ItemKind> {
        self.bump();
        let name = self.parse_ident_name()?;
        let generics = self.parse_generics()?;
        self.skip_layout();
        if !self.eat(TokenKind::LParen) {
            self.error(self.current().span, "expected '(' after function name");
            return None;
        }
        let params = self.parse_params()?;
        let ret_ty = if self.eat(TokenKind::Arrow) {
            self.skip_layout();
            Some(self.parse_ty()?)
        } else {
            None
        };
        self.skip_layout();
        if !self.eat(TokenKind::Colon) {
            self.error(self.current().span, "expected ':' before function body");
            return None;
        }
        let body = self.parse_block()?;
        Some(ItemKind::Function {
            name,
            generics,
            params,
            ret_ty,
            body,
            is_pub,
            attrs,
        })
    }

    fn parse_params(&mut self) -> Option<Vec<Param>> {
        self.skip_layout();
        let mut params = Vec::new();
        if self.eat(TokenKind::RParen) {
            return Some(params);
        }
        loop {
            let start = self.current().span;
            let name = if self.eat(TokenKind::KwSelf) {
                SmolStr::new("self")
            } else {
                self.parse_ident_name()?
            };
            let ty = if self.eat(TokenKind::Colon) {
                self.skip_layout();
                Some(self.parse_ty()?)
            } else {
                None
            };
            let default = if self.eat(TokenKind::Assign) {
                self.skip_layout();
                Some(self.parse_expr()?)
            } else {
                None
            };
            params.push(Param {
                name,
                ty,
                default,
                span: self.span_from(start),
            });
            self.skip_layout();
            if self.eat(TokenKind::RParen) {
                break;
            }
            if !self.eat(TokenKind::Comma) {
                self.error(self.current().span, "expected ',' or ')'");
                return None;
            }
        }
        Some(params)
    }

    fn parse_class_item(&mut self, is_pub: bool, attrs: Vec<Attribute>) -> Option<ItemKind> {
        self.bump();
        let name = self.parse_ident_name()?;
        let generics = self.parse_generics()?;
        self.skip_layout();
        let bases = if self.eat(TokenKind::LParen) {
            let mut bases = Vec::new();
            if !self.eat(TokenKind::RParen) {
                loop {
                    bases.push(self.parse_ty()?);
                    self.skip_layout();
                    if self.eat(TokenKind::RParen) {
                        break;
                    }
                    if !self.eat(TokenKind::Comma) {
                        self.error(self.current().span, "expected ',' or ')'");
                        return None;
                    }
                }
            }
            bases
        } else {
            Vec::new()
        };
        self.skip_layout();
        if !self.eat(TokenKind::Colon) {
            self.error(self.current().span, "expected ':' after class header");
            return None;
        }
        let body = self.parse_item_block()?;
        Some(ItemKind::Class {
            name,
            generics,
            bases,
            body,
            is_pub,
            attrs,
        })
    }

    fn parse_struct_item(&mut self, is_pub: bool, attrs: Vec<Attribute>) -> Option<ItemKind> {
        self.bump();
        let name = self.parse_ident_name()?;
        let generics = self.parse_generics()?;
        self.skip_layout();
        if !self.eat(TokenKind::Colon) {
            self.error(self.current().span, "expected ':' after struct name");
            return None;
        }
        let fields = self.parse_struct_fields_block()?;
        Some(ItemKind::Struct {
            name,
            generics,
            fields,
            is_pub,
            attrs,
        })
    }

    fn parse_struct_fields_block(&mut self) -> Option<Vec<FieldDef>> {
        if !self.eat(TokenKind::Newline) || !self.eat(TokenKind::Indent) {
            self.error(self.current().span, "expected indented struct fields");
            return None;
        }
        let mut fields = Vec::new();
        while !matches!(self.current_kind(), TokenKind::Dedent | TokenKind::Eof) {
            self.skip_stmt_separators();
            if matches!(self.current_kind(), TokenKind::Dedent | TokenKind::Eof) {
                break;
            }
            let start = self.current().span;
            let name = self.parse_ident_name()?;
            if !self.eat(TokenKind::Colon) {
                self.error(self.current().span, "expected ':' after field name");
                return None;
            }
            self.skip_layout();
            let ty = self.parse_ty()?;
            fields.push(FieldDef {
                name,
                ty,
                span: self.span_from(start),
            });
            self.skip_stmt_separators();
        }
        self.eat(TokenKind::Dedent);
        Some(fields)
    }

    fn parse_enum_item(&mut self, is_pub: bool, attrs: Vec<Attribute>) -> Option<ItemKind> {
        self.bump();
        let name = self.parse_ident_name()?;
        let generics = self.parse_generics()?;
        self.skip_layout();
        if !self.eat(TokenKind::Colon) {
            self.error(self.current().span, "expected ':' after enum name");
            return None;
        }
        if !self.eat(TokenKind::Newline) || !self.eat(TokenKind::Indent) {
            self.error(self.current().span, "expected indented enum variants");
            return None;
        }
        let mut variants = Vec::new();
        while !matches!(self.current_kind(), TokenKind::Dedent | TokenKind::Eof) {
            self.skip_stmt_separators();
            if matches!(self.current_kind(), TokenKind::Dedent | TokenKind::Eof) {
                break;
            }
            let start = self.current().span;
            let vname = self.parse_ident_name()?;
            let fields = if self.eat(TokenKind::LParen) {
                let mut types = Vec::new();
                if !self.eat(TokenKind::RParen) {
                    loop {
                        types.push(self.parse_ty()?);
                        self.skip_layout();
                        if self.eat(TokenKind::RParen) {
                            break;
                        }
                        if !self.eat(TokenKind::Comma) {
                            self.error(self.current().span, "expected ',' or ')'");
                            return None;
                        }
                    }
                }
                VariantFields::Tuple(types)
            } else {
                VariantFields::Unit
            };
            variants.push(Variant {
                name: vname,
                fields,
                span: self.span_from(start),
            });
            self.skip_stmt_separators();
        }
        self.eat(TokenKind::Dedent);
        Some(ItemKind::Enum {
            name,
            generics,
            variants,
            is_pub,
            attrs,
        })
    }

    fn parse_interface_item(&mut self, is_pub: bool, attrs: Vec<Attribute>) -> Option<ItemKind> {
        self.bump();
        let name = self.parse_ident_name()?;
        let generics = self.parse_generics()?;
        self.skip_layout();
        if !self.eat(TokenKind::Colon) {
            self.error(self.current().span, "expected ':' after interface name");
            return None;
        }
        let items = self.parse_interface_item_block()?;
        Some(ItemKind::Interface {
            name,
            generics,
            items,
            is_pub,
            attrs,
        })
    }

    fn parse_interface_item_block(&mut self) -> Option<Vec<InterfaceItem>> {
        if !self.eat(TokenKind::Newline) || !self.eat(TokenKind::Indent) {
            self.error(self.current().span, "expected indented interface body");
            return None;
        }
        let mut items = Vec::new();
        while !matches!(self.current_kind(), TokenKind::Dedent | TokenKind::Eof) {
            self.skip_stmt_separators();
            if matches!(self.current_kind(), TokenKind::Dedent | TokenKind::Eof) {
                break;
            }
            if let Some(item) = self.parse_interface_member() {
                items.push(item);
            } else if self.handler.has_errors() {
                self.synchronize();
            } else {
                break;
            }
            self.skip_stmt_separators();
        }
        self.eat(TokenKind::Dedent);
        Some(items)
    }

    fn parse_interface_member(&mut self) -> Option<InterfaceItem> {
        let start = self.current().span;
        if !self.eat(TokenKind::KwDef) {
            self.error(start, "expected 'def' in interface body");
            return None;
        }
        let name = self.parse_ident_name()?;
        let generics = self.parse_generics()?;
        self.skip_layout();
        if !self.eat(TokenKind::LParen) {
            self.error(self.current().span, "expected '('");
            return None;
        }
        let params = self.parse_params()?;
        let ret_ty = if self.eat(TokenKind::Arrow) {
            self.skip_layout();
            Some(self.parse_ty()?)
        } else {
            None
        };
        self.skip_layout();
        let default_body = if self.eat(TokenKind::Colon) {
            Some(self.parse_block()?)
        } else {
            None
        };
        Some(InterfaceItem::Function {
            name,
            generics,
            params,
            ret_ty,
            default_body,
            span: self.span_from(start),
        })
    }

    fn parse_impl_item(&mut self, attrs: Vec<Attribute>) -> Option<ItemKind> {
        self.bump();
        let generics = self.parse_generics()?;
        self.skip_layout();
        let path = self.parse_path()?;
        let (interface_ref, self_ty) = if self.eat(TokenKind::KwFor) {
            self.skip_layout();
            (Some(path), self.parse_ty()?)
        } else {
            (None, self.path_to_ty(path)?)
        };
        self.skip_layout();
        if !self.eat(TokenKind::Colon) {
            self.error(self.current().span, "expected ':' after impl header");
            return None;
        }
        let items = self.parse_impl_item_block()?;
        Some(ItemKind::Impl {
            generics,
            interface_ref,
            self_ty,
            items,
            attrs,
        })
    }

    fn parse_impl_item_block(&mut self) -> Option<Vec<ImplItem>> {
        if !self.eat(TokenKind::Newline) || !self.eat(TokenKind::Indent) {
            self.error(self.current().span, "expected indented impl body");
            return None;
        }
        let mut items = Vec::new();
        while !matches!(self.current_kind(), TokenKind::Dedent | TokenKind::Eof) {
            self.skip_stmt_separators();
            if matches!(self.current_kind(), TokenKind::Dedent | TokenKind::Eof) {
                break;
            }
            if let Some(item) = self.parse_impl_member() {
                items.push(item);
            } else if self.handler.has_errors() {
                self.synchronize();
            } else {
                break;
            }
            self.skip_stmt_separators();
        }
        self.eat(TokenKind::Dedent);
        Some(items)
    }

    fn parse_impl_member(&mut self) -> Option<ImplItem> {
        let start = self.current().span;
        if !self.eat(TokenKind::KwDef) {
            self.error(start, "expected 'def' in impl body");
            return None;
        }
        let name = self.parse_ident_name()?;
        let generics = self.parse_generics()?;
        self.skip_layout();
        if !self.eat(TokenKind::LParen) {
            self.error(self.current().span, "expected '('");
            return None;
        }
        let params = self.parse_params()?;
        let ret_ty = if self.eat(TokenKind::Arrow) {
            self.skip_layout();
            Some(self.parse_ty()?)
        } else {
            None
        };
        self.skip_layout();
        if !self.eat(TokenKind::Colon) {
            self.error(self.current().span, "expected ':' before impl method body");
            return None;
        }
        let body = self.parse_block()?;
        Some(ImplItem::Function {
            name,
            generics,
            params,
            ret_ty,
            body,
            span: self.span_from(start),
        })
    }

    fn path_to_ty(&mut self, path: Path) -> Option<TyId> {
        let span = path.span;
        Some(self.arena.alloc_ty(TyKind::Path(path), span))
    }

    fn parse_import_item(&mut self) -> Option<ItemKind> {
        self.bump();
        self.skip_layout();
        let path = self.parse_path()?;
        let alias = if self.eat(TokenKind::KwAs) {
            self.skip_layout();
            Some(self.parse_ident_name()?)
        } else {
            None
        };
        Some(ItemKind::Import { path, alias })
    }

    fn parse_from_import_item(&mut self) -> Option<ItemKind> {
        self.bump();
        self.skip_layout();
        let mut path = self.parse_path()?;
        if !self.eat(TokenKind::KwImport) {
            self.error(self.current().span, "expected 'import' in from-import");
            return None;
        }
        self.skip_layout();
        let name = self.parse_ident_name()?;
        let alias = if self.eat(TokenKind::KwAs) {
            self.skip_layout();
            Some(self.parse_ident_name()?)
        } else {
            None
        };
        let span = path.span;
        path.segments.push(rpython_ast::PathSegment {
            ident: name,
            args: Vec::new(),
            span,
        });
        Some(ItemKind::Import { path, alias })
    }

    fn parse_item_block(&mut self) -> Option<Vec<ItemId>> {
        if !self.eat(TokenKind::Newline) || !self.eat(TokenKind::Indent) {
            self.error(self.current().span, "expected indented item block");
            return None;
        }
        let mut items = Vec::new();
        while !matches!(self.current_kind(), TokenKind::Dedent | TokenKind::Eof) {
            self.skip_stmt_separators();
            if matches!(self.current_kind(), TokenKind::Dedent | TokenKind::Eof) {
                break;
            }
            if let Some(item) = self.parse_item() {
                items.push(item);
            } else if self.handler.has_errors() {
                self.synchronize();
            } else {
                break;
            }
            self.skip_stmt_separators();
        }
        self.eat(TokenKind::Dedent);
        Some(items)
    }

    fn parse_ident_name(&mut self) -> Option<SmolStr> {
        match self.current_kind() {
            TokenKind::Ident { name } => {
                let name = name.clone();
                self.bump();
                Some(name)
            }
            _ => {
                self.error(self.current().span, "expected identifier");
                None
            }
        }
    }

    pub fn parse_ty(&mut self) -> Option<TyId> {
        self.skip_layout();
        let start = self.current().span;

        if self.eat(TokenKind::AmpMut) {
            self.skip_layout();
            let inner = self.parse_ty()?;
            let span = self.span_from(start);
            return Some(self.arena.alloc_ty(
                TyKind::Ref {
                    mutability: TyMutability::Mut,
                    inner,
                },
                span,
            ));
        }
        if self.eat(TokenKind::Amp) {
            self.skip_layout();
            let inner = self.parse_ty()?;
            let span = self.span_from(start);
            return Some(self.arena.alloc_ty(
                TyKind::Ref {
                    mutability: TyMutability::Imm,
                    inner,
                },
                span,
            ));
        }

        if self.eat(TokenKind::LParen) {
            self.skip_layout();
            let mut elems = Vec::new();
            if !self.eat(TokenKind::RParen) {
                loop {
                    elems.push(self.parse_ty()?);
                    self.skip_layout();
                    if self.eat(TokenKind::RParen) {
                        break;
                    }
                    if !self.eat(TokenKind::Comma) {
                        self.error(self.current().span, "expected ',' or ')'");
                        return None;
                    }
                }
            }
            let span = self.span_from(start);
            return Some(self.arena.alloc_ty(TyKind::Tuple(elems), span));
        }

        let path = self.parse_path()?;
        let span = path.span;
        Some(self.arena.alloc_ty(TyKind::Path(path), span))
    }
}
