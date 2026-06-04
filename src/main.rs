use std::fs;

use barb::interpreter::{Interpreter, parse_code};

fn main() {
    let contents = fs::read_to_string("hello.bf").expect("Should have been able to read the file");
    let operations = parse_code(&contents);
    let mut interpreter = Interpreter {
        memory: [0; 30000],
        data_ptr: 0,
        instr_ptr: 0,
        program: operations,
    };
    interpreter.run();
}
