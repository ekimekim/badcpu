use std::collections::HashMap;


const MAX_EXPANSION_DEPTH: usize = 1024;


// State needed on a line-to-line basis in assembler
struct Assembler {
	bank_number: u8,
	// None if bank is not yet written to
	banks: [Option<Box<Bank>>; 256],
	// Maps symbol name to symbol
	symbols: HashMap<String, Symbol>,
}

struct Bank {
	number: u8,
	position: u8,
	data: [u8; 256],
}

// Symbols get replaced by their result. They may take arguments.
// Replacements happen at the AST layer, not strings.
// Symbols are expanded outer first (ie. symbols in a macro are only expanded after the macro itself),
// with macro args expanding before the macro is.
// eg. f(g) -> g+h would expand g first, then f, then h.
enum Symbol {
	// Locations expand to a Ast::Location node which may be used in a bank() builtin,
	// but will be coerced to the address if used otherwise.
	Location(Ast::Location),
	// Macros expand directly to a result AST
	Macro {
		params: Vec<String>,
		body: Ast::Node,
	},
	// Builtins call a rust function to return a calculated value
	Builtin {
		num_params: usize,
		func: fn(&Assembler, &[Ast::Node]) -> Result<Ast::Node, AssemblyError>,
	},
}

impl Symbol {
	fn expand(&self, assembler: &Assembler, args: &[Ast::Node]) -> Result<Ast::Node, AssemblyError> {
		match self {
			Symbol::Location(location) => Ok(location),
			Symbol::Macro{params, body} => Ok(expand_ast(params, body, args)),
			Symbol::Builtin{num_params, func} => {
				if args.len() == num_params {
					func(assembler, args)
				} else {
					unimplemented!() // TODO return "wrong args" error
				}
			},
		}
	}

	fn expand_ast(params: &[String], body: &Ast::Node, args: &[Ast::Node]) {
		unimplemented!() // TODO
	}
}

impl Assembler {
	fn new() -> Self {
		let mut symbols = HashMap::new();
		symbols.insert("bank".to_string(), Symbol::Builtin{num_params: 1, func: builtin_bank});
		symbols.insert("@".to_string(), Symbol::Builtin{num_params: 1, func: builtin_position});
		Assembler{
			current_bank: 0,
			banks: [None; 256],
			symbols,
		}
	}

	// Returns the current Bank
	fn bank(&self) -> &Bank {
		if let None = self.banks[self.bank_number] {
			self.banks[self.bank_number] = Bank {
				number: self.bank_number,
				position: 0,
				data: [0; u8],
			}
		};
		self.banks[self.bank_number]
	}

	// Takes an Ast chunk consisting of any number of statements, expands any symbols,
	// and updates the assembler state accordingly.
	fn ingest(&mut self, chunk: Ast::Node) -> Result<(), AssemblyError> {
		let expanded = self.expand_node(chunk)?;
		let statements: Vec<Ast::Statement> = Vec::new();
		collect_statements(&mut statements, expanded)?;
		for statement in statements {
			// TODO
		}
	}

	// Traverse any depth of Sequence nodes and flatten into a list of Statement nodes
	fn collect_statements(statements: &mut Vec<Ast::Statement>, node: Ast::Node) -> Result<(), AssemblyError> {
		match node {
			Ast::Sequence(children) => {
				for child in children {
					collect_statements(statements, child)?;
				}
			},
			statement @ Ast::Statement => {
				statements.push(statement);
			}
			_ => {
				unimplemented!() // TODO error - not a statement
			}
		};
	}

	// Recursively expands all nodes in the given AST, returning a transformed AST
	fn expand_node(&self, node: Ast::Node, depth: usize) -> Result<Ast::Node, AssemblyError> {
		if depth > MAX_EXPANSION_DEPTH {
			unimplemented!() // TODO error
		}

		// Children are expanded first. For calls, this is the args. For others, this is just
		// all sub-parts.
		let {value, children} = node;
		let expanded_children = children.into_iter()
			.map(|child| self.expand_node(child, depth + 1))
			.collect()? // colects into Result<Vec<Ast::Node>, AssemblyError>

		match value {
			// Symbol includes identifiers, operators, macros
			Ast::Node::Symbol(ident) => {
				let expanded = self.expand_symbol(ident, &expanded_children)?;
				// We may need to then expand the results
				self.expand_node(expanded, depth + 1)
			}
			// Identifiers are treated as zero-arg calls
			Ast::Node::Identifier(ident) => {
				let expanded = self.expand_symbol(ident, &[])?;
				// We may need to then expand the results
				self.expand_node(expanded, depth + 1)
			},
			// For anything else, pass through unchanged
			n => Ok(n),
		}
	}

	// Look up symbol and expand it
	fn expand_symbol(

	// Builtin definitions

	// @ returns the current position
	fn builtin_position(&self, _args: &[Ast::Node]) -> Result<Ast::Node, AssemblyError> {
		Ok(Ast::Node::Int(self.bank().position))
	}

	// @@ returns the current bank
	fn builtin_current_bank(&self, _args: &[Ast::Node]) -> Result<Ast::Node, AssemblyError> {
		Ok(Ast::Node::Int(self.bank_number))
	}

	// bank(location) returns the bank as a number, error for non-locations
	fn builtin_bank(&self, args: &[Ast::Node]) -> Result<Ast::Node, AssemblyError> {
		if let Ast::Node::Location(location) = args[0] {
			Ok(Ast::Node::Int(location.addr))
		} else {
			unimplemented!() // TODO
		}
	}
}
