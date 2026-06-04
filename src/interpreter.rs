use std::io::{self, Read};

use crate::shared::Operation;

pub struct Interpreter {
    pub memory: [u8; 30000],
    pub data_ptr: usize,
    pub instr_ptr: usize,
    pub program: Vec<Operation>,
}

impl Interpreter {
    pub fn run(&mut self) {
        while self.instr_ptr < self.program.len() {
            let operation = &self.program[self.instr_ptr];

            match operation {
                Operation::Next => self.data_ptr += 1,
                Operation::Prev => self.data_ptr -= 1,
                Operation::Increment => self.memory[self.data_ptr] += 1,
                Operation::Decrement => self.memory[self.data_ptr] -= 1,
                Operation::Output => print!("{}", self.memory[self.data_ptr] as char),
                Operation::Input => {
                    let mut buffer = [0u8; 1];
                    if io::stdin().read(&mut buffer).is_ok() {
                        self.memory[self.data_ptr] = buffer[0];
                    } else {
                        self.memory[self.data_ptr] = 0;
                    }
                }
                Operation::JumpIfZero(target) => {
                    if self.memory[self.data_ptr] == 0 {
                        self.instr_ptr = *target;
                        continue;
                    }
                }
                Operation::JumpIfNonzero(target) => {
                    if self.memory[self.data_ptr] != 0 {
                        self.instr_ptr = *target;
                        continue;
                    }
                }
            }

            self.instr_ptr += 1;
        }
    }
}

pub fn parse_code(code: &str) -> Vec<Operation> {
    let mut operaions: Vec<Operation> = Vec::new();

    let mut stack: Vec<usize> = Vec::new();

    for token in code.chars() {
        match token {
            '>' => operaions.push(Operation::Next),
            '<' => operaions.push(Operation::Prev),
            '+' => operaions.push(Operation::Increment),
            '-' => operaions.push(Operation::Decrement),
            '.' => operaions.push(Operation::Output),
            ',' => operaions.push(Operation::Input),
            '[' => {
                stack.push(operaions.len());
                operaions.push(Operation::JumpIfZero(0));
            }
            ']' => {
                let start_idx = stack.pop().expect("括号不匹配：多余的 ]");
                let end_idx = operaions.len();

                if let Operation::JumpIfZero(ref mut target) = operaions[start_idx] {
                    *target = end_idx;
                }

                operaions.push(Operation::JumpIfNonzero(start_idx));
            }
            _ => {}
        }
    }
    return operaions;
}
