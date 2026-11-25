//! WAFFLE command-line tool.

use anyhow::Result;
use clap::{Parser, Subcommand};
use log::debug;
use std::path::PathBuf;
use waffle::{entity::EntityRef, FrontendOptions, Func, Module, OptOptions};

#[derive(Debug, Parser)]
#[command(name = "waffle-util", about = "WAFFLE utility.")]
struct Options {
    #[arg(short, long)]
    debug: bool,

    #[arg(
        help = "Do basic optimizations: GVN and const-prop",
        long = "basic-opts"
    )]
    basic_opts: bool,

    #[arg(
        help = "Enable parsing of debug-info from input",
        short = 'g',
        long = "debug-info"
    )]
    debug_info: bool,

    #[arg(help = "Transform to maximal SSA", long = "max-ssa")]
    max_ssa: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[command(name = "print-ir", about = "Parse Wasm and print resulting IR")]
    PrintIR {
        #[arg(help = "Wasm file to parse")]
        wasm: PathBuf,
    },
    #[command(name = "print-func", about = "Parse Wasm and print one function body")]
    PrintFunc {
        #[arg(help = "Wasm file to parse")]
        wasm: PathBuf,
        #[arg(help = "Index of Wasm function to print")]
        func: usize,
    },
    #[command(name = "roundtrip", about = "Round-trip Wasm through IR")]
    RoundTrip {
        #[arg(help = "Wasm file to parse", short = 'i')]
        input: PathBuf,
        #[arg(help = "Wasm file to produce", short = 'o')]
        output: PathBuf,
    },
}

fn apply_options(opts: &Options, module: &mut Module) -> Result<()> {
    module.expand_all_funcs()?;
    if opts.basic_opts {
        module.per_func_body(|body| body.optimize(&OptOptions::default()));
    }
    if opts.max_ssa {
        module.per_func_body(|body| body.convert_to_max_ssa(None));
    }
    Ok(())
}

fn main() -> Result<()> {
    let opts = Options::parse();

    let mut logger = env_logger::Builder::from_default_env();
    if opts.debug {
        logger.filter_level(log::LevelFilter::Debug);
    }
    let _ = logger.try_init();

    let mut options = FrontendOptions::default();
    options.debug = opts.debug_info;

    match &opts.command {
        Command::PrintIR { wasm } => {
            let bytes = std::fs::read(wasm)?;
            debug!("Loaded {} bytes of Wasm data", bytes.len());
            let mut module = Module::from_wasm_bytes(&bytes[..], &options)?;
            apply_options(&opts, &mut module)?;
            println!("{}", module.display());
        }
        Command::PrintFunc { wasm, func } => {
            let bytes = std::fs::read(wasm)?;
            debug!("Loaded {} bytes of Wasm data", bytes.len());
            let mut module = Module::from_wasm_bytes(&bytes[..], &options)?;
            apply_options(&opts, &mut module)?;
            println!(
                "{}",
                module.funcs[Func::new(*func)]
                    .body()
                    .unwrap()
                    .display_verbose("", Some(&module))
            );
        }
        Command::RoundTrip { input, output } => {
            let bytes = std::fs::read(input)?;
            debug!("Loaded {} bytes of Wasm data", bytes.len());
            let mut module = Module::from_wasm_bytes(&bytes[..], &options)?;
            apply_options(&opts, &mut module)?;
            let produced = module.to_wasm_bytes()?;
            std::fs::write(output, &produced[..])?;
        }
    }

    Ok(())
}
