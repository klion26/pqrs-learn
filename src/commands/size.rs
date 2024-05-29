use std::path::PathBuf;

use clap::{Parser};
use log::debug;

use crate::errors::PQRSError;
use crate::errors::PQRSError::FileNotFound;
use crate::utils::{check_path_present, get_pretty_size, get_size, open_file};

#[derive(Parser, Debug)]
pub struct SizeCommandArgs {
    files: Vec<PathBuf>,
    #[clap(short, long)]
    compressed: bool,
    #[clap(short, long)]
    pretty: bool,
}

pub(crate) fn execute(opts: SizeCommandArgs) -> Result<(), PQRSError> {
    debug!("The file names to read are: {:?}", opts.files);

    for file_name in &opts.files {
        if !check_path_present(file_name) {
            return Err(FileNotFound(file_name.to_path_buf()));
        }
    }

    println!("Size in bytes:");
    for file_name in &opts.files {
        let file = open_file(file_name)?;
        let size_info = get_size(file)?;

        println!();
        println!("File Name: {}", file_name.display());

        if !opts.compressed {
            if opts.pretty {
                println!("Uncompressed size: {}", get_pretty_size(size_info.0));
            } else {
                println!("Uncompressed size: {}", size_info.0);
            }
        } else {
            if opts.pretty {
                println!("compressed size: {}", get_pretty_size(size_info.1));
            } else {
                println!("compressed size: {}", size_info.1);
            }
        }
        println!();
    }

    Ok(())
}
