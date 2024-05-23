use std::fmt;
use std::fmt::Formatter;
use clap::{App, Arg, ArgMatches, SubCommand};
use log::debug;
use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::schema::printer::{print_file_metadata, print_parquet_metadata};
use crate::errors::PQRSError;
use crate::errors::PQRSError::FileNotFound;
use crate::utils::{check_path_present, open_file};
use crate::command::PQRSCommand;

pub struct SchemaCommand<'a> {
    file_names: Vec<&'a str>,
    use_detailed: bool,
}

impl<'a> SchemaCommand<'a> {
    pub(crate) fn command() -> App<'static, 'static> {
        SubCommand::with_name("schema")
            .about("Prints the schema of Parquet file(s)")
            .arg(
                Arg::with_name("files")
                    .index(1)
                    .multiple(true)
                    .value_name("FILES")
                    .value_delimiter(" ")
                    .required(true)
                    .help("Parquet files to read"),
            )
            .arg(
                Arg::with_name("detailed")
                    .long("detailed")
                    .short("D")
                    .takes_value(false)
                    .required(false)
                    .help("Enable printing full file meatdata"),
            )
    }
    
    pub(crate) fn new(matchers: &'a ArgMatches<'a>) -> Self {
        Self {
            file_names: matchers.values_of("files").unwrap().collect(),
            use_detailed: matchers.is_present("detailed"),
        }
    }
}


impl<'a> PQRSCommand for SchemaCommand<'a> {
    fn execute(&self) -> Result<(), PQRSError> {
        debug!("{:#?}", self);


        debug!("The file names to read are: {:#?}", &self.file_names);
        debug!("Print Detailed output: {:#?}", &self.use_detailed);

        for file_name in &self.file_names {
            if !check_path_present(*file_name) {
                return Err(FileNotFound(String::from(*file_name)))
            }
        }

        for file_name in &self.file_names {
            let file = open_file(file_name)?;
            match  SerializedFileReader::new(file) {
                Ok(parquet_reader) => {
                    let metadata = parquet_reader.metadata();
                    println!("Metadata for file: {}", file_name);
                    println!();
                    if self.use_detailed {
                        print_parquet_metadata(&mut std::io::stdout(), metadata);
                    } else {
                        print_file_metadata(&mut std::io::stdout(), metadata.file_metadata())
                    }
                }
                Err(e) => {return Err(PQRSError::ParquetError(e))}
            }
        }

        Ok(())
    }
}

impl<'a> fmt::Debug for SchemaCommand<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "The file names to read are: {}", &self.file_names.join(", "))?;
        writeln!(f, "Print detailed output: {}", &self.use_detailed)?;

        Ok(())
    }
}