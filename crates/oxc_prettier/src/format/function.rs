use oxc_ast::ast::*;

use crate::{
    dynamic_text,
    format::function_parameters::should_group_function_parameters,
    group, if_break, indent,
    ir::{Doc, DocBuilder},
    softline, space, text, Format, Prettier,
};

pub(super) fn print_function<'a>(
    p: &mut Prettier<'a>,
    func: &Function<'a>,
    property_name: Option<&str>,
) -> Doc<'a> {
    let mut parts = p.vec();

    if func.declare {
        parts.push(text!("declare "));
    }

    if func.r#async {
        parts.push(text!("async "));
    }

    if let Some(name) = property_name {
        parts.push(dynamic_text!(p, name));
    } else {
        parts.push(text!("function"));
        if func.generator {
            parts.push(text!("*"));
        }

        parts.push(text!(" "));
    }

    if let Some(id) = &func.id {
        parts.push(dynamic_text!(p, id.name.as_str()));
    }

    if let Some(type_params) = &func.type_parameters {
        parts.push(type_params.format(p));
    }
    // Prettier has `returnTypeDoc` to group together, write this for keep same with prettier.
    parts.push(group!(p, {
        if should_group_function_parameters(func) {
            group!(p, func.params.format(p))
        } else {
            func.params.format(p)
        }
    }));

    if let Some(return_type) = &func.return_type {
        parts.push(text!(": "));
        parts.push(return_type.type_annotation.format(p));
    }

    if let Some(body) = &func.body {
        parts.push(space!());
        parts.push(body.format(p));
    }
    if func.is_ts_declare_function() || func.body.is_none() {
        if let Some(semi) = p.semi() {
            parts.push(semi);
        }
    }

    Doc::Array(parts)
}

pub(super) fn print_method<'a>(p: &mut Prettier<'a>, method: &MethodDefinition<'a>) -> Doc<'a> {
    let mut parts = p.vec();

    if let Some(accessibility) = &method.accessibility {
        parts.push(text!(accessibility.as_str()));
        parts.push(space!());
    }

    if method.r#static {
        parts.push(text!("static "));
    }

    if matches!(method.r#type, MethodDefinitionType::TSAbstractMethodDefinition) {
        parts.push(text!("abstract "));
    }

    if method.r#override {
        parts.push(text!("override "));
    }

    match method.kind {
        MethodDefinitionKind::Constructor | MethodDefinitionKind::Method => {}
        MethodDefinitionKind::Get => {
            parts.push(text!("get "));
        }
        MethodDefinitionKind::Set => {
            parts.push(text!("set "));
        }
    }

    if method.value.r#async {
        parts.push(text!("async "));
    }

    if method.value.generator {
        parts.push(text!("*"));
    }

    parts.push(method.key.format(p));

    if method.optional {
        parts.push(text!("?"));
    }

    parts.push(print_method_value(p, &method.value));

    Doc::Array(parts)
}

fn print_method_value<'a>(p: &mut Prettier<'a>, function: &Function<'a>) -> Doc<'a> {
    let mut parts = p.vec();
    let parameters_doc = function.params.format(p);
    let should_group_parameters = should_group_function_parameters(function);
    let parameters_doc =
        if should_group_parameters { group!(p, parameters_doc) } else { parameters_doc };

    if let Some(type_parameters) = &function.type_parameters {
        parts.push(type_parameters.format(p));
    }

    parts.push(group!(p, parameters_doc));

    if let Some(ret_typ) = &function.return_type {
        parts.push(text!(": "));
        parts.push(ret_typ.type_annotation.format(p));
    }

    if let Some(body) = &function.body {
        parts.push(space!());
        parts.push(body.format(p));
    } else if p.options.semi {
        parts.push(text!(";"));
    }

    Doc::Array(parts)
}

pub(super) fn print_return_or_throw_argument<'a>(
    p: &mut Prettier<'a>,
    argument: Option<&Expression<'a>>,
    is_return: bool,
) -> Doc<'a> {
    let mut parts = p.vec();

    parts.push(text!(if is_return { "return" } else { "throw" }));

    if let Some(argument) = argument {
        parts.push(space!());
        parts.push(
            if argument.is_binaryish() || matches!(argument, Expression::SequenceExpression(_)) {
                group![
                    p,
                    if_break!(p, "("),
                    indent!(p, softline!(), argument.format(p)),
                    softline!(),
                    if_break!(p, ")"),
                ]
            } else {
                argument.format(p)
            },
        );
    }

    if let Some(semi) = p.semi() {
        parts.push(semi);
    }
    Doc::Array(parts)
}
