//! # rtee
//!
//! `rtee` reads from standard input and writes to standard output and
//! files.
//!

extern crate clap;
extern crate nix;

use clap::{App, Arg};
use nix::sys::signal;
use std::fs::OpenOptions;
use std::io::{self, BufWriter, Write};

fn main() {
    let matches = App::new("pipe fitting")
        .version(clap::crate_version!())
        .author("Gary Pennington <garypen@gmail.com>")
        .about("tee, but rustee")
        .arg(
            Arg::with_name("append")
                .short('a')
                .long("append")
                .help("Append output rather than overwrite"),
        )
        .arg(
            Arg::with_name("ignore")
                .short('i')
                .long("ignore")
                .help("Ignore SIGINT"),
        )
        .arg(
            Arg::with_name("file")
                .help("Sets the output files to use")
                .multiple(true),
        )
        .get_matches();

    let ignore = matches.is_present("ignore");
    let append = matches.is_present("append");
    let files = matches.values_of("file");

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
