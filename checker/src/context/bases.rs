//! TODO work in progress
use super::ContextId;
use crate::TypeId;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub struct Boundary(pub(crate) ContextId);

/// Contains the constraint / bases of dynamic poly types
///
/// TODO generic as only environments should have mutable bases
#[derive(Default, Debug)]
pub struct Bases {
	pub(crate) immutable_bases: HashMap<TypeId, TypeId>,
	pub(crate) mutable_bases: HashMap<TypeId, (Boundary, TypeId)>,
}

impl Bases {
	pub(crate) fn _does_type_have_mutable_base(&self, _on: TypeId) -> bool {
		todo!()
	}

	pub(crate) fn merge(&mut self, bases: Bases, context_id: ContextId) {
		self.immutable_bases.extend(bases.immutable_bases);
		for (ty, (ctx_ceil, base)) in bases.mutable_bases {
			let existing = if ctx_ceil.0 == context_id {
				self.immutable_bases.insert(ty, base).is_some()
			} else {
				self.mutable_bases.insert(ty, (ctx_ceil, base)).is_some()
			};
			if existing {
				crate::utils::notify!("Found existing constraint, should be safe to override");
			}
		}
	}

	/// INTERFACE extends HAPPEN AFTER THE TYPE HAS BEEN CRATED
	#[allow(unused)]
	pub fn connect_extends(&mut self, on: TypeId, ty: TypeId) {
		let res = self.immutable_bases.insert(on, ty);
		debug_assert!(res.is_none());
	}

	// pub(crate) fn get_local_type_base(&self, ty: TypeId) -> Option<TypeId> {
	// 	self.mutable_bases.get(&ty).map(|b| b.1).or_else(|| self.immutable_bases.get(&ty).copied())
	// }
}
