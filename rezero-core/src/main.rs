mod ir;
mod parser;
mod verifier;
mod codegen;
mod vm;

use clap::{Parser, Subcommand};
use std::fs;
use std::io::{self, Read};
use z3::{Config, Context};

#[derive(Parser)]
#[command(name = "rezero-core")]
#[command(about = "ReZero Formally-Verified Reversible Compiler Stack", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Verify an .rz file using SMT solver
    Verify {
        file: String,
        #[arg(long)]
        stdin: bool,
    },
    /// Compile an .rz file to reversible .rzb bytecode
    Compile {
        file: String,
        #[arg(short, long)]
        output: String,
    },
    /// Run a compiled .rzb file in the reversible VM
    Run {
        file: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Verify { file, stdin } => {
            let content = if stdin {
                let mut buf = String::new();
                io::stdin().read_to_string(&mut buf)?;
                buf
            } else {
                fs::read_to_string(file)?
            };

            let program = parser::parse_rz(&content).map_err(|e| anyhow::anyhow!(e))?;

            let cfg = Config::new();
            let ctx = Context::new(&cfg);
            let mut verifier = verifier::Verifier::new(&ctx);

            for func in &program.functions {
                if let Err(e) = verifier.verify_function(func) {
                    let result = serde_json::json!({
                        "status": "verified_failed",
                        "error": {
                            "message": e,
                            "line": 0,
                            "character": 0
                        }
                    });
                    println!("{}", result);
                    return Ok(());
                }
            }

            println!("{}", serde_json::json!({ "status": "verified_success" }));
        }
        Commands::Compile { file, output } => {
            let content = fs::read_to_string(file)?;
            let program = parser::parse_rz(&content).map_err(|e| anyhow::anyhow!(e))?;
            let rzb = codegen::CodeGenerator::compile_to_rzb(program);
            fs::write(output, rzb)?;
            println!("Compilation successful.");
        }
        Commands::Run { file } => {
            let content = fs::read_to_string(file)?;
            let program: ir::Program = serde_json::from_str(&content)?;
            let mut vm = vm::ReversibleVM::new(program);

            println!("Starting Reversible VM...");
            while vm.step_forward() {
                println!("PC {}: Variables={:?}", vm.state.pc, vm.state.variables);
            }
            println!("Execution finished.");
        }
    }

    Ok(())
}
