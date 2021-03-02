use getopts::Options;
use std::env;
use std::fs::File;
use std::ffi::OsStr;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

macro_rules! except {
    ($error:ident, $extra:tt) => {{
        let stderr = std::io::stderr();
        let mut stderr = stderr.lock();
        let _ = write!(&mut stderr, "{}\n", $extra);
        let _ = write!(&mut stderr, "{}\n", $error);
        std::process::exit(-1);
    }};
}

fn get_temp_dest(path: &Path) -> PathBuf {
    let old_name = path.file_name().expect("Received path to directory instead of file!");
    let mut new_name = std::ffi::OsString::new();
    new_name.push(".");
    new_name.push(old_name);

    // Freeze its current value, because it'll serve as a template
    let new_name = new_name;

    // Handle cases where the temp file exists
    let mut num = 0;
    let mut new_path = path.with_file_name(&new_name);
    while new_path.exists() {
        num += 1;
        let mut new_name = new_name.clone();
        new_name.push(format!("-{}", num));
        new_path = path.with_file_name(new_name);
    }

    return new_path;
}

fn print_usage(program: &OsStr, opts: Options, include_info: bool, include_copyright: bool) {
    let program = Path::new(program);
    let command = program.file_name().map(Path::new).unwrap_or(program);

    let copyright =
        "rewrite 0.2 by NeoSmart Technologies. Written by Mahmoud Al-Qudsi <mqudsi@neosmart.net>";
    let brief = format!("Usage: {} FILE [options]", command.display());
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
    // Create the temporary file in the same directory as outfile. This lets us guarantee a rename
    // (instead of a move) upon completion where possible.
    let src = Path::new(outfile);
    let tempfile = get_temp_dest(src);

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
    let args: Vec<_> = env::args_os().collect();
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
