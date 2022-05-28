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
    fn new(value: u8) -> Self {
        if !(1..=10).contains(&value) {
            panic!("Invalid value.");
        }

        Self { value }
    }
    fn value(&self) -> u8 {
        self.value
    }
}

pub struct ArrayLength {
    value: u16,
}
impl ArrayLength {
    fn new(value: u16) -> Self {
        if !(1..=1024).contains(&value) {
            panic!("Invalid value.");
        }

        Self { value }
    }
    fn value(&self) -> u16 {
        self.value
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

    use reqwest::blocking::Client;
    use reqwest::header::HeaderName;

    let client = Client::new();
    let resp = client.get(url)
        .header(HeaderName::from_static("x-api-key"), "")
        .send()?
        .text()?;

    println!("{}", resp);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_test() {
        get_block(&DataType::Uint16, &ArrayLength::new(5));
    }
}
