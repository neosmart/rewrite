use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
extern crate uuid;
use uuid::Uuid;
extern crate getopts;
use getopts::Options;
use std::io;

macro_rules! exit_with_exception {
    ($error:ident, $extra:tt) => {
        let _ = write!(&mut std::io::stderr(), "{}\n", $extra);
        let _ = write!(&mut std::io::stderr(), "{}\n", $error);
        std::process::exit(-1);
    };
}

fn print_usage(program: &str, opts: Options, include_info: bool, include_copyright: bool) {
    let path = PathBuf::from(program);
    let command = path.file_name().unwrap().to_string_lossy();

    let copyright =
        "rewrite 0.2 by NeoSmart Technologies. Written by Mahmoud Al-Qudsi <mqudsi@neosmart.net>";
    let brief = format!("Usage: {} FILE [options]", command);
    let info = "Safely rewrite contents of FILE with stdin, even where FILE is being read by \
                upstream command";

    if include_copyright {
        println!("{}", copyright);
    }
    if include_info {
        println!("{}", info);
    }
    print!("{}", opts.usage(&brief));
}

fn redirect_to_file(outfile: &str) {
    // Create the temporary file in the same directory as outfile this lets us guarantee a rename
    // (instead of a move) on completion
    let mut tempfile = PathBuf::from(outfile);
    tempfile.pop(); // Now refers to parent, which might be nothing
    tempfile.push(Uuid::new_v4().hyphenated().to_string());
    // println!("{}", tempfile.display());

    {
        let mut buffer = [0; 512];
        let mut stdin = io::stdin();
        let mut f = File::create(&tempfile).unwrap_or_else(|e| {
            exit_with_exception!(e, "Failed to create temporary output file!");
        });

        loop {
            let read_bytes = stdin.read(&mut buffer).unwrap_or_else(|e| {
                exit_with_exception!(e, "Error reading from stdin!");
            });
            if read_bytes == 0 {
                break;
            }

            let write_bytes = f.write(&buffer[0..read_bytes]).unwrap_or_else(|e| {
                exit_with_exception!(e, "Failed to write to temporary output file!");
            });

            debug_assert!(write_bytes == read_bytes);
        }
    }

    std::fs::rename(&tempfile, &outfile).unwrap_or_else(|_x| {
        // fs::rename() does not support cross-device linking.
        // Copy and delete instead.
        std::fs::copy(&tempfile, &outfile).unwrap_or_else(|e| {
            exit_with_exception!(e, "Failed to create output file!");
        });
        std::fs::remove_file(&tempfile).unwrap_or_else(|e| {
            exit_with_exception!(e, "Failed to delete temporary output file!");
        });
    });
}

fn main() {
    let args: Vec<_> = env::args().collect();
    let program = &args[0];

    let mut opts = Options::new();
    opts.optflag("h", "help", "prints this help info");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => {
            println!("{}", e);
            print_usage(&program, opts, false, false);
            return;
        }
    };

    if matches.opt_present("h") {
        print_usage(&program, opts, true, true);
        return;
    }

    let infile = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        print_usage(&program, opts, true, false);
        return;
    };

    redirect_to_file(&infile);
}
