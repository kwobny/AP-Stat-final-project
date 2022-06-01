//! # QRNG API
//! 
//! This module provides a rust api for requesting random numbers
//! from the ANU QRNG website.
//! Internally, it interacts with the ANU QRNG website's own json web api.
//!
//! The module aims to mirror the ANU QRNG website's json api as
//! directly and similarly as possible.

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use std::fmt::{Display, Formatter, self};

use std::borrow::Borrow;

use serde_json::Value;

use reqwest::blocking::Client;
use reqwest::header::HeaderName;

pub enum DataType {
    Uint8,
    Uint16,
    Hex8(BlockSize),
    Hex16(BlockSize),
}

pub struct BlockSize {
    value: u8,
}
impl BlockSize {
    pub fn new(value: u8) -> Self {
        if !(1..=10).contains(&value) {
            panic!("block size is not between 1 and 10 (inclusive)");
        }

        Self { value }
    }
    pub fn value(&self) -> u8 {
        self.value
    }
}

pub struct ArrayLength {
    value: u16,
}
impl ArrayLength {
    pub fn new(value: u16) -> Self {
        if !(1..=1024).contains(&value) {
            panic!("array length is not between 1 and 1024 (inclusive)");
        }

        Self { value }
    }
    pub fn value(&self) -> u16 {
        self.value
    }
}

pub enum QrngNumbers {
    Uint8(Vec<u8>),
    Uint16(Vec<u16>),
    Hex(Vec<String>),
}
impl From<Vec<u8>> for QrngNumbers {
    fn from(vect: Vec<u8>) -> Self {
        QrngNumbers::Uint8(vect)
    }
}
impl From<Vec<u16>> for QrngNumbers {
    fn from(vect: Vec<u16>) -> Self {
        QrngNumbers::Uint16(vect)
    }
}

macro_rules! unwrap_or_error {
    ( $obj:expr, $variant:path, $error:expr $(,)? ) => {
        if let $variant(x) = $obj {
            x
        } else {
            return $error;
        }
    };
    ( $obj:expr, $pattern:pat_param => $target:ident, $error:expr $(,)? ) => {
        if let $pattern = $obj {
            $target
        } else {
            return $error;
        }
    };
}

#[derive(Debug)]
struct GenericError<'a>(&'a str, Option<Box<dyn std::error::Error + 'static>>);
impl Display for GenericError<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for GenericError<'_> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self.1 {
            Some(ref x) => Some(x.borrow()),
            None => None,
        }
    }
}

fn qrng_numbers_from_json(json: Value) -> Result<QrngNumbers, GenericError<'static>> {
    type ReturnType = Result<QrngNumbers, GenericError<'static>>;
    macro_rules! err_from_str {
        ( $s:expr ) => {
            Err(GenericError($s, None))
        };
    }

    const INVALID_JSON: ReturnType = err_from_str!("invalid json response");
    const UNSUCCESSFUL_REQUEST: ReturnType = err_from_str!("json success property is false");
    const INVALID_NUMBER: ReturnType = err_from_str!("a number in the data does not conform to the type implied by the type property");
    const UNKNOWN_DATA_TYPE: ReturnType = err_from_str!("unrecognized data type in type property");

    let mut obj = unwrap_or_error!(json, Value::Object, INVALID_JSON);

    let success = unwrap_or_error!(
        obj.get("success"),
        Some(Value::Bool(x)) => x,
        INVALID_JSON,
    );
    if *success == false {
        return UNSUCCESSFUL_REQUEST;
    }

    let values = unwrap_or_error!(
        obj.remove("data"),
        Some(Value::Array(x)) => x,
        INVALID_JSON,
    );
    let type_of_data = unwrap_or_error!(
        obj.get("type"),
        Some(Value::String(x)) => x,
        INVALID_JSON,
    );

    fn uint_vector<T>(values: Vec<Value>) -> ReturnType
    where
        T: TryFrom<u64>,
        Vec<T>: Into<QrngNumbers>,
    {
        let mut ret_vec: Vec<T> = Vec::new();
        for val in values {
            let number = unwrap_or_error!(val, Value::Number, INVALID_NUMBER);
            let number = unwrap_or_error!(number.as_u64(), Some, INVALID_NUMBER);
            ret_vec.push(unwrap_or_error!(number.try_into(), Ok, INVALID_NUMBER));
        }
        Ok(ret_vec.into())
    }

    match &type_of_data[..] {
        "uint8" => uint_vector::<u8>(values),
        "uint16" => uint_vector::<u16>(values),
        "hex8"|"hex16" => {
            let mut ret_vec: Vec<String> = Vec::new();
            for val in values {
                let hex = unwrap_or_error!(val, Value::String, INVALID_JSON);
                ret_vec.push(hex);
            }
            Ok(QrngNumbers::Hex(ret_vec))
        },
        _ => UNKNOWN_DATA_TYPE,
    }
}

pub fn get_block(data_type: &DataType, array_length: &ArrayLength) -> Result<QrngNumbers, Box<dyn std::error::Error>> {
    let type_str = match data_type {
        DataType::Uint8 => "uint8",
        DataType::Uint16 => "uint16",
        DataType::Hex8(_) => "hex8",
        DataType::Hex16(_) => "hex16",
    };

    let mut url = format!(
        "https://api.quantumnumbers.anu.edu.au?length={}&type={}",
        array_length.value(),
        type_str,
    );
    if let DataType::Hex8(block_size) | DataType::Hex16(block_size) = data_type {
        url += &format!("&size={}", block_size.value());
    }

    let client = Client::new();
    let resp: Value = client.get(url)
        .header(HeaderName::from_static("x-api-key"), get_api_key()?)
        .send()?
        .json()?;

    let parsed_numbers = qrng_numbers_from_json(resp);

    match parsed_numbers {
        Ok(x) => Ok(x),
        Err(x) => Err(Box::new(x)),
    }
}

fn get_api_key() -> Result<String, Box<dyn std::error::Error>> {
    let mut file = match File::open(&Path::new(".env/qrng_api_key.txt")) {
        Ok(x) => x,
        Err(x) => return Err(Box::new(GenericError("failed to open qrng api key file", Some(Box::new(x))))),
    };
    let mut key = String::new();
    if let Err(x) = file.read_to_string(&mut key) {
        return Err(Box::new(GenericError("failed to read qrng api key file into variable", Some(Box::new(x)))));
    }
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_test() -> Result<(), Box<dyn std::error::Error>> {
        // get_block(&DataType::Hex16(BlockSize::new(4)), &ArrayLength::new(5))?;
        let resp = r#"{"success": true, "type": "uint16", "length": "5", "data": [49840, 24264, 44448, 26560, 22008]}"#;
        let resp: Value = serde_json::from_str(resp)?;
        println!("{:?}", resp);
        Ok(())
    }
}
