use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::build_tools::{is_strict, py_schema_err};
use crate::errors::{ErrorType, ErrorTypeDefaults, ValError, ValResult};
use crate::input::{downcast_python_input, Input};
use crate::tools::SchemaDict;
use crate::validator::{BuildValidator, CombinedValidator, DefinitionsBuilder, Exactness, ValidationState, Validator};

use emval::{validate_email, EmailValidator as EmvalEmailValidator, ValidationError as EmvalValidationError};

#[derive(Debug, Clone)]
pub struct EmailValidator {
    strict: bool,
    allow_smtputf8: bool,
    allow_empty_local: bool,
    allow_quoted_local: bool,
    allow_domain_literal: bool,
    deliverable_address: bool,
    name: String,
}

impl BuildValidator for EmailValidator {
    const EXPECTED_TYPE: &'static str = "email";

    fn build(
        schema: &crate::tools::SchemaDict,
        config: Option<&crate::tools::SchemaDict>,
        _definitions: &mut DefinitionsBuilder<CombinedValidator>,
    ) -> PyResult<CombinedValidator> {
        // Extract configuration from the schema, similar to how `url.rs` does it
        Ok(Self {
            strict: is_strict(schema, config)?,
            allow_smtputf8: schema.get_as(intern!(schema.py(), "allow_smtputf8"))?.unwrap_or(false),
            allow_empty_local: schema
                .get_as(intern!(schema.py(), "allow_empty_local"))?
                .unwrap_or(false),
            allow_quoted_local: schema
                .get_as(intern!(schema.py(), "allow_quoted_local"))?
                .unwrap_or(false),
            allow_domain_literal: schema
                .get_as(intern!(schema.py(), "allow_domain_literal"))?
                .unwrap_or(false),
            deliverable_address: schema
                .get_as(intern!(schema.py(), "deliverable_address"))?
                .unwrap_or(false),
            name,
        }
        .into())
    }
}

impl Validator for EmailValidator {
    fn validate<'py>(
        &self,
        py: Python<'py>,
        input: &(impl Input<'py> + ?Sized),
        state: &mut ValidationState<'_, 'py>,
    ) -> ValResult<PyObject> {
        let email_str = match input.validate_str(state.strict_or(self.strict), false) {
            Ok(val_match) => {
                let either_str = val_match.into_inner();
                either_str.as_cow()?
            }
            Err(_) => {
                // If not a string, no fallback as with URLs (e.g. PyUrl), just return error
                return Err(ValError::new(ErrorTypeDefaults::StrType, input));
            }
        };

        let emval = EmvalEmailValidator {
            allow_smtputf8: self.allow_smtputf8,
            allow_empty_local: self.allow_empty_local,
            allow_quoted_local: self.allow_quoted_local,
            allow_domain_literal: self.allow_domain_literal,
            deliverable_address: self.deliverable_address,
        };

        match emval.validate_email(&email_str) {
            Ok(valid_email) => {
                // Lax rather than strict to preserve V2.4 semantics that str wins over email in union if needed
                state.floor_exactness(Exactness::Lax);

                // Return the normalized email as a Python string (or potentially a Python object)
                // Ok(valid_email.normalized.into_py(py))
                // return a validated email object
                Ok(valid_email.into_py(py))
            }
            Err(err) => Err(ValError::new(
                ErrorType::EmailParsing {
                    error: err.to_string(),
                    context: None,
                },
                input,
            )),
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}
