// The MIT License (MIT)

// Copyright (c) 2021 AnonymousDapper

#![deny(rust_2018_idioms)]

use clap::clap_app;

use colored::Colorize;

use std::path::Path;

use rusty_8::{dis, display};

fn print_fatal<S: std::fmt::Display>(msg: S) -> ! {
    eprintln!("[{}]: {}", "rusty-8 error".red().bold(), msg);

    std::process::exit(1);
}

fn read_file<P: AsRef<Path>>(path_ref: P) -> Vec<u8> {
    let path = path_ref.as_ref();

    if path.exists() {
        let f = std::fs::read(path);

        match f {
            Ok(s) => s,
            Err(e) => print_fatal(e),
        }
    } else {
        print_fatal(format!("{}: not found", path.display()));
    }
}

fn check_u64(v: String) -> Result<(), String> {
    if v.parse::<u64>().is_ok() {
        return Ok(());
    }

    Err(format!("`{}` is not a valid integer", v))
}

fn main() {
    let matches = clap_app!(tmp =>
        (version: env!("CARGO_PKG_VERSION"))
        (about: env!("CARGO_PKG_DESCRIPTION"))
        (@arg debug: -D --debug "Enable debug output")
        (@arg delay: --("cycle-sleep") +takes_value {check_u64} "Milliseconds to sleep between cycles (default 2, unless debug then 500)")
        (@arg disassemble: --disassemble "Perform disassembly instead of executing")
        (@arg file: * +takes_value "Path to CHIP-8 ROM")
    )
    .name("rusty-8")
    .get_matches();

    let debug = matches.is_present("debug");
    let disassembly = matches.is_present("disassemble");

    if matches.is_present("file") {
        let file_name = matches.value_of_os("file").unwrap();

        let source = read_file(file_name);

        let cycle_delay = match matches.value_of("delay") {
            Some(num_s) => num_s.parse::<u64>().unwrap(),
            None => 2,
        };

        let mut system = rusty_8::System::new(cycle_delay);

        system.load_rom(&source);
        display::init(file_name.to_str().unwrap());

        // CTRL+C signal handler
        ctrlc::set_handler(rusty_8::handle_ctrlc).expect("Error setting break handler");

        if debug {
            system.run(Some(&mut dis::Disassembler::new()))
        } else {
            system.run(None);
        }
    } else {
        println!("Nothing to do.");
    }

    /*display::init();

    #[rustfmt::skip]
    let data = [
        [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff],
        [128, 0, 0, 0, 0, 0, 0, 1],
        [128, 0, 0, 0, 0, 0, 0, 1],
        [128, 0, 0, 0, 0, 0, 0, 1],
        [128, 0, 0, 0, 0, 0, 0, 1],
        [128, 0, 0, 0, 0, 0, 0, 1],
        [128, 0, 0, 0, 0, 0, 0, 1],
        [128, 0, 0, 0, 0, 0, 0, 1],
        [128, 0, 0, 0, 0, 0, 0, 1],
        [128, 0, 0, 0, 0, 0, 0, 1],
        [128, 0, 0, 0, 0, 0, 0, 1],
        [128, 0, 0, 0, 0, 0, 0, 1],
        [128, 0, 0, 0, 0, 0, 0, 1],
        [128, 0, 0, 0, 0, 0, 0, 1],
        [128, 0, 0, 0, 0, 0, 0, 1],
        [128, 0, 0, 0, 0, 0, 0, 1],
        [128, 0, 0, 0, 0, 0, 0, 1],
        [128, 0, 0, 0, 0, 0, 0, 1],
        [128, 0, 0, 0, 0, 0, 0, 1],
        [128, 0, 0, 0, 0, 0, 0, 1],
        [128, 0, 0, 0, 0, 0, 0, 1],
        [128, 0, 0, 0, 0, 0, 0, 1],
        [128, 0, 0, 0, 0, 0, 0, 1],
        [128, 0, 0, 0, 0, 0, 0, 1],
        [128, 0, 0, 0, 0, 0, 0, 1],
        [128, 0, 0, 0, 0, 0, 0, 1],
        [128, 0, 0, 0, 0, 0, 0, 1],
        [128, 0, 0, 0, 0, 0, 0, 1],
        [128, 0, 0, 0, 0, 0, 0, 1],
        [128, 0, 0, 0, 0, 0, 0, 1],
        [128, 0, 0, 0, 0, 0, 0, 1],
        [128, 0, 0, 0, 0, 0, 0, 1],
    ];

    //    0xF0, 0x90, 0x90, 0x90, 0xF0,
    //    0x20, 0x60, 0x20, 0x20, 0x70,
    //    0xF0, 0x10, 0xF0, 0x80, 0xF0,
    //    0xF0, 0x10, 0xF0, 0x10, 0xF0,
    //    0x90, 0x90, 0xF0, 0x10, 0x10,
    //    0xF0, 0x80, 0xF0, 0x10, 0xF0,
    //    0xF0, 0x80, 0xF0, 0x90, 0xF0,
    //    0xF0, 0x10, 0x20, 0x40, 0x40,
    //    0xF0, 0x90, 0xF0, 0x90, 0xF0,
    //    0xF0, 0x90, 0xF0, 0x10, 0xF0,
    //    0xF0, 0x90, 0xF0, 0x90, 0x90,
    //    0xE0, 0x90, 0xE0, 0x90, 0xE0,
    //    0xF0, 0x80, 0x80, 0x80, 0xF0,
    //    0xE0, 0x90, 0x90, 0x90, 0xE0,
    //    0xF0, 0x80, 0xF0, 0x80, 0xF0,
    //    0xF0, 0x80, 0xF0, 0x80, 0x80

    //let data: [u8; 255] = [0; 255];

    display::write_display(&data);

    let cpu = rusty_8::System::new();

    let disassembler = dis::Disassembler::from(&cpu);

    disassembler.print_state();*/

    println!("{}", termion::cursor::Show);
}
