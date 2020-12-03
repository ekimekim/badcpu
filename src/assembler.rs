use std::collections::HashMap;


// State needed on a line-to-line basis in assembler
struct Assembler {
	current_bank: u8,
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

enum Symbol {
	// An actual point in the code, ie. a label
	Location{bank: u8, addr: u8},
	Macro(Macro)
}

// Macros are symbols with a number of arguments that get replaced by their result.
// Macro replacements happen at the AST layer, not strings.
// Macros are expanded outer-first, ie. f(g) -> g+h would expand f first, then g, then h.
// Macros may expand directly to a result AST (including all user-defined macros)
// or they may call a builtin function to return a calculated value.
enum Macro {
	Ast{
		params: Vec<Ast::Identifier>,
		body: Ast::Node,
	},
	Builtin{
		num_params: usize,
		func: fn(&[Ast::Node]) -> Result<Ast::Node, AssemblyError>,
	},
}

impl Macro {
	fn expand(&self, args: &[Ast::Node]) -> Result<Ast::Node, AssemblyError> {
		match self {
			Macro::Ast{ref params, ref body} => {
				Ok(expand_ast(params, body, args))
			},
			Macro::Builtin{num_params, func} => {
				if args.len() == num_params {
					func(args)
				} else {
					unimplemented!() // TODO return "wrong args" error
				}
			},
		}
	}

	fn expand_ast(params: &[Ast::Identifer], body: &Ast::Node, args: &[Ast::Node]) {
		unimplemented!() // TODO
	}
}

impl Assembler {
	fn new() -> Self {
		let mut symbols = HashMap::new();
		symbols.insert("bank".to_string(), 
		Assembler{
			current_bank: 0,
			banks: [None; 256],
			symbols,
		}
	}
}
