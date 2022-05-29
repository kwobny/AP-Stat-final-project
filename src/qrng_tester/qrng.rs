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

macro_rules! unwrap_or_else {
    ( $variant:path, $obj:expr, $or_else:block ) => {
        if let $variant(x) = $obj {
            x
        } else {
            $or_else
        }
    };
}

fn qrng_numbers_from_json(json: Value) -> Result<QrngNumbers, &'static str> {
    let mut obj = unwrap_or_else!(Value::Object, json, {
        return Err("");
    });

    let success = if let Some(Value::Bool(x)) = obj.get("success") { x }
    else {
        return Err("");
    };
    if *success == false {
        return Err("");
    }

    let type_of_data = if let Some(Value::String(x)) = obj.remove("type") { x }
    else {
        return Err("");
    };
    let values = if let Some(Value::Array(x)) = obj.remove("data") { x }
    else {
        return Err("");
    };

    fn uint_vector<T: TryFrom<u64>>(values: &Vec<Value>) -> Result<Vec<T>, &'static str> {
        let mut ret_vec: Vec<T> = Vec::new();
        for val in values {
            let number = unwrap_or_else!(Value::Number, val, {
                return Err("");
            });
            let number = unwrap_or_else!(Some, number.as_u64(), {
                return Err("");
            });
            ret_vec.push(unwrap_or_else!(Ok, number.try_into(), {
                return Err("");
            }));
        }
        Ok(ret_vec)
    }

    match &type_of_data[..] {
        "uint8" => match uint_vector(&values) {
            Ok(x) => Ok(QrngNumbers::Uint8(x)),
            Err(x) => Err(x),
        }
        "uint16" => match uint_vector(&values) {
            Ok(x) => Ok(QrngNumbers::Uint16(x)),
            Err(x) => Err(x),
        }
        "hex8"|"hex16" => {
            let mut ret_vec: Vec<String> = Vec::new();
            for val in values {
                let hex = unwrap_or_else!(Value::String, val, {
                    return Err("");
                });
                ret_vec.push(hex);
            }
            Ok(QrngNumbers::Hex(ret_vec))
        },
        _ => Err(""),
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
