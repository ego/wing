use std::fmt::{Debug, Display};
use std::ops::Deref;

use crate::ast::{Type as AstType, *};
use crate::type_env::TypeEnv;

#[derive(Debug)]
pub enum Type {
	Anything,
	Nothing,
	Number,
	String,
	Duration,
	Boolean,
	Function(Box<FunctionSignature>),
	Class(Class),
	ClassInstance(*const Class),
}

pub struct Class {
	name: Symbol, // TODO: do we really need the name here, we should alway get here through a Class Type in some env which has the name of the type
	env: Box<TypeEnv>,
	parent: Option<TypeRef>, // Must be a Type::Class type
}

impl Debug for Class {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Class")
			.field("name", &self.name)
			.field("parent", &self.parent)
			.finish()
	}
}

impl PartialEq for Type {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Function(l0), Self::Function(r0)) => l0 == r0,
			(Self::Class(l0), Self::Class(r0)) => {
				// If our parent is equal to `other` then treat both classes as equal (inheritance)
				if let Some(parent) = l0.parent {
					let parent_type: &Type = parent.into();
					return parent_type.eq(other);
				}
				false
			}
			(Self::ClassInstance(l0), Self::ClassInstance(r0)) => l0 == r0,
			_ => core::mem::discriminant(self) == core::mem::discriminant(other),
		}
	}
}

#[derive(PartialEq, Debug)]
pub struct FunctionSignature {
	pub args: Vec<TypeRef>,
	pub return_type: Option<TypeRef>,
}

impl Display for Type {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Type::Anything => write!(f, "anything"),
			Type::Nothing => write!(f, "nothing"),
			Type::Number => write!(f, "number"),
			Type::String => write!(f, "string"),
			Type::Duration => write!(f, "duration"),
			Type::Boolean => write!(f, "bool"),
			Type::Function(func_sig) => {
				if let Some(ret_val) = &func_sig.return_type {
					write!(
						f,
						"fn({}) -> {}",
						func_sig
							.args
							.iter()
							.map(|a| format!("{}", a))
							.collect::<Vec<String>>()
							.join(", "),
						format!("{}", ret_val)
					)
				} else {
					write!(
						f,
						"fn({})",
						func_sig
							.args
							.iter()
							.map(|a| format!("{}", a))
							.collect::<Vec<String>>()
							.join(",")
					)
				}
			}
			Type::Class(_) => todo!(),
			Type::ClassInstance(_) => todo!(),
		}
	}
}

//type TypeRef = *const Type;

#[derive(Clone, Copy)]
pub struct TypeRef(*const Type);

impl From<&Box<Type>> for TypeRef {
	fn from(t: &Box<Type>) -> Self {
		Self(&**t as *const Type)
	}
}

impl From<TypeRef> for &Type {
	fn from(t: TypeRef) -> Self {
		unsafe { &*t.0 }
	}
}

// impl Deref for TypeRef {
//     type Target = Type;

//     fn deref(&self) -> &Self::Target {
//         (*self).into()
//     }
// }

impl Display for TypeRef {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let t: &Type = (*self).into();
		write!(f, "{}", t)
	}
}

impl PartialEq for TypeRef {
	fn eq(&self, other: &Self) -> bool {
		// Types are equal if they point to the same type definition
		if self.0 == other.0 {
			true
		} else {
			// If the self and other aren't the the same, we need to use the specific types equality function
			let t1: &Type = (*self).into();
			let t2: &Type = (*other).into();
			t1.eq(t2) // Same as `t1 == t2`, used eq for verbosity
		}
	}
}

impl Debug for TypeRef {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self.0)
	}
}

struct Types {
	// TODO: Remove the box and change TypeRef to just be an index into the types array
	// Note: we need the box so reallocations of the vec while growing won't change the addresses of the types since they are referenced from the TypeRef struct
	types: Vec<Box<Type>>,
	numeric_idx: usize,
	string_idx: usize,
	bool_idx: usize,
	duration_idx: usize,
}

