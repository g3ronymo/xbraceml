use std::process;
use std::io;
use clap::Parser;
use xbraceml::{Config, Plugin};

/// Makes writing xml less redundant.
/// Converts a simple markup into xml. Extensible via
/// easy to write and language independent plugins.
#[derive(Parser)]
struct Cli {
    /// Path to a input file or "-". If "-" read standard input.
    source: String,
    /// Path to a output file or "-". If "-" write to standard output.
    destination: String,
    /// Disable special elements. 
    #[arg(short)]
    disable_special_elements: bool,
    /// use long form for empty elements
    #[arg(short)]
    long_empty: bool,
    /// Path to a plugin file, a directory containing plugins, or a
    /// executable command. Can be specified multiple times
    #[arg(short)]
    plugins: Vec<String>,
}

impl Cli {
    fn to_config(&self) -> Result<Config, io::Error> {
        let mut plugins: Vec<Plugin> = Vec::new();
        for s in self.plugins.iter() {
            let mut p = Plugin::init(s)?;
            plugins.append(&mut p);
        }
        let config = Config{
            src: self.source.clone(),
            dst: self.destination.clone(),
            disable_special_elements: self.disable_special_elements,
            long_empty: self.long_empty,
            plugins,
        };
        return Ok(config);
    }
}

fn main() {
    simple_logger::init_with_level(log::Level::Warn).unwrap();
    let config = Cli::parse().to_config().unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });
    if let Err(e) = xbraceml::run(&config) {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}
