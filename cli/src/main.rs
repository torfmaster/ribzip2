use libribzip2::stream::{decode_stream, encode_stream};
use libribzip2::EncodingStrategy;
use num_cpus;
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

fn main() {
    let opt = Opt::from_args();
    match opt {
        Opt::Decompress { input } => {
            for file_name in input {
                let mut in_file = match File::open(&file_name) {
                    Err(why) => panic!("Couldn't open {}: {}", file_name.display(), why),
                    Ok(file) => file,
                };

                let mut out_file_name = file_name.clone();
                out_file_name.set_extension(OsString::from(""));
                if out_file_name.exists(){
                    panic!("Output file {} already exists", out_file_name.display());
                }
                let out_file = match File::create(&out_file_name) {
                    Err(why) => panic!("Couldn't create {}: {}", out_file_name.display(), why),
                    Ok(file) => file,
                };

                decode_stream(&mut in_file, out_file).unwrap();
            }
        }
        Opt::Compress {
            input,
            threads,
            encoding_options,
        } => {
            for file_name in input {
                let mut in_file = match File::open(&file_name) {
                    Err(why) => panic!("Couldn't open {}: {}", file_name.display(), why),
                    Ok(file) => file,
                };

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

                if out_file_name.exists(){
                    panic!("Output file {} already exists", out_file_name.display());
                }
                
                let out_file = match File::create(&out_file_name) {
                    Err(why) => panic!("Couldn't create {}: {}", out_file_name.display(), why),
                    Ok(file) => file,
                };

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
}
