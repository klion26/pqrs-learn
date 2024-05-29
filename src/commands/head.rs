use std::path::PathBuf;

use clap::{Parser};
use log::debug;

use crate::errors::PQRSError;
use crate::errors::PQRSError::FileNotFound;
use crate::utils::{check_path_present, Formats, open_file, print_rows};

#[derive(Parser, Debug)]
pub struct HeadCommandArgs {
    #[clap(short, long, conflicts_with = "json")]
    csv: bool,
    #[clap(short, long, conflicts_with = "csv")]
    json: bool,
    #[clap(short = 'n', long, default_value = "5")]
    records: usize,
    file: PathBuf,
}

pub fn execute(opts: HeadCommandArgs) -> Result<(), PQRSError> {
    let format = if opts.json {
        Formats::Json
    } else if opts.csv {
        Formats::Csv
    } else {
        Formats::Default
    };

    debug!("The file name to read is: {}", opts.file.display());
    debug!("Number of records to print is: {}", opts.records);
    debug!("output format: {}", format);


    if !check_path_present(&opts.file) {
        return Err(FileNotFound(opts.file));
    }

    let file = open_file(&opts.file)?;
    print_rows(file, Some(opts.records), format)
}