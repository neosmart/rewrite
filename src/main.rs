use getopts::Options;
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use uuid::Uuid;

macro_rules! except {
    ($error:ident, $extra:tt) => {{
        let stderr = std::io::stderr();
        let mut stderr = stderr.lock();
        let _ = write!(&mut stderr, "{}\n", $extra);
        let _ = write!(&mut stderr, "{}\n", $error);
        std::process::exit(-1);
    }};
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

    {
        let mut buffer = [0; 512];
        let stdin = std::io::stdin();
        let mut stdin = stdin.lock();
        let mut f = File::create(&tempfile).unwrap_or_else(|e| {
            except!(e, "Failed to create temporary output file!");
        });

        loop {
            let bytes_read = match stdin.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => n,
                Err(e) => except!(e, "Error reading from stdin!"),
            };

            if let Err(e) = f.write_all(&buffer[0..bytes_read]) {
                except!(e, "Failed to write to temporary output file!");
            };
        }
    }

    if std::fs::rename(&tempfile, &outfile).is_err() {
        // fs::rename() does not support cross-device linking.
        // Copy and delete instead.
        std::fs::copy(&tempfile, &outfile).unwrap_or_else(|e| {
            except!(e, "Failed to create output file!");
        });
        std::fs::remove_file(&tempfile).unwrap_or_else(|e| {
            except!(e, "Failed to delete temporary output file!");
        });
    }
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

    if matches.free.is_empty() {
        print_usage(&program, opts, true, false);
        return;
    }

    let infile = &matches.free[0];
    redirect_to_file(infile);
}
