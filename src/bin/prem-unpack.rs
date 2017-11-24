extern crate byteorder;
extern crate failure;
extern crate clap;
extern crate prem_tools;

use clap::{Arg, ArgMatches, App};
use failure::Error;
use std::fs::File;
use std::io::{self, Write};
use std::process;

fn run(matches: &ArgMatches) -> Result<(), Error> {
    let result = match matches.value_of("INPUT") {
        None | Some("-") => {
            let input = io::stdin();
            prem_tools::uncompress(input)?
        }
        Some(path) => {
            let input = File::open(path)?;
            prem_tools::uncompress(input)?
        }
    };

    match matches.value_of("OUTPUT") {
        None | Some("-") => {
            let mut output = io::stdout();
            output.write_all(&result)?;
        }
        Some(path) => {
            let mut output = File::create(path)?;
            output.write_all(&result)?;
        }
    }
    Ok(())
}

fn main() {
    let matches = App::new("prem-unpack")
        .author("Joonas Javanainen <joonas.javanainen@gmail.com>")
        .about("Prehistorik Man compressed resource unpacker")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("OUTPUT")
                .help("Output file, or - to use standard output"),
        )
        .arg(
            Arg::with_name("INPUT")
                .help("Input file, or - to use standard input")
                .index(1),
        )
        .get_matches();
    if let Err(ref e) = run(&matches) {
        eprintln!("{}\n{}", e, e.backtrace());
        process::exit(1);
    }
}
