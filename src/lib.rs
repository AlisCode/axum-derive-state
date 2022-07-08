use std::collections::HashSet;

use ident_case::RenameRule;
use proc_macro2::{Span, TokenStream};
use proc_macro_error::proc_macro_error;
use quote::{quote, ToTokens};
use syn::{spanned::Spanned, Fields, Ident, ItemStruct, LitInt, Type};

#[proc_macro_error]
#[proc_macro_derive(State)]
pub fn derive_state_attr(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = TokenStream::from(input);
    let extracted_fields = parse_axum_state_macro_input(input.clone());
    match extracted_fields {
        Ok(input) => {
            let output = generate_from_impls(input);
            return proc_macro::TokenStream::from(output);
        }
        Err(err) => match err {
            ExtractFieldError::UnsupportedVariant(
                AxumStateMacroUnsupportedVariant::ExpectedStruct,
            ) => {
                // TODO: We can support enum with a bit more work. Define if we want to support that or
                // not.
                proc_macro_error::abort!(input, "Deriving State is only supported on structures");
            }
            ExtractFieldError::UnsupportedVariant(AxumStateMacroUnsupportedVariant::UnitStruct) => {
                // Actually, the state can be empty, but then using the derive macro is useless.
                proc_macro_error::abort!(input, "The state of your app may never be empty");
            }
            ExtractFieldError::DuplicatedField {
                r#type,
                failing_ident,
                span,
            } => {
                let duplicated_type = r#type.to_token_stream();
                let help_text = match failing_ident {
                    FailingFieldIdent::Named(ident) => {
                        let suggested_ident =
                            RenameRule::PascalCase.apply_to_field(ident.to_string());
                        format!(
                            "Consider wrapping this type in a newtype, e.g. pub struct {}({});",
                            suggested_ident, duplicated_type
                        )
                    }
                    FailingFieldIdent::Anonymous => format!(
                        "Consider wrapping this type in a newtype, e.g. pub struct MySubstate({});",
                        duplicated_type
                    ),
                };
                proc_macro_error::abort!(
                    span,
                    "Type {} is used multiple times in the state",
                    duplicated_type;
                    note = "The state may only contain exactly one instance of a given type";
                    help = help_text;
                )
            }
        },
    };
}

#[derive(Debug)]
enum FailingFieldIdent {
    Named(Ident),
    Anonymous,
}

#[derive(Debug)]
struct AxumStateMacroInput {
    fields: Vec<FieldDefinition>,
    ident: Ident,
}

#[derive(Debug, Clone)]
struct FieldDefinition {
    name: IdentOrTupleMember,
    r#type: Type,
}

#[derive(Debug, Clone)]
enum IdentOrTupleMember {
    Ident(Ident),
    TupleMember(LitInt),
}

impl ToTokens for IdentOrTupleMember {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            IdentOrTupleMember::Ident(ident) => ident.to_tokens(tokens),
            IdentOrTupleMember::TupleMember(lit) => lit.to_tokens(tokens),
        }
    }
}

#[derive(Debug)]
enum AxumStateMacroUnsupportedVariant {
    UnitStruct,
    ExpectedStruct,
}

#[derive(Debug)]
enum ExtractFieldError {
    UnsupportedVariant(AxumStateMacroUnsupportedVariant),
    DuplicatedField {
        r#type: Type,
        failing_ident: FailingFieldIdent,
        span: Span,
    },
}

fn parse_axum_state_macro_input(
    input: TokenStream,
) -> Result<AxumStateMacroInput, ExtractFieldError> {
    let parsed = syn::parse2::<ItemStruct>(input).map_err(|_| {
        ExtractFieldError::UnsupportedVariant(AxumStateMacroUnsupportedVariant::ExpectedStruct)
    })?;
    let mut fields_types = HashSet::<Type>::default();
    let fields = match parsed.fields {
        Fields::Named(named) => named
            .named
            .into_iter()
            .map(|field| {
                let field_def = FieldDefinition {
                    name: IdentOrTupleMember::Ident(field.ident.clone().unwrap()),
                    r#type: field.ty.clone(),
                };
                fields_types
                    .insert(field_def.r#type.clone())
                    .then_some(field_def.clone())
                    .ok_or_else(|| ExtractFieldError::DuplicatedField {
                        r#type: field_def.r#type.clone(),
                        span: field.span(),
                        failing_ident: FailingFieldIdent::Named(field.ident.unwrap()),
                    })
            })
            .collect::<Result<Vec<_>, _>>(),
        Fields::Unnamed(unnamed) => unnamed
            .unnamed
            .into_iter()
            .enumerate()
            .map(|(index, field)| {
                let field_def = FieldDefinition {
                    name: IdentOrTupleMember::TupleMember(LitInt::new(
                        &index.to_string(),
                        field.ty.span(),
                    )),
                    r#type: field.ty.clone(),
                };
                fields_types
                    .insert(field_def.r#type.clone())
                    .then_some(field_def.clone())
                    .ok_or_else(|| ExtractFieldError::DuplicatedField {
                        r#type: field_def.r#type.clone(),
                        span: field.span(),
                        failing_ident: FailingFieldIdent::Anonymous,
                    })
            })
            .collect::<Result<Vec<_>, _>>(),
        Fields::Unit => Err(ExtractFieldError::UnsupportedVariant(
            AxumStateMacroUnsupportedVariant::UnitStruct,
        )),
    }?;
    Ok(AxumStateMacroInput {
        fields,
        ident: parsed.ident,
    })
}

fn generate_from_impls(input: AxumStateMacroInput) -> TokenStream {
    let ident = input.ident;
    input
        .fields
        .into_iter()
        .map(|field| {
            let field_name = field.name;
            let field_type = field.r#type;
            quote!(
                impl From<#ident> for #field_type {
                    fn from(state: #ident) -> #field_type {
                        state.#field_name
                    }
                }
            )
        })
        .collect()
}

#[cfg(test)]
pub mod tests {

    #[test]
    fn trybuild_tests() {
        let tests = trybuild::TestCases::new();
        tests.compile_fail("tests/failing/*.rs");
        tests.pass("tests/compiling/*.rs");
    }
}
