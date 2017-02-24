use std::env;
use std::io::prelude::*;
use std::fs::File;
use std::path::PathBuf;
extern crate uuid;
use uuid::Uuid;
extern crate getopts;
use getopts::Options;
use std::io;

fn print_usage(program: &str, opts: Options) {
    let path = PathBuf::from(program);
    let command = path.file_name().unwrap().to_string_lossy();

    let brief = format!("Usage: {} FILE [options]", command);
    let info = "Safely rewrite contents of FILE with stdin, even\nwhere FILE is being read by \
                upstream command";
    print!("{}", opts.usage(&[&brief, info].join("\n")));
}

fn redirect_to_file(outfile: &str) -> Result<(), io::Error> {
    let mut tempfile = env::temp_dir();
    tempfile.push(Uuid::new_v4().hyphenated().to_string());
    // println!("{}", tempfile.display());

    {
        let mut buffer = [0; 512];
        let mut stdin = io::stdin();
        let mut f = File::create(&tempfile).expect("Failed to create temporary file!");

        loop {
            let read_bytes = stdin.read(&mut buffer).expect("Error reading from stdin!");
            if read_bytes == 0 {
                break;
            }

            let write_bytes = f.write(&buffer[0..read_bytes])
                .expect("Failed to write to temporary file!");

            assert!(write_bytes == read_bytes);
        }
    }

    match std::fs::rename(&tempfile, &outfile) {
        Ok(m) => m,
        _ => {
            // fs::rename does not support cross-device linking
            // copy and delete instead
            std::fs::copy(&tempfile, &outfile).expect("Failed to create output file!");
            std::fs::remove_file(&tempfile)
                .unwrap_or_else(|e| print!("Failed to delete temporary file!\n{:?}", e));
        }
    };

    return Ok(());
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.optflag("h", "help", "prints this help info");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }
    let infile = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        print_usage(&program, opts);
        return;
    };

    assert!(redirect_to_file(&infile).is_ok());
}
