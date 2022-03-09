use libribzip2::stream::{decode_stream, encode_stream};
use libribzip2::EncodingStrategy;
use num_cpus;
use std::fmt;
use std::fs::File;
use std::path::PathBuf;
use std::{ffi::OsString, io::BufWriter};
use structopt::StructOpt;

#[derive(StructOpt)]
enum Opt {
    Decompress {
        #[structopt(parse(from_os_str), required = true)]
        input: Vec<PathBuf>,
    },
    Compress {
        #[structopt(parse(from_os_str), required = true)]
        input: Vec<PathBuf>,
        #[structopt(long)]
        threads: Option<usize>,
        #[structopt(subcommand)]
        encoding_options: Option<EncodingOptions>,
    },
}

#[derive(StructOpt, Clone, Copy)]
pub(crate) enum EncodingOptions {
    Single,
    KMeans {
        #[structopt(default_value = "3", long)]
        iterations: usize,
        #[structopt(default_value = "6", long)]
        num_tables: usize,
    },
}

#[derive(Debug)]
pub enum FileError {
    DuplicateError(PathBuf),
    IoError(std::io::Error),
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileError::DuplicateError(file_path) => {
                write!(f, "Output file {} already exists", file_path.display())
            }
            FileError::IoError(io_error) => write!(f, "{}", io_error),
        }
    }
}

impl From<std::io::Error> for FileError {
    fn from(err: std::io::Error) -> Self {
        FileError::IoError(err)
    }
}

impl std::error::Error for FileError {}

fn create_file(file_path: &PathBuf) -> Result<File, FileError> {
    if file_path.exists() {
        return Err(FileError::DuplicateError(file_path.clone()));
    }
    let file = File::create(&file_path)?;
    Ok(file)
}

fn open_file(file_path: &PathBuf) -> Result<File, FileError> {
    let file = File::open(&file_path)?;
    Ok(file)
}

fn try_main(opt: Opt) -> Result<(), FileError> {
    match opt {
        Opt::Decompress { input } => {
            for file_name in input {
                let mut in_file = open_file(&file_name)?;
                let mut out_file_name = file_name.clone();
                out_file_name.set_extension(OsString::from(""));
                let out_file = create_file(&out_file_name)?;
                decode_stream(&mut in_file, out_file).unwrap();
            }
        }
        Opt::Compress {
            input,
            threads,
            encoding_options,
        } => {
            for file_name in input {
                let mut in_file = open_file(&file_name)?;
                let mut out_file_name = file_name.clone();
                let extension = out_file_name.extension().map(|x| {
                    let mut y = x.to_os_string();
                    y.push(".bz2");
                    y
                });

                match extension {
                    Some(ext) => {
                        out_file_name.set_extension(ext);
                    }
                    None => {
                        out_file_name.set_extension(OsString::from("bz2"));
                    }
                }

                let out_file = create_file(&out_file_name)?;
                let mut out_file = BufWriter::new(out_file);
                let encoding_strategy = match encoding_options {
                    Some(EncodingOptions::Single) | None => EncodingStrategy::Single,
                    Some(EncodingOptions::KMeans {
                        iterations,
                        num_tables,
                    }) => EncodingStrategy::BlockWise {
                        num_iterations: iterations,
                        num_clusters: num_tables,
                    },
                };
                let threads_val = threads.unwrap_or(num_cpus::get());
                encode_stream(&mut in_file, &mut out_file, threads_val, encoding_strategy);
            }
        }
    }

    Ok(())
}

fn main() {
    let opt = Opt::from_args();
    try_main(opt).unwrap_or_else(|err| {
        eprintln!("{}", err);
        std::process::exit(1);
    });
}
