mod ir; mod parser; mod verifier; mod codegen;
use clap::{Parser, Subcommand};
use std::{fs, process::Command, io::{self, Read}};
use z3::{Config, Context};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Verify {
        file: String,
        #[arg(long)]
        stdin: bool,
    },
    Build {
        file: String,
        #[arg(short, long)]
        output: String,
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

            let p = match parser::parse(&content) {
                Ok(prog) => prog,
                Err(e) => {
                    println!("{}", serde_json::json!({ "status": "verified_failed", "error": { "message": format!("Parse Error: {}", e), "line": 0, "character": 0 } }));
                    return Ok(());
                }
            };

            let cfg = Config::new();
            let ctx = Context::new(&cfg);
            let mut v = verifier::Verifier::new(&ctx);

            for f in &p.functions {
                if let Err(e) = v.verify_function(f) {
                    println!("{}", serde_json::json!({ "status": "verified_failed", "error": { "message": format!("{}", e), "line": 0, "character": 0 } }));
                    return Ok(());
                }
            }
            println!("{}", serde_json::json!({ "status": "verified_success" }));
        }
        Commands::Build { file, output } => {
            let p = parser::parse(&fs::read_to_string(file)?)?;
            let c_code = codegen::NativeGenerator::generate_c(&p);
            fs::write("temp.c", c_code)?;
            Command::new("clang").args(["temp.c", "-o", &output]).status()?;
            fs::remove_file("temp.c")?;
            println!("Build successful: {}", output);
        }
    }
    Ok(())
}
