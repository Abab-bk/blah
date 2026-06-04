use std::fs;

use barb::compiler::CodeGen;
use barb::interpreter::{Interpreter, parse_code};
use inkwell::context::Context;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let use_interpret = args.iter().count() > 1 && args[1] == "-i";
    let file_index = if use_interpret { 2 } else { 1 };
    let filename = args.get(file_index).unwrap();

    let contents = fs::read_to_string(filename).unwrap();
    let operations = parse_code(&contents);

    if use_interpret {
        let mut interpreter = Interpreter {
            memory: [0; 30000],
            data_ptr: 0,
            instr_ptr: 0,
            program: operations,
        };
        interpreter.run();
    } else {
        let context = Context::create();
        let codegen = CodeGen::new(&context);
        codegen.compile(&operations);
        codegen.jit_run();
    }
}
