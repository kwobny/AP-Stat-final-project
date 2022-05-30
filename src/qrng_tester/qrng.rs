use reqwest::blocking::Client;
use reqwest::header::HeaderName;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use serde_json::Value;

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
            panic!("Invalid value.");
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
            panic!("Invalid value.");
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

fn qrng_numbers_from_json(json: Value) -> Result<QrngNumbers, &'static str> {
    const INVALID_JSON: Result<QrngNumbers, &str> = Err("Invalid json");
    const UNSUCCESSFUL_REQUEST: Result<QrngNumbers, &str> = Err("Unsuccessful request");
    const NUMBER_OUT_OF_RANGE: Result<QrngNumbers, &str> = Err("Number out of range");
    const UNKNOWN_DATA_TYPE: Result<QrngNumbers, &str> = Err("Unknown data type");

    let mut obj = unwrap_or_error!(json, Value::Object, INVALID_JSON);

    let success = unwrap_or_error!(
        obj.get("success"),
        Some(Value::Bool(x)) => x,
        INVALID_JSON,
    );
    if *success == false {
        return UNSUCCESSFUL_REQUEST;
    }

    let type_of_data = unwrap_or_error!(
        obj.remove("type"),
        Some(Value::String(x)) => x,
        INVALID_JSON,
    );
    let values = unwrap_or_error!(
        obj.remove("data"),
        Some(Value::Array(x)) => x,
        INVALID_JSON,
    );

    fn uint_vector<T>(values: &Vec<Value>) -> Result<QrngNumbers, &'static str>
    where
        T: TryFrom<u64>,
        Vec<T>: Into<QrngNumbers>,
    {
        let mut ret_vec: Vec<T> = Vec::new();
        for val in values {
            let number = unwrap_or_error!(val, Value::Number, INVALID_JSON);
            let number = unwrap_or_error!(number.as_u64(), Some, INVALID_JSON);
            ret_vec.push(unwrap_or_error!(number.try_into(), Ok, NUMBER_OUT_OF_RANGE));
        }
        Ok(ret_vec.into())
    }

    match &type_of_data[..] {
        "uint8" => uint_vector::<u8>(&values),
        "uint16" => uint_vector::<u16>(&values),
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

pub fn get_block(data_type: &DataType, array_length: &ArrayLength) -> Result<(), Box<dyn std::error::Error>> {
    fn get_type_str(data_type: &DataType) -> &str {
        match data_type {
            DataType::Uint8 => "uint8",
            DataType::Uint16 => "uint16",
            DataType::Hex8(_) => "hex8",
            DataType::Hex16(_) => "hex16",
        }
    }

    let url = match data_type {
        DataType::Uint8 | DataType::Uint16 => format!(
            "https://api.quantumnumbers.anu.edu.au?length={}&type={}",
            array_length.value(),
            get_type_str(data_type),
        ),
        DataType::Hex8(block_size) | DataType::Hex16(block_size) => format!(
            "https://api.quantumnumbers.anu.edu.au?length={}&type={}&size={}",
            array_length.value(),
            get_type_str(data_type),
            block_size.value(),
        ),
    };

    let client = Client::new();
    let resp: Value = client.get(url)
        .header(HeaderName::from_static("x-api-key"), get_api_key())
        .send()?
        .json()?;

    println!("{:?}", resp);

    Ok(())
}

fn get_api_key() -> String {
    let mut file = File::open(&Path::new(".env/qrng_api_key.txt"))
        .expect("Error opening qrng api key file.");
    let mut key = String::new();
    file.read_to_string(&mut key).expect("Error reading qrng api key file into variable");
    key
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
