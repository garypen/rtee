//! # rtee
//!
//! `rtee` reads from standard input and writes to standard output and
//! files.
//!

use std::fs::OpenOptions;
use std::io::{self, BufWriter, Write};

use clap::arg;
use clap::command;
use clap::ArgAction;

use nix::sys::signal;

fn main() {
    let matches = command!() // requires `cargo` feature
        .arg(
            arg!(
                -a --append "Append output rather than overwrite"
            )
            .action(ArgAction::SetTrue),
        )
        .arg(
            arg!(
                -i --ignore "ignore SIGINT"
            )
            .action(ArgAction::SetTrue),
        )
        .arg(
            arg!(
                [file] "Sets the output files to use"
            )
            .action(ArgAction::Append),
        )
        .get_matches();

    let ignore = matches
        .get_one::<bool>("ignore")
        .cloned()
        .unwrap_or_default();
    let append = matches
        .get_one::<bool>("append")
        .cloned()
        .unwrap_or_default();
    let files = matches.get_many::<String>("file");

    if ignore {
        let sig_action = signal::SigAction::new(
            signal::SigHandler::SigIgn,
            signal::SaFlags::empty(),
            signal::SigSet::empty(),
        );
        unsafe {
            signal::sigaction(signal::SIGINT, &sig_action).expect("Failed to ignore SIGINT!");
        }
    }

    let mut fopts = OpenOptions::new();
    fopts.create(true).write(true);

    if append {
        fopts.append(true);
    }

    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    if let Some(file_list) = files {
        let mut writers: Vec<BufWriter<Box<dyn Write>>> =
            vec![BufWriter::new(Box::new(&mut stdout))];
        for el in file_list {
            if el == "-" {
                writers.push(BufWriter::new(Box::new(io::stdout())));
            } else {
                writers.push(BufWriter::new(Box::new(
                    fopts
                        .open(el)
                        .unwrap_or_else(|_| panic!("Failed to open {}!", el)),
                )));
            }
        }
        let mut input = String::new();
        while stdin
            .read_line(&mut input)
            .expect("Failed to read input line!")
            > 0
        {
            for writer in &mut writers {
                writer
                    .write_all(input.as_bytes())
                    .expect("Failed to write output line");
                writer.flush().expect("Failed to flush output line");
            }
            input.clear();
        }
    } else {
        io::copy(&mut stdin, &mut stdout).expect("Failed to copy input to output");
    }
}
