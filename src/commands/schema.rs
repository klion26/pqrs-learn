use std::fmt;
use std::fmt::Formatter;
use std::path::PathBuf;
use clap::{Arg, ArgMatches, Parser};
use log::debug;
use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::schema::printer::{print_file_metadata, print_parquet_metadata};
use crate::errors::PQRSError;
use crate::errors::PQRSError::FileNotFound;
use crate::utils::{check_path_present, open_file};

#[derive(Parser, Debug)]
pub struct SchemaCommandArgs {
    files: Vec<PathBuf>,
    #[arg(short = 'D', long)]
    detailed: bool,
    #[arg(short, long, conflicts_with = "detailed")]
    json: bool,
}

pub fn execute(opts: SchemaCommandArgs) -> Result<(), PQRSError> {
    debug!("The file names to read are: {:?}", opts.files);
    debug!("Print detailed output:{:?}", opts.detailed);

    for file_name in &opts.files {
        if !check_path_present(file_name) {
            return Err(FileNotFound(file_name.to_path_buf()));
        }
    }

    for file_name in &opts.files {
        let file = open_file(file_name)?;
        match SerializedFileReader::new(file) {
            Ok(parquet_reader) => {
                let metadata = parquet_reader.metadata();
                println!("Metadata for file: {}", file_name.display());
                println!();
                if opts.detailed {
                    print_parquet_metadata(&mut std::io::stdout(), metadata);
                } else {
                    print_file_metadata(&mut std::io::stdout(), metadata.file_metadata())
                }
            }
            Err(e) => { return Err(PQRSError::ParquetError(e)); }
        }
    }

    Ok(())
}
