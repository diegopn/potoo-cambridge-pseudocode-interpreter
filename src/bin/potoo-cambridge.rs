use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use potoo_cambridge_pseudocode_interpreter::{
    Edition, FileMode, Host, Interpreter, Value, check, parse,
};

struct CliHost {
    state: u64,
    files: HashMap<String, CliFile>,
}

enum CliFile {
    Read(BufReader<File>),
    Write(BufWriter<File>),
    Append(BufWriter<File>),
}

impl Host for CliHost {
    fn input(&mut self) -> Result<String, String> {
        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .map_err(|error| error.to_string())?;
        Ok(line)
    }

    fn output(&mut self, text: &str) -> Result<(), String> {
        println!("{text}");
        io::stdout().flush().map_err(|error| error.to_string())
    }

    fn random(&mut self, upper_exclusive: f64) -> Result<f64, String> {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        let unit = (self.state as f64 + 0.5) / (u64::MAX as f64 + 1.0);
        Ok(unit * upper_exclusive)
    }

    fn open_file(&mut self, path: &str, mode: FileMode) -> Result<(), String> {
        if self.files.contains_key(path) {
            return Err(format!("file {path} is already open"));
        }
        let file = match mode {
            FileMode::Read => CliFile::Read(BufReader::new(
                File::open(path).map_err(|error| error.to_string())?,
            )),
            FileMode::Write => CliFile::Write(BufWriter::new(
                File::create(path).map_err(|error| error.to_string())?,
            )),
            FileMode::Append => CliFile::Append(BufWriter::new(
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                    .map_err(|error| error.to_string())?,
            )),
            FileMode::Random => {
                return Err("random-record file service is not configured in the CLI".into());
            }
        };
        self.files.insert(path.to_owned(), file);
        Ok(())
    }

    fn read_file_line(&mut self, path: &str) -> Result<Option<String>, String> {
        let Some(CliFile::Read(reader)) = self.files.get_mut(path) else {
            return Err("file is not open in READ mode".into());
        };
        let mut line = String::new();
        if reader
            .read_line(&mut line)
            .map_err(|error| error.to_string())?
            == 0
        {
            return Ok(None);
        }
        if line.ends_with('\n') {
            line.pop();
        }
        if line.ends_with('\r') {
            line.pop();
        }
        Ok(Some(line))
    }

    fn write_file_line(&mut self, path: &str, line: &str) -> Result<(), String> {
        let writer = match self.files.get_mut(path) {
            Some(CliFile::Write(writer) | CliFile::Append(writer)) => writer,
            _ => return Err("file is not open in WRITE or APPEND mode".into()),
        };
        writeln!(writer, "{line}").map_err(|error| error.to_string())
    }

    fn file_eof(&mut self, path: &str) -> Result<bool, String> {
        let Some(CliFile::Read(reader)) = self.files.get_mut(path) else {
            return Err("file is not open in READ mode".into());
        };
        reader
            .fill_buf()
            .map(|buffer| buffer.is_empty())
            .map_err(|error| error.to_string())
    }

    fn close_file(&mut self, path: &str) -> Result<(), String> {
        let Some(file) = self.files.remove(path) else {
            return Err("file is not open".into());
        };
        match file {
            CliFile::Write(mut writer) | CliFile::Append(mut writer) => {
                writer.flush().map_err(|error| error.to_string())
            }
            CliFile::Read(_) => Ok(()),
        }
    }

    fn seek_record(&mut self, _path: &str, _address: i64) -> Result<(), String> {
        Err("random-record file service is not configured in the CLI".into())
    }

    fn read_record(&mut self, _path: &str) -> Result<Value, String> {
        Err("random-record file service is not configured in the CLI".into())
    }

    fn write_record(&mut self, _path: &str, _value: &Value) -> Result<(), String> {
        Err("random-record file service is not configured in the CLI".into())
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    if args.next().as_deref() != Some("--edition") {
        return Err("usage: potoo-cambridge --edition 2026|2027-2029 <source-file>".into());
    }
    let edition = match args.next().as_deref() {
        Some("2026") => Edition::Cambridge2026,
        Some("2027-2029") => Edition::Cambridge2027To2029,
        _ => return Err("edition must be 2026 or 2027-2029".into()),
    };
    let path = args
        .next()
        .ok_or_else(|| "source file is required".to_owned())?;
    if args.next().is_some() {
        return Err("unexpected extra argument".into());
    }
    let source = std::fs::read_to_string(&path).map_err(|error| format!("{path}: {error}"))?;
    let program = parse(&source, edition).map_err(|errors| {
        errors
            .into_iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    check(&program).map_err(|errors| {
        errors
            .into_iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_nanos() as u64
        | 1;
    Interpreter::new(CliHost {
        state: seed,
        files: HashMap::new(),
    })
    .run(&program)
    .map_err(|error| error.to_string())?;
    Ok(())
}