impl Types {
	pub fn new() -> Self {
		let mut types = vec![];
		types.push(Box::new(Type::Number));
		let numeric_idx = types.len() - 1;
		types.push(Box::new(Type::String));
		let string_idx = types.len() - 1;
		types.push(Box::new(Type::Boolean));
		let bool_idx = types.len() - 1;
		types.push(Box::new(Type::Duration));
		let duration_idx = types.len() - 1;

		Self {
			types,
			numeric_idx,
			string_idx,
			bool_idx,
			duration_idx,
		}
	}

	pub fn number(&self) -> TypeRef {
		(&self.types[self.numeric_idx]).into()
	}

	pub fn string(&self) -> TypeRef {
		(&self.types[self.string_idx]).into()
	}

	pub fn bool(&self) -> TypeRef {
		(&self.types[self.bool_idx]).into()
	}

	pub fn duration(&self) -> TypeRef {
		(&self.types[self.duration_idx]).into()
	}

	pub fn add_type(&mut self, t: Type) -> TypeRef {
		self.types.push(Box::new(t));
		(&self.types[self.types.len() - 1]).into()
	}
}

pub struct TypeChecker {
	types: Types,
}

impl TypeChecker {
	pub fn new() -> Self {
		Self { types: Types::new() }
	}

	pub fn get_primitive_type_by_name(&self, name: &str) -> TypeRef {
		match name {
			"number" => self.types.number(),
			"string" => self.types.string(),
			"bool" => self.types.bool(),
			"duration" => self.types.duration(),
			other => panic!("Type {} is not a primitive type", other),
		}
	}

	pub fn type_check_exp(&self, exp: &Expression, env: &TypeEnv) -> Option<TypeRef> {
		match exp {
			Expression::Literal(lit) => match lit {
				Literal::String(_) => Some(self.types.string()),
				Literal::Number(_) => Some(self.types.number()),
				Literal::Duration(_) => Some(self.types.duration()),
				Literal::Boolean(_) => Some(self.types.bool()),
			},
			Expression::Binary { op, lexp, rexp } => {
				let ltype = self.type_check_exp(lexp, env).unwrap();
				let rtype = self.type_check_exp(rexp, env).unwrap();
				self.validate_type(ltype, rtype, rexp);
				if op.boolean_args() {
					self.validate_type(ltype, self.types.bool(), rexp);
				} else if op.numerical_args() {
					self.validate_type(ltype, self.types.number(), rexp);
				}

				if op.boolean_result() {
					Some(self.types.bool())
				} else {
					self.validate_type(ltype, self.types.number(), rexp);
					Some(ltype)
				}
			}
			Expression::Unary { op: _, exp: unary_exp } => {
				let _type = self.type_check_exp(&unary_exp, env).unwrap();
				// Add bool vs num support here (! => bool, +- => num)
				self.validate_type(_type, self.types.number(), &unary_exp);
				Some(_type)
			}
			Expression::Reference(_ref) => Some(env.lookup(&_ref.identifier)),
			Expression::New {
				class: _,
				obj_id: _,
				arg_list: _,
			} => todo!(),
			Expression::FunctionCall { function, args } => {
				let func_type = env.lookup(&function.identifier);

				if let &Type::Function(ref func_type) = func_type.into() {
					// TODO: named args
					// Argument arity check
					if args.pos_args.len() != func_type.args.len() {
						panic!(
							"Expected {} arguments for function {}, but got {} instead.",
							func_type.args.len(),
							function.identifier,
							args.pos_args.len()
						)
					}
					// Argument type check
					for (passed_arg, expected_arg) in args.pos_args.iter().zip(func_type.args.iter()) {
						let passed_arg_type = self.type_check_exp(passed_arg, env).unwrap();
						self.validate_type(passed_arg_type, *expected_arg, passed_arg);
					}

					func_type.return_type
				} else {
					panic!("Identifier {} is not a function", function.identifier)
				}
			}
			Expression::MethodCall(_) => todo!(),
			Expression::CapturedObjMethodCall(_) => todo!(),
		}
	}

