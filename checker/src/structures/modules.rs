use std::{collections::HashMap, path::PathBuf};

use derive_enum_from_into::EnumFrom;

use crate::{Diagnostic, Variable};

pub struct SynthesisedModule<M> {
	pub content: M,
	// TODO this should not be on unchecked module
	// TODO export default
	pub(crate) exported_variables: HashMap<String, Variable>,
}

#[derive(Debug, EnumFrom)]
pub enum ModuleFromPathError {
	// ParseError
	ParseError(()),
	PathDoesNotExist(PathBuf),
	NoResolverForExtension(String),
}

impl From<ModuleFromPathError> for Diagnostic {
	fn from(err: ModuleFromPathError) -> Self {
		todo!()
	}
}

impl<U> SynthesisedModule<U> {
	pub fn get_exports(&self) -> &HashMap<String, Variable> {
		&self.exported_variables
	}
}
