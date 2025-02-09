mod class_member;

use std::fmt::Debug;

use crate::{throw_unexpected_token_with_token, to_string_bracketed, tsx_keywords, Expression};
pub use class_member::*;
use iterator_endiate::EndiateIteratorExt;

use crate::{
	extensions::decorators::Decorated, visiting::Visitable, ASTNode, ExpressionOrStatementPosition,
	GenericTypeConstraint, Keyword, ParseOptions, ParseResult, Span, TSXKeyword, TSXToken,
	TypeAnnotation, VisitOptions,
};
use tokenizer_lib::{sized_tokens::TokenReaderWithTokenEnds, Token, TokenReader};

#[derive(Debug, Clone, PartialEq, Eq, get_field_by_type::GetFieldByType)]
#[get_field_by_type_target(Span)]
#[cfg_attr(feature = "self-rust-tokenize", derive(self_rust_tokenize::SelfRustTokenize))]
#[cfg_attr(feature = "serde-serialize", derive(serde::Serialize))]
pub struct ClassDeclaration<T: ExpressionOrStatementPosition> {
	pub class_keyword: Keyword<tsx_keywords::Class>,
	pub name: T,
	pub type_parameters: Option<Vec<GenericTypeConstraint>>,
	pub extends: Option<Box<Expression>>,
	pub implements: Option<Vec<TypeAnnotation>>,
	pub members: Vec<Decorated<ClassMember>>,
	pub position: Span,
}

impl<U: ExpressionOrStatementPosition + Debug + PartialEq + Eq + Clone + 'static> ASTNode
	for ClassDeclaration<U>
{
	fn from_reader(
		reader: &mut impl TokenReader<TSXToken, crate::TokenStart>,
		state: &mut crate::ParsingState,
		options: &ParseOptions,
	) -> ParseResult<Self> {
		let class_keyword = Keyword::from_reader(reader)?;
		Self::from_reader_sub_class_keyword(reader, state, options, class_keyword)
	}

	fn to_string_from_buffer<T: source_map::ToString>(
		&self,
		buf: &mut T,
		options: &crate::ToStringOptions,
		depth: u8,
	) {
		self.to_string_from_buffer(buf, options, depth);
	}

	fn get_position(&self) -> &Span {
		&self.position
	}
}

impl<U: ExpressionOrStatementPosition> ClassDeclaration<U> {
	pub(crate) fn from_reader_sub_class_keyword(
		reader: &mut impl TokenReader<TSXToken, crate::TokenStart>,
		state: &mut crate::ParsingState,
		options: &ParseOptions,
		class_keyword: Keyword<tsx_keywords::Class>,
	) -> ParseResult<Self> {
		let name = U::from_reader(reader, state, options)?;
		let type_parameters = reader
			.conditional_next(|token| *token == TSXToken::OpenChevron)
			.is_some()
			.then(|| {
				crate::parse_bracketed(reader, state, options, None, TSXToken::CloseChevron)
					.map(|(params, _)| params)
			})
			.transpose()?;

		let extends = match reader.peek() {
			Some(Token(TSXToken::Keyword(TSXKeyword::Extends), _)) => {
				reader.next();
				Some(Expression::from_reader(reader, state, options)?.into())
			}
			_ => None,
		};
		let implements = match reader.peek() {
			Some(Token(TSXToken::Keyword(TSXKeyword::Implements), _)) => {
				reader.next();
				let mut implements = Vec::new();
				loop {
					implements.push(TypeAnnotation::from_reader(reader, state, options)?);
					match reader.next().ok_or_else(crate::errors::parse_lexing_error)? {
						Token(TSXToken::Comma, _) => {}
						Token(TSXToken::OpenBrace, _pos) => break,
						token => {
							return throw_unexpected_token_with_token(
								token,
								&[TSXToken::OpenBrace, TSXToken::Comma],
							);
						}
					}
				}
				Some(implements)
			}
			_ => None,
		};
		reader.expect_next(TSXToken::OpenBrace)?;
		let mut members: Vec<Decorated<ClassMember>> = Vec::new();
		loop {
			if let Some(Token(TSXToken::CloseBrace, _)) = reader.peek() {
				break;
			}
			let value = Decorated::<ClassMember>::from_reader(reader, state, options)?;
			members.push(value);

			if let Some(Token(TSXToken::SemiColon, _)) = reader.peek() {
				reader.next();
			}
		}
		let position =
			class_keyword.get_position().union(reader.expect_next_get_end(TSXToken::CloseBrace)?);
		Ok(ClassDeclaration {
			class_keyword,
			name,
			type_parameters,
			extends,
			implements,
			members,
			position,
		})
	}

	pub(crate) fn to_string_from_buffer<T: source_map::ToString>(
		&self,
		buf: &mut T,
		options: &crate::ToStringOptions,
		depth: u8,
	) {
		buf.push_str("class ");
		if let Some(name) = self.name.as_option_str() {
			buf.push_str(name);
		}
		if let Some(type_parameters) = &self.type_parameters {
			to_string_bracketed(type_parameters, ('<', '>'), buf, options, depth);
		}
		if let Some(extends) = &self.extends {
			buf.push_str(" extends ");
			extends.to_string_from_buffer(buf, options, depth);
		}
		options.add_gap(buf);
		buf.push('{');
		for (at_end, member) in self.members.iter().endiate() {
			if options.pretty {
				buf.push_new_line();
				options.add_indent(depth + 1, buf);
			}
			member.to_string_from_buffer(buf, options, depth);
			if !options.pretty && !at_end {
				buf.push(';');
			}
		}
		if options.pretty && !self.members.is_empty() {
			buf.push_new_line();
		}
		buf.push('}');
	}
}

impl<T: ExpressionOrStatementPosition> Visitable for ClassDeclaration<T> {
	fn visit<TData>(
		&self,
		visitors: &mut (impl crate::VisitorReceiver<TData> + ?Sized),
		data: &mut TData,
		options: &VisitOptions,
		chain: &mut temporary_annex::Annex<crate::Chain>,
	) {
		visitors.visit_variable(
			&crate::visiting::ImmutableVariableOrProperty::ClassName(
				self.name.as_option_variable_identifier(),
			),
			data,
			chain,
		);
		self.extends.visit(visitors, data, options, chain);
		self.members.visit(visitors, data, options, chain);
	}

	fn visit_mut<TData>(
		&mut self,
		visitors: &mut (impl crate::VisitorMutReceiver<TData> + ?Sized),
		data: &mut TData,
		options: &VisitOptions,
		chain: &mut temporary_annex::Annex<crate::Chain>,
	) {
		visitors.visit_variable_mut(
			&mut crate::visiting::MutableVariableOrProperty::ClassName(
				self.name.as_option_variable_identifier_mut(),
			),
			data,
			chain,
		);
		self.extends.visit_mut(visitors, data, options, chain);
		self.members.visit_mut(visitors, data, options, chain);
	}
}