	fn validate_type(&self, actual_type: TypeRef, expected_type: TypeRef, value: &Expression) {
		if actual_type != expected_type {
			println!("{}", self.types.number());
			println!("{}", actual_type);
			panic!("Expected type {} of {:?} to be {}", actual_type, value, expected_type);
		}
	}

	pub fn type_check_scope(&mut self, scope: &Scope, env: &mut TypeEnv) {
		for statement in scope.statements.iter() {
			self.type_check_statement(statement, env);
		}
	}

	fn resolve_type(&mut self, ast_type: &AstType, env: &TypeEnv) -> TypeRef {
		match ast_type {
			AstType::Number => self.types.number(),
			AstType::String => self.types.string(),
			AstType::Bool => self.types.bool(),
			AstType::Duration => self.types.duration(),
			AstType::FunctionSignature(ast_sig) => {
				let mut args = vec![];
				for arg in ast_sig.parameters.iter() {
					args.push(self.resolve_type(arg, env));
				}
				let sig = FunctionSignature {
					args,
					return_type: ast_sig.return_type.as_ref().map(|t| self.resolve_type(t, env)),
				};
				// TODO: avoid creating a new type for each function_sig resolution
				self.types.add_type(Type::Function(Box::new(sig)))
			}
			AstType::Class(class_name) => {
				// Lookup this class name in the current environment
				env.lookup(&class_name)
			}
		}
	}

