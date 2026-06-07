use std::collections::HashMap;
use std::fs::{self, write};
use std::path::{Path, PathBuf};

use barb::compiler::CodeGen;
use barb::interpreter::Interpreter;
use barb::package::{self, Manifest};
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
    New {
        name: String,
    },
    Install,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Run { file }) => {
            let operations = parse(file);
            run(operations, cli.interpret);
        }
        Some(Commands::Build { file, output_dir }) => {
            let operations = parse(file);
            let context = Context::create();
            let codegen = CodeGen::new(&context);
            codegen.compile(&operations);
            codegen.build(output_dir, file);
        }
        Some(Commands::New { name }) => {
            let dir = Path::new(name);
            fs::create_dir_all(dir).unwrap();

            let manifest = Manifest {
                name: name.clone(),
                features: HashMap::new(),
                dependencies: HashMap::new(),
            };

            write(dir.join("bark.toml"), toml::to_string(&manifest).unwrap()).unwrap();

            let hello = ">++++++++[<+++++++++>-]<.>++++[<+++++++>-]<+.+++++++..+++.>>++++++[<+++++++>-]<++.------------.>++++++[<+++++++++>-]<+.<.+++.------.--------.>>>++++[<++++++++>-]<+.";
            write(dir.join("main.bf"), hello).unwrap();

            println!("Created project: {name}");
        }
        Some(Commands::Install) => {
            let root = std::env::current_dir().unwrap();
            match package::Resolver::new(&root) {
                Some(ref r) => {
                    let _ = r;
                    println!("Dependencies resolved");
                }
                None => eprintln!("error: no bark.toml found in the current directory"),
            }
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
    package::parse_source(path)
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
