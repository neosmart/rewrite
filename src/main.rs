use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

macro_rules! except {
    ($error:ident, $extra:tt) => {{
        let stderr = std::io::stderr();
        let mut stderr = stderr.lock();
        let _ = writeln!(&mut stderr, "{}", $extra);
        let _ = writeln!(&mut stderr, "{}", $error);
        std::process::exit(-1);
    }};
}

fn get_temp_dest(path: &Path) -> PathBuf {
    let old_name = path
        .file_name()
        .expect("Received path to directory instead of file!");
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

fn help<W: std::io::Write>(output: &mut W, verbose: bool) {
    let _ = writeln!(output, "Usage: rewrite [options] FILE");
    let _ = writeln!(
        output,
        "Safely rewrite contents of FILE with stdin, even where FILE \
        is being read by upstream command"
    );

    if verbose {
        let _ = writeln!(output, "");
        let _ = writeln!(output, "Options:");
        let _ = writeln!(output, "\t-h, --help      prints this help info and exit");
        let _ = writeln!(output, "\t-V, --version   show version info and exit");
    }
}

fn version<W: std::io::Write>(output: &mut W) {
    let _ = writeln!(output, "rewrite {}", env!("CARGO_PKG_VERSION"));
    let _ = writeln!(
        output,
        "Copyright (C) NeoSmart Technologies 2017-2021. \
        Written by Mahmoud Al-Qudsi <mqudsi@neosmart.net>"
    );
}

fn redirect_to_file(outfile: &OsStr) {
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
    let args = std::env::args_os();

    let mut file = None;
    let mut skip_switches = false;
    for arg_os in args.skip(1) {
        if let Some(arg) = arg_os.to_str() {
            if !skip_switches && arg.starts_with("-") && arg != "-" {
                match arg {
                    "-h" | "--help" => {
                        help(&mut std::io::stdout(), true);
                        std::process::exit(0);
                    }
                    "-V" | "--version" => {
                        version(&mut std::io::stdout());
                        std::process::exit(0);
                    }
                    // "--line-buffered" => {
                    //     force_flush = true;
                    // }
                    "--" => {
                        skip_switches = true;
                        continue;
                    }
                    _ => {
                        eprintln!("{}: Invalid option!", arg);
                        eprintln!("");
                        help(&mut std::io::stderr(), false);
                        eprintln!("Try 'rewrite --help' for more information");
                        std::process::exit(-1);
                    }
                }
            }
        }

        if file.replace(arg_os).is_some() {
            // A destination was provided twice
            eprintln!("Multiple output files provided!");
            eprintln!("Try 'rewrite --help' for usage information");
            std::process::exit(-1);
        }
    }

    let file = match file {
        Some(file) => file,
        None => {
            version(&mut std::io::stderr());
            eprintln!("");
            help(&mut std::io::stderr(), false);
            std::process::exit(-1);
        }
    };

    redirect_to_file(&file);
}
