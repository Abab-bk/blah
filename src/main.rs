use std::fs;
use std::path::PathBuf;

use barb::compiler::CodeGen;
use barb::interpreter::{Interpreter, parse_code};
use clap::{CommandFactory, Parser, Subcommand};
use inkwell::context::Context;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    #[arg(short, long, global = true)]
    interpret: bool,

    #[command(subcommand)]
    command: Option<Commands>,

    file: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Run { file: PathBuf },
}

fn main() {
    let cli = Cli::parse();

    let file_path = match &cli.command {
        Some(Commands::Run { file }) => file,
        None => match &cli.file {
            Some(file) => file,
            None => Cli::command()
                .error(
                    clap::error::ErrorKind::MissingRequiredArgument,
                    "requires a brainfuck file",
                )
                .exit(),
        },
    };

    let contents = fs::read_to_string(file_path).unwrap();
    let operations = parse_code(&contents);

    if cli.interpret {
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
