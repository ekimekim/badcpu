use std::collections::HashMap;


const MAX_EXPANSION_DEPTH: usize = 1024;

/*
Syntax:
	comment: # text
	instructions:
		[+|-] [!] INSTR {ARGS}
		+ means only run if cond set, - or omitted means only if cond unset
		! means set cond
		ARGS are register (A, IP, P, [P]) or number depending on instr
	data: NUMBER {, NUMBER}
	set current position: @[BANK:]NUMBER
	define label: IDENT:
		a label is equivalent to a zero-arg macro
	define macro:
		IDENT({ARGS}) { BODY }
	apply macro:
		IDENT {ARGS}
		by default each word after IDENT will be parsed as a seperate arg,
		override with (), eg. "outer (inner 1) 2" will invoke the inner macro
		with arg 1, then invoke the outer macro with args of inner's result, and 2.
*/

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
	// Locations expand to a 16-bit (bank, addr) value.
	// Note in most cases this will end up coerced to a u8 when used
	Location(u16),
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
		let expanded = self.eval_node(chunk, 0)?;
		self.ingest_node(expanded)
	}

	// Takes a fully-expanded Ast chunk and updates assember state from it
	fn ingest_node(&mut self, chunk: Ast::Node) -> Result<(), AssemblyError> {
		Ok(()) // TODO
	}

	// Recursively evaluates all nodes in the given AST, returning a transformed AST (the expression result)
	// and mutating assembler state if needed.
	fn eval_node(&mut self, node: Ast::Node, depth: usize) -> Result<Ast::Node, AssemblyError> {
		if depth > MAX_EXPANSION_DEPTH {
			unimplemented!() // TODO error
		}

		// First, eval the children in order (order matters for side effects).
		// In most cases all sub-nodes are children, but defining a macro is an exception,
		// its body is stored seperately and is not evaluated.
		let {value, children} = node;
		let children = children.into_iter()
			.map(|child| self.eval_node(child, depth + 1))
			.collect()? // colects into Result<Vec<Ast::Node>, AssemblyError>

		match value {
			// Symbol includes identifiers, operators/builtins and macros.
			// We expand to their result, then evaluate the result.
			Ast::Node::Symbol(ident) => {
				let expanded = self.expand_symbol(ident, &children)?;
				self.eval_node(expanded, depth + 1)
			},
			// Primitive values return themselves
			Ast::Node::Integer | Ast::Node::Atom => Ok(Ast::Node{value, children}),
			// All the following are evaluated for side effects and return Ast::Node::Void.
			// Sequence nodes have no effect except to eval all their children,
			// which we've already done.
			Ast::Node::Sequence => Ok(Ast::Node::Void),
			// Labels are added to the symbols table
			Ast::Node::Label
			// Macro definitions are added to the symbols table
			Ast::Node::Definition(ident, body)
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

}
