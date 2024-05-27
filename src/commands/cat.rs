
use clap::{Arg, ArgMatches, Parser};
use log::debug;
use crate::errors::PQRSError;
use crate::errors::PQRSError::FileNotFound;
use crate::utils::{check_path_present, is_hidden, open_file, print_rows};
use std::fmt;
use walkdir::WalkDir;
use std::collections::HashSet;
use std::fs::metadata;
use std::path::PathBuf;
use linked_hash_set::LinkedHashSet;
use crate::utils::Formats;

#[derive(Parser, Debug)]
pub struct CatCommandArgs {
    #[clap(short, long, conflicts_with = "json")]
    csv: bool,
    #[clap(long = "no-header", requires = "csv", conflicts_with = "json")]
    csv_no_header: bool,
    #[clap(short, long, conflicts_with = "csv")]
    json: bool,
    locations: Vec<PathBuf>,
}

pub(crate) fn execute(opts: CatCommandArgs) -> Result<(), PQRSError> {
    let format = if opts.json {
        Formats::Json
    } else if opts.csv {
        Formats::Csv
    } else if opts.csv_no_header {
        Formats::CsvNoHeader
    } else {
        Formats::Default
    };

    debug!("The location to read from are: {:?} using output format: {:?}", &opts.locations, format);

    let mut directories = vec![];
    let mut files = LinkedHashSet::new();
    for location in &opts.locations {
        let meta = metadata(location).unwrap();
        if meta.is_dir() {
            directories.push(location.clone())
        }

        if meta.is_file() {
            files.insert(location.clone());
        }
    }

    for directory in &directories {
        let walker = WalkDir::new(directory).into_iter();
        for entry in walker
            .filter_entry(|e| !is_hidden(e))
            .filter_map(|e| e.ok()) {
            debug!("{}", entry.path().display());
            let path = entry.path().to_path_buf();
            let meta = metadata(&path).unwrap();
            if meta.is_file() {
                files.insert(path);
            }
        }
    }

    debug!("The files are: {:?}", files);

    for file_name in &files {
        if !check_path_present(file_name) {
            return Err(FileNotFound(file_name.to_path_buf()));
        }
    }

    for file_name in &files {
        let file = open_file(file_name)?;
        let info_string = format!("File: {}", file_name.display());
        let length = info_string.len();
        eprintln!("\n{}", "#".repeat(length));
        eprintln!("{}", info_string);
        eprintln!("{}\n", "#".repeat(length));
        print_rows(file, None, format)?;
    }

    Ok(())
}