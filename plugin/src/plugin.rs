
#![feature(box_syntax, plugin_registrar, quote, rustc_private)]

extern crate rustc;
extern crate syntax;

use rustc::plugin::Registry;
use syntax::ast::{Arm, Expr, Field, MetaItem, Mutability, Stmt};
use syntax::codemap::Span;
use syntax::ext::base::{Annotatable, MultiDecorator, ExtCtxt};
use syntax::ext::build::AstBuilder;
use syntax::ext::deriving::generic;
use syntax::parse::token;
use syntax::ptr::P;

#[plugin_registrar]
pub fn plugin_registrar(registry: &mut Registry) {
    registry.register_syntax_extension(token::intern("derive_CerealData"),
        MultiDecorator(box expand)
    );
}

pub fn expand(ecx: &mut ExtCtxt, span: Span, meta_item: &MetaItem, ann: &Annotatable, push: &mut FnMut(Annotatable)) {
    generic::TraitDef {
        span: span,
        attributes: Vec::new(),
        path: generic::ty::Path {
            path: vec!["cereal", "CerealData"],
            lifetime: None,
            params: Vec::new(),
            global: true,
        },
        generics: generic::ty::LifetimeBounds::empty(),
        additional_bounds: Vec::new(),
        associated_types: Vec::new(),
        methods: vec![
            generic::MethodDef {
                name: "read",
                attributes: Vec::new(),
                generics: generic::ty::LifetimeBounds::empty(),
                explicit_self: None,
                ret_ty: generic::ty::Literal(generic::ty::Path {
                    path: vec!["cereal", "CerealResult"],
                    lifetime: None,
                    params: vec![
                        box generic::ty::Self_,
                    ],
                    global: true,
                }),
                is_unsafe: false,
                args: vec![
                    generic::ty::Ptr(box generic::ty::Literal(generic::ty::Path {
                            path: vec!["std", "io", "Read"],
                            lifetime: None,
                            params: Vec::new(),
                            global: true,
                        }),
                        generic::ty::PtrTy::Borrowed(None, Mutability::MutMutable)
                    ),
                ],
                combine_substructure: generic::combine_substructure(box read_body),
            },
            generic::MethodDef {
                name: "write",
                attributes: Vec::new(),
                generics: generic::ty::LifetimeBounds::empty(),
                explicit_self: generic::ty::borrowed_explicit_self(),
                ret_ty: generic::ty::Literal(generic::ty::Path {
                    path: vec!["cereal", "CerealResult"],
                    lifetime: None,
                    params: vec![
                        box generic::ty::Tuple(Vec::new()),
                    ],
                    global: true,
                }),
                is_unsafe: false,
                args: vec![
                    generic::ty::Ptr(box generic::ty::Literal(generic::ty::Path {
                            path: vec!["std", "io", "Write"],
                            lifetime: None,
                            params: Vec::new(),
                            global: true,
                        }),
                        generic::ty::PtrTy::Borrowed(None, Mutability::MutMutable)
                    ),
                ],
                combine_substructure: generic::combine_substructure(box write_body),
            },
        ],
    }.expand(ecx, meta_item, &ann, push);
}

pub fn read_body(ecx: &mut ExtCtxt, span: Span, substr: &generic::Substructure) -> P<Expr> {
    let ref reader = substr.nonself_args[0];
    let expr = match *substr.fields {
        generic::StaticStruct(_, generic::Unnamed(ref fields)) if fields.is_empty() => {
            ecx.expr_ident(span, substr.type_ident)
        },
        generic::StaticStruct(_, generic::Named(ref fields)) if fields.is_empty() => {
            ecx.expr_ident(span, substr.type_ident)
        },
        generic::StaticStruct(_, generic::Unnamed(ref fields)) => {
            let all: Vec<P<Expr>> = (0..fields.len()).map(|_| {
                quote_expr!(ecx, try!(::cereal::CerealData::read($reader)))
            }).collect();

            ecx.expr_call_ident(span, substr.type_ident, all)
        },
        generic::StaticStruct(_, generic::Named(ref fields)) => {
            let all: Vec<Field> = fields.iter().map(|&(ident, _)| {
                ecx.field_imm(span, ident, quote_expr!(ecx, try!(::cereal::CerealData::read($reader))))
            }).collect();

            ecx.expr_struct_ident(span, substr.type_ident, all)
        },
        generic::StaticEnum(_, ref variants) => {
            let mut arms: Vec<Arm> = variants.iter().enumerate().map(|(id, &(ident, _, ref fields))| {
                let pat = ecx.pat_lit(span, ecx.expr_usize(span, id));
                let ty = substr.type_ident;
                let path = ecx.path_global(span, vec![ty, ident]);
                let expr = match *fields {
                    generic::Unnamed(ref fields) if fields.is_empty() => {
                        ecx.expr_path(path)
                    },
                    generic::Named(ref fields) if fields.is_empty() => {
                        ecx.expr_path(path)
                    },
                    generic::Unnamed(ref fields) => {
                        let all: Vec<P<Expr>> = (0..fields.len()).map(|_| {
                            quote_expr!(ecx, try!(::cereal::CerealData::read($reader)))
                        }).collect();

                        ecx.expr_call_global(span, vec![ty, ident], all)
                    },
                    generic::Named(ref fields) => {
                        let all: Vec<Field> = fields.iter().map(|&(ident, _)| {
                            ecx.field_imm(span, ident, quote_expr!(ecx, try!(::cereal::CerealData::read($reader))))
                        }).collect();

                        ecx.expr_struct(span, path, all)
                    },
                };
                ecx.arm(span, vec![pat], expr)
            }).collect();
            arms.push(ecx.arm(span, vec![ecx.pat_wild(span)],
                quote_expr!(ecx,
                    return ::std::result::Result::Err(::cereal::CerealError::Msg("Unknown variant".to_string()))
                )
            ));
            let expr = quote_expr!(ecx, try!(<usize as ::cereal::CerealData>::read($reader)));
            ecx.expr_match(span, expr, arms)
        },
        _ => {
            ecx.span_err(span, "Deriving CerealData for enums is currently unsupported!");
            return ecx.expr_none(span)
        },
    };
    quote_expr!(ecx, Ok($expr))
}

pub fn write_body(ecx: &mut ExtCtxt, span: Span, substr: &generic::Substructure) -> P<Expr> {
    let ref writer = substr.nonself_args[0];
    match *substr.fields {
        generic::Struct(ref fields) => {
            let all: Vec<P<Stmt>> = fields.iter().map(|f| {
                let ref self_ = f.self_;
                quote_stmt!(ecx, {
                    try!(::cereal::CerealData::write(&$self_, $writer));
                }).unwrap()
            }).collect();

            quote_expr!(ecx, {
                $all
                Ok(())
            })
        },
        generic::EnumMatching(var_id, _, ref fields) => {
            let all: Vec<P<Stmt>> = fields.iter().map(|f| {
                let ref self_ = f.self_;
                quote_stmt!(ecx, {
                    try!(::cereal::CerealData::write(&$self_, $writer));
                }).unwrap()
            }).collect();

            quote_expr!(ecx, {
                try!(::cereal::CerealData::write(&$var_id, $writer));
                $all
                Ok(())
            })
        },
        _ => {
            ecx.span_err(span, "Unsupported type for CerealData");
            ecx.expr_none(span)
        }
    }
}
