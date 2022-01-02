use std::fs::File;
use std::path::PathBuf;
use std::{ffi::OsString, io::BufWriter};
use bitwise::bitreader::BitReaderImpl;
use structopt::StructOpt;

use stream::{write_stream, write_stream_data};

mod stream;
mod bitwise;
mod lib;

#[derive(StructOpt)]
enum Opt {
    Decompress {
        #[structopt(parse(from_os_str), required = true)]
        input: Vec<PathBuf>,
    },
    Compress {
        #[structopt(parse(from_os_str), required = true)]
        input: Vec<PathBuf>,
        #[structopt(default_value = "1", long)]
        threads: usize,
    },
}

fn main() {
    let opt = Opt::from_args();
    match opt {
        Opt::Decompress { input } => {
            for file_name in input {
                let mut out_file_name = file_name.clone();
                out_file_name.set_extension(OsString::from("out"));
                let out_file = File::create(out_file_name).expect("Could not create file.");
                let mut in_file = File::open(file_name).unwrap();
                let mut in_file_bit_reader = BitReaderImpl::from_reader(&mut in_file);
                write_stream(&mut in_file_bit_reader, out_file).unwrap();
            }
        }
        Opt::Compress { input, threads } => {
            for file_name in input {
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

                let out_file = File::create(out_file_name).expect("Could not create File.");
                let mut out_file = BufWriter::new(out_file);
                let mut in_file = File::open(file_name).unwrap();
                write_stream_data(&mut in_file, &mut out_file, threads);
            }
        }
    }
}
