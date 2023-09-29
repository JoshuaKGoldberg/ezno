use parser::{visiting::StatementOrDeclarationMut, Declaration};

pub struct ConstToLet;

// TODO different data
impl parser::visiting::VisitorMut<StatementOrDeclarationMut<'_>, ()> for ConstToLet {
	fn visit_mut(
		&mut self,
		item: &mut StatementOrDeclarationMut,
		_data: &mut (),
		_chain: &parser::visiting::Chain,
	) {
		if let StatementOrDeclarationMut::Declaration(Declaration::Variable(decl)) = item {
			if let parser::declarations::VariableDeclaration::ConstDeclaration {
				keyword,
				declarations,
				position,
			} = decl
			{
				*decl = parser::declarations::VariableDeclaration::LetDeclaration {
					keyword: parser::Keyword::new(keyword.get_position().clone()),
					declarations: declarations
						.drain(..)
						.map(|dec| parser::declarations::VariableDeclarationItem {
							name: dec.name,
							type_annotation: dec.type_annotation,
							expression: Some(dec.expression),
							position: dec.position,
						})
						.collect(),
					position: position.clone(),
				};
			}
		}
	}
}
