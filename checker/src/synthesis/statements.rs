use super::{
	expressions::synthesise_multiple_expression, synthesise_block, variables::register_variable,
};
use crate::{
	behavior::iteration::{synthesise_iteration, IterationBehavior},
	context::Scope,
	diagnostics::TypeCheckError,
	synthesis::EznoParser,
	CheckingData, Environment, TypeId,
};

use parser::{expressions::MultipleExpression, ASTNode, BlockOrSingleStatement, Statement};
use std::collections::HashMap;

pub type ExportedItems = HashMap<String, crate::behavior::variables::VariableOrImport>;
pub type ReturnResult = Option<TypeId>;

pub struct StatementInformation {
	label: Option<String>,
}

pub(super) fn synthesise_statement<T: crate::ReadFromFS>(
	statement: &Statement,
	information: Option<StatementInformation>,
	environment: &mut Environment,
	checking_data: &mut CheckingData<T, super::EznoParser>,
) {
	match statement {
		Statement::Expression(expression) => {
			synthesise_multiple_expression(
				expression,
				environment,
				checking_data,
				TypeId::ANY_TYPE,
			);
		}
		Statement::Return(return_statement) => {
			let returned = if let Some(ref expression) = return_statement.1 {
				// TODO expecting based of expected return type
				synthesise_multiple_expression(
					expression,
					environment,
					checking_data,
					TypeId::ANY_TYPE,
				)
			} else {
				TypeId::UNDEFINED_TYPE
			};

			let position = return_statement.2.with_source(environment.get_source());

			environment.return_value(returned, position);
		}
		Statement::If(if_statement) => {
			fn run_condition<T: crate::ReadFromFS>(
				current: (&MultipleExpression, &BlockOrSingleStatement),
				others: &[(&MultipleExpression, &BlockOrSingleStatement)],
				last: Option<&BlockOrSingleStatement>,
				environment: &mut Environment,
				checking_data: &mut CheckingData<T, super::EznoParser>,
			) {
				let condition = synthesise_multiple_expression(
					current.0,
					environment,
					checking_data,
					TypeId::ANY_TYPE,
				);

				environment.new_conditional_context(
					condition,
					|env: &mut Environment, data: &mut CheckingData<T, EznoParser>| {
						synthesise_block_or_single_statement(current.1, env, data);
					},
					if !others.is_empty() || last.is_some() {
						Some(|env: &mut Environment, data: &mut CheckingData<T, EznoParser>| {
							if let [current, others @ ..] = &others {
								run_condition(*current, others, last, env, data);
							} else {
								synthesise_block_or_single_statement(last.unwrap(), env, data);
							}
						})
					} else {
						None
					},
					checking_data,
				);
			}

			let others = if_statement
				.else_conditions
				.iter()
				.map(|cond| (&cond.condition, &cond.inner))
				.collect::<Vec<_>>();

			let last = if_statement.trailing_else.as_ref().map(|b| &b.inner);

			run_condition(
				(&if_statement.condition, &if_statement.inner),
				others.as_slice(),
				last,
				environment,
				checking_data,
			);
		}
		Statement::Switch(stmt) => {
			checking_data.diagnostics_container.add_error(TypeCheckError::Unsupported {
				thing: "Switch statement",
				at: stmt.get_position().with_source(environment.get_source()),
			});
		}
		Statement::WhileLoop(stmt) => synthesise_iteration(
			IterationBehavior::While(&stmt.condition),
			information.and_then(|info| info.label),
			environment,
			checking_data,
			|environment, checking_data| {
				synthesise_block_or_single_statement(&stmt.inner, environment, checking_data);
			},
		),
		Statement::DoWhileLoop(stmt) => synthesise_iteration(
			IterationBehavior::DoWhile(&stmt.condition),
			information.and_then(|info| info.label),
			environment,
			checking_data,
			|environment, checking_data| {
				synthesise_block_or_single_statement(&stmt.inner, environment, checking_data);
			},
		),
		Statement::ForLoop(stmt) => match &stmt.condition {
			parser::statements::ForLoopCondition::ForOf {
				keyword: _,
				variable,
				of,
				position: _,
			} => {
				synthesise_iteration(
					IterationBehavior::ForOf { lhs: variable.get_ast_ref(), rhs: of },
					information.and_then(|info| info.label),
					environment,
					checking_data,
					|environment, checking_data| {
						synthesise_block_or_single_statement(
							&stmt.inner,
							environment,
							checking_data,
						);
					},
				);
			}
			parser::statements::ForLoopCondition::ForIn {
				keyword: _,
				variable,
				r#in,
				position: _,
			} => {
				synthesise_iteration(
					IterationBehavior::ForIn { lhs: variable.get_ast_ref(), rhs: r#in },
					information.and_then(|info| info.label),
					environment,
					checking_data,
					|environment, checking_data| {
						synthesise_block_or_single_statement(
							&stmt.inner,
							environment,
							checking_data,
						);
					},
				);
			}
			parser::statements::ForLoopCondition::Statements {
				initialiser,
				condition,
				afterthought,
				position: _,
			} => synthesise_iteration(
				IterationBehavior::For { initialiser, condition, afterthought },
				information.and_then(|info| info.label),
				environment,
				checking_data,
				|environment, checking_data| {
					synthesise_block_or_single_statement(&stmt.inner, environment, checking_data);
				},
			),
		},
		Statement::Block(ref block) => {
			let (_result, _, _) = environment.new_lexical_environment_fold_into_parent(
				Scope::Block {},
				checking_data,
				|environment, checking_data| synthesise_block(&block.0, environment, checking_data),
			);
		}
		Statement::Cursor(_cursor_id, _) => {
			todo!("Dump environment data somewhere")
		}
		Statement::Continue(label, position) => {
			if let Err(err) = environment.add_continue(label.as_deref(), *position) {
				checking_data
					.diagnostics_container
					.add_error(TypeCheckError::NotInLoopOrCouldNotFindLabel(err));
			}
		}
		Statement::Break(label, position) => {
			if let Err(err) = environment.add_break(label.as_deref(), *position) {
				checking_data
					.diagnostics_container
					.add_error(TypeCheckError::NotInLoopOrCouldNotFindLabel(err));
			}
		}
		Statement::Throw(stmt) => {
			let thrown_value = synthesise_multiple_expression(
				&stmt.1,
				environment,
				checking_data,
				TypeId::ANY_TYPE,
			);
			let thrown_position = stmt.2.with_source(environment.get_source());
			environment.throw_value(thrown_value, thrown_position);
		}
		Statement::Labelled { position: _, name, statement } => {
			// Labels on invalid statements is caught at parse time

			synthesise_statement(
				statement,
				Some(StatementInformation { label: Some(name.clone()) }),
				environment,
				checking_data,
			);
		}
		Statement::VarVariable(_) => {
			checking_data.raise_unimplemented_error(
				"var variables statements",
				statement.get_position().with_source(environment.get_source()),
			);
		}
		Statement::TryCatch(stmt) => {
			let throw_type: TypeId =
				environment.new_try_context(checking_data, |environment, checking_data| {
					synthesise_block(&stmt.try_inner.0, environment, checking_data);
				});

			if let Some(ref catch_block) = stmt.catch_inner {
				// TODO catch when never
				environment.new_lexical_environment_fold_into_parent(
					crate::Scope::Block {},
					checking_data,
					|environment, checking_data| {
						if let Some((clause, _type)) = &stmt.exception_var {
							// TODO clause.type_annotation
							register_variable(
								clause.get_ast_ref(),
								environment,
								checking_data,
								crate::context::VariableRegisterBehavior::CatchVariable {
									ty: throw_type,
								},
								// TODO
								None,
							);
						}
						synthesise_block(&catch_block.0, environment, checking_data);
					},
				);
			}
		}
		// TODO do these higher up in the block. To set relevant information
		Statement::Comment(s, _) if s.starts_with("@ts") => {
			crate::utils::notify!("acknowledge '@ts-ignore' and other comments");
		}
		Statement::MultiLineComment(s, _) if s.starts_with('*') => {
			crate::utils::notify!("acknowledge '@ts-ignore' and other comments");
		}
		Statement::Comment(..)
		| Statement::MultiLineComment(..)
		| Statement::Debugger(_)
		| Statement::Empty(_) => {}
	}
}

/// Expects that this caller has already create a context for this to run in
fn synthesise_block_or_single_statement<T: crate::ReadFromFS>(
	block_or_single_statement: &BlockOrSingleStatement,
	environment: &mut Environment,
	checking_data: &mut CheckingData<T, super::EznoParser>,
) {
	match block_or_single_statement {
		BlockOrSingleStatement::Braced(block) => {
			synthesise_block(&block.0, environment, checking_data);
		}
		BlockOrSingleStatement::SingleStatement(statement) => {
			synthesise_statement(statement, None, environment, checking_data);
		}
	}
}