	fn type_check_statement(&mut self, statement: &Statement, env: &mut TypeEnv) {
		match statement {
			Statement::VariableDef {
				var_name,
				initial_value,
			} => {
				let exp_type = self.type_check_exp(initial_value, env).unwrap();
				env.define(&var_name, exp_type);
			}
			Statement::FunctionDefinition(func_def) => {
				// TODO: make sure this function returns on all control paths when there's a return type (can be done by recursively traversing the statements and making sure there's a "return" statements in all control paths)

				// Create a type_checker function signature from the AST function definition, assuming success we can add this function to the env
				// TODO: why not just use `self.resolve_type(&AstType::FunctionSignature(sig), env);`??
				let mut args = vec![];
				let ast_sig = &func_def.signature;
				for arg in ast_sig.parameters.iter() {
					args.push(self.resolve_type(arg, env));
				}
				let return_type = ast_sig.return_type.as_ref().map(|t| self.resolve_type(&t, env));
				let function_type = self.types.add_type(Type::Function(Box::new(FunctionSignature {
					args: args.clone(),
					return_type,
				})));

				// Add this function to the env
				env.define(&func_def.name, function_type);

				let mut function_env = TypeEnv::new(Some(env), return_type);
				for (param, param_type) in func_def.parameters.iter().zip(args.iter()) {
					function_env.define(&param, *param_type);
				}
				// TODO: we created `function_env` but `type_check_scope` will also create a wrapper env for the scope which is redundant
				self.type_check_scope(&func_def.statements, &mut function_env);
			}
			Statement::InflightFunctionDefinition {
				name: _,
				parameters: _,
				statements: _,
			} => todo!(),
			Statement::ForLoop {
				iterator,
				iterable,
				statements,
			} => {
				// TODO: Expression must be iterable
				let exp_type = self.type_check_exp(iterable, env).unwrap();

				let mut scope_env = TypeEnv::new(Some(env), env.return_type);
				scope_env.define(&iterator, exp_type);

				self.type_check_scope(statements, &mut scope_env);
			}
			Statement::If {
				condition,
				statements,
				else_statements,
			} => {
				let cond_type = self.type_check_exp(condition, env).unwrap();
				self.validate_type(cond_type, self.types.bool(), condition);

				let mut scope_env = TypeEnv::new(Some(env), env.return_type);
				self.type_check_scope(statements, &mut scope_env);

				if let Some(else_scope) = else_statements {
					let mut else_scope_env = TypeEnv::new(Some(env), env.return_type);
					self.type_check_scope(else_scope, &mut else_scope_env);
				}
			}
			Statement::Expression(e) => {
				self.type_check_exp(e, env);
			}
			Statement::Assignment { variable, value } => {
				let exp_type = self.type_check_exp(value, env).unwrap();
				self.validate_type(exp_type, env.lookup(&variable.identifier), value);
			}
			Statement::Use {
				module_name: _,
				identifier: _,
			} => todo!(),
			Statement::Scope(scope) => {
				let mut scope_env = TypeEnv::new(Some(env), env.return_type);
				for statement in scope.statements.iter() {
					self.type_check_statement(statement, &mut scope_env);
				}
			}
			Statement::Return(exp) => {
				if let Some(return_expression) = exp {
					let return_type = self.type_check_exp(return_expression, env).unwrap();
					if let Some(expected_return_type) = env.return_type {
						self.validate_type(return_type, expected_return_type, return_expression);
					} else {
						panic!("Return statement outside of function cannot return a value.");
					}
				} else {
					if let Some(expected_return_type) = env.return_type {
						panic!("Expected return statement to return type {}", expected_return_type);
					}
				}
			}
			Statement::Class {
				name,
				members,
				methods,
				parent,
				constructor,
			} => {
				// Verify parent is actually a known Class
				let parent_class = if let Some(parent_symbol) = parent {
					let t = env.lookup(parent_symbol);
					if let &Type::Class(ref _class) = t.into() {
						Some(t)
					} else {
						panic!("Class {}'s parent {} is not a class", name, parent_symbol);
					}
				} else {
					None
				};

				// Add myself to the current environment (so class implementation can reference itself)
				// self.types.add_type(Type::Class(Class{ name, env: todo!(), parent: todo!() }))
				// env.define(name, type)

				// Create environment representing this class
				let mut class_env = TypeEnv::new(Some(env), None);

				// Add members to the class env
				// TODO: we need to somehow add ourselves to the current env before adding all class members to the class env, so if there's a member of
				//   my own type it'll resolve correctly. Currently `class A { a: A }` will fail or resolve `a` to an outer `A`. (same for methods below).
				for member in members.iter() {
					let member_type = self.resolve_type(&member.member_type, env);
					class_env.define(&member.name, member_type);
				}
				// Add methods to the class env
				for method in methods.iter() {
					let mut sig = method.signature.clone();

					// Add myself as first parameter to all class methods (self)
					sig.parameters.insert(0, AstType::Class(name.clone())); // TODO: this'll probably fail us since we don't have ourselves in the env yet..

					let method_type = self.resolve_type(&AstType::FunctionSignature(sig), env);
					class_env.define(&method.name, method_type);
				}

				// Type check methods
				for method in methods.iter() {
					// Lookup the method in the class_env
					let method_type = class_env.lookup(&method.name);
					let method_sig = if let &Type::Function(ref s) = method_type.into() {
						s
					} else {
						panic!(
							"Method {}.{} isn't defined as a function in the class environment",
							name, method.name
						)
					};

					// Create method environment and prime it with args
					let mut method_env = TypeEnv::new(Some(env), method_sig.return_type);
					// Add `this` as first argument
					let mut actual_parameters = vec![Symbol {
						name: "this".into(),
						span: method.name.span.clone(),
					}];
					actual_parameters.extend(method.parameters.clone());
					for (param, param_type) in actual_parameters.iter().zip(method_sig.args.iter()) {
						method_env.define(&param, *param_type);
					}
					// Check function scope
					self.type_check_scope(&method.statements, &mut method_env);
				}

				// Define a new class type and add it to the environment
				let t = self.types.add_type(Type::Class(Class {
					name: name.clone(),
					env: Box::new(class_env),
					parent: parent_class,
				}));
				env.define(name, t)
			}
		}
	}
}
