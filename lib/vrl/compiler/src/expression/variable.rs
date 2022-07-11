use std::fmt;

use diagnostic::{DiagnosticMessage, Label};
use value::Value;

use crate::{
    expression::{levenstein, Resolved},
    parser::ast::Ident,
    state::{ExternalEnv, LocalEnv},
    BatchContext, Context, Expression, Span, TypeDef,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variable {
    ident: Ident,
    value: Option<Value>,
}

impl Variable {
    pub(crate) fn new(span: Span, ident: Ident, local: &LocalEnv) -> Result<Self, Error> {
        let value = match local.variable(&ident) {
            Some(variable) => variable.value.as_ref().cloned(),
            None => {
                let idents = local
                    .variable_idents()
                    .map(std::clone::Clone::clone)
                    .collect::<Vec<_>>();

                return Err(Error::undefined(ident, span, idents));
            }
        };

        Ok(Self { ident, value })
    }

    pub fn ident(&self) -> &Ident {
        &self.ident
    }

    pub fn value(&self) -> Option<&Value> {
        self.value.as_ref()
    }
}

impl Expression for Variable {
    fn resolve(&self, ctx: &mut Context) -> Resolved {
        Ok(ctx
            .state()
            .variable(&self.ident)
            .cloned()
            .unwrap_or(Value::Null))
    }

    fn resolve_batch(&mut self, ctx: &mut BatchContext, selection_vector: &[usize]) {
        for index in selection_vector {
            let index = *index;
            ctx.resolved_values[index] = Ok(ctx.states[index]
                .variable(&self.ident)
                .cloned()
                .unwrap_or(Value::Null));
        }
    }

    fn type_def(&self, (local, _): (&LocalEnv, &ExternalEnv)) -> TypeDef {
        local
            .variable(&self.ident)
            .cloned()
            .map_or_else(|| TypeDef::null().infallible(), |d| d.type_def)
    }
}

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.ident.fmt(f)
    }
}

#[derive(Debug)]
pub(crate) struct Error {
    variant: ErrorVariant,
    ident: Ident,
    span: Span,
}

impl Error {
    fn undefined(ident: Ident, span: Span, idents: Vec<Ident>) -> Self {
        Error {
            variant: ErrorVariant::Undefined { idents },
            ident,
            span,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum ErrorVariant {
    #[error("call to undefined variable")]
    Undefined { idents: Vec<Ident> },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#}", self.variant)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.variant)
    }
}

impl DiagnosticMessage for Error {
    fn code(&self) -> usize {
        use ErrorVariant::Undefined;

        match &self.variant {
            Undefined { .. } => 701,
        }
    }

    fn labels(&self) -> Vec<Label> {
        use ErrorVariant::Undefined;

        match &self.variant {
            Undefined { idents } => {
                let mut vec = vec![Label::primary("undefined variable", self.span)];
                let ident_chars = self.ident.as_ref().chars().collect::<Vec<_>>();

                let mut builtin = vec![Ident::new("null"), Ident::new("true"), Ident::new("false")];
                let mut idents = idents.clone();

                idents.append(&mut builtin);

                if let Some((idx, _)) = idents
                    .iter()
                    .map(|possible| {
                        let possible_chars = possible.chars().collect::<Vec<_>>();
                        levenstein::distance(&ident_chars, &possible_chars)
                    })
                    .enumerate()
                    .min_by_key(|(_, score)| *score)
                {
                    {
                        let guessed = &idents[idx];
                        vec.push(Label::context(
                            format!(r#"did you mean "{}"?"#, guessed),
                            self.span,
                        ));
                    }
                }

                vec
            }
        }
    }
}
