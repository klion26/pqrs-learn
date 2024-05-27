use std::io;
use parquet::errors::ParquetError;
use std::num::ParseIntError;
use std::path::PathBuf;
use thiserror::Error;
use arrow::error::ArrowError;
use serde_json::Error as SerdeJsonError;
use std::string::FromUtf8Error;
use std::io::{BufWriter, IntoInnerError};

#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum PQRSError {
    #[error("File {0} not found, please check if it exists")]
    FileNotFound(PathBuf),
    #[error("Could not open file: {0}")]
    CouldNotOpenFile(PathBuf),
    #[error("File already exists: {0}")]
    FileExists(PathBuf),
    #[error("Could not read Parquet File")]
    ParquetError(#[from] ParquetError),
    #[error("Unable to read given integer")]
    UnableToReadNumber(#[from] ParseIntError),
    #[error("Unable to process file")]
    UnableProcessFile(#[from] io::Error),
    #[error("Unable to read/write arrow data")]
    ArrowReadWriteError(#[from] ArrowError),
    #[error("Unsupported operation")]
    UnsupportedOperation(),
    #[error("Could not convert to/from json")]
    SerdeJsonError(#[from] SerdeJsonError),
    #[error("Could not create string from UTF8 bytes")]
    UTF8ConvertError(#[from] FromUtf8Error),
    #[error("Could not read/write to buffer")]
    BufferWriteError(#[from] IntoInnerError<BufWriter<Vec<u8>>>)
}