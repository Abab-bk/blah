use std::path::{Path, PathBuf};
use std::fs;

use barb::compiler::CodeGen;
use barb::interpreter::{Interpreter, parse_code};
use barb::shared::Operation;
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
    Run {
        file: PathBuf,
    },
    Build {
        file: PathBuf,

        #[arg(short, long, default_value = "build/")]
        output_dir: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Run { file }) => {
            let operations = parse(&file);
            run(operations, cli.interpret);
        }
        Some(Commands::Build { file, output_dir }) => {
            let operations = parse(&file);
            let context = Context::create();
            let codegen = CodeGen::new(&context);
            codegen.compile(&operations);
            codegen.build(output_dir, file);
        }
        None => {
            let file = cli.file.as_ref().unwrap_or_else(|| {
                Cli::command()
                    .error(
                        clap::error::ErrorKind::MissingRequiredArgument,
                        "requires a brainfuck file",
                    )
                    .exit();
            });
            let operations = parse(file);
            run(operations, cli.interpret);
        }
    }
}

fn parse(path: &Path) -> Vec<Operation> {
    let contents = fs::read_to_string(path).unwrap();
    parse_code(&contents)
}

fn run(operations: Vec<Operation>, interpret: bool) {
    if interpret {
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
