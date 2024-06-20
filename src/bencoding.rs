use std::collections::HashMap;

/// Possible errors that can occur during bencode parsing.
#[derive(PartialEq, Eq, Debug)]
pub enum ParseError {
    /// Indicates an invalid byte was encountered at the given position.
    InvalidByte(usize),
    /// Indicates the end of the stream was reached unexpectedly.
    UnexpectedEndOfStream,
    /// Indicates the stream contained invalid UTF-8.
    InvalidUtf8,
}

/// Represents a bencode value.
#[derive(PartialEq, Eq, Debug)]
pub enum Bencode {
    /// Represents an integer value.
    Int(i64),
    /// Represents a string value.
    Str(Vec<u8>),
    /// Represents a list of bencode values.
    List(Vec<Bencode>),
    /// Represents a dictionary of bencode values.
    Dict(HashMap<String, Bencode>),
}

impl Bencode {
    /// Returns the integer value if this is a `Bencode::Int`.
    ///
    /// # Returns
    ///
    /// An `Option` containing the integer value or `None` if this is not a `Bencode::Int`.
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Bencode::Int(i) => Some(*i),
            _ => None,
        }
    }

    /// Returns the string value if this is a `Bencode::Str`.
    ///
    /// # Returns
    ///
    /// An `Option` containing the string value or `None` if this is not a `Bencode::Str`.
    pub fn as_bytes(&self) -> Option<&Vec<u8>> {
        match self {
            Bencode::Str(s) => Some(s),
            _ => None,
        }
    }

    /// Returns the list value if this is a `Bencode::List`.
    ///
    /// # Returns
    ///
    /// An `Option` containing the list value or `None` if this is not a `Bencode::List`.
    pub fn as_list(&self) -> Option<&Vec<Bencode>> {
        match self {
            Bencode::List(l) => Some(l),
            _ => None,
        }
    }

    /// Returns the dictionary value if this is a `Bencode::Dict`.
    ///
    /// # Returns
    ///
    /// An `Option` containing the dictionary value or `None` if this is not a `Bencode::Dict`.
    pub fn as_dict(&self) -> Option<&HashMap<String, Bencode>> {
        match self {
            Bencode::Dict(d) => Some(d),
            _ => None,
        }
    }
}

struct Decoder<'a> {
    stream: &'a [u8],
    pos: usize,
}

impl<'a> Decoder<'a> {
    pub fn new(stream: &'a [u8]) -> Self {
        Decoder { stream, pos: 0 }
    }

    pub fn decode(&mut self) -> Result<Bencode, ParseError> {
        self.parse()
    }

    fn parse(&mut self) -> Result<Bencode, ParseError> {
        if self.pos >= self.stream.len() {
            return Err(ParseError::UnexpectedEndOfStream);
        }

        let curr_byte = self.stream[self.pos];
        match curr_byte {
            b'd' => self.parse_dict(),
            b'l' => self.parse_list(),
            b'i' => self.parse_int(),
            b'0'..=b'9' => self.parse_str(),
            _ => Err(ParseError::InvalidByte(self.pos)),
        }
    }

    fn parse_list(&mut self) -> Result<Bencode, ParseError> {
        let mut list: Vec<Bencode> = Vec::new();
        self.pos += 1; // Skip the 'l'
        while self.stream[self.pos] != b'e' {
            let parsed = self.parse()?;
            list.push(parsed);
        }
        self.pos += 1; // Skip the 'e'
        Ok(Bencode::List(list))
    }

    fn parse_dict(&mut self) -> Result<Bencode, ParseError> {
        let mut dict: HashMap<String, Bencode> = HashMap::new();
        self.pos += 1; // Skip the 'd'
        while self.stream[self.pos] != b'e' {
            let key = match self.parse_str()? {
                Bencode::Str(s) => s,
                _ => return Err(ParseError::InvalidByte(self.pos)),
            };
            let value = self.parse()?;
            let key = match String::from_utf8(key) {
                Ok(s) => s,
                Err(_) => return Err(ParseError::InvalidUtf8),
            };
            dict.insert(key, value);
        }
        self.pos += 1; // Skip the 'e'
        Ok(Bencode::Dict(dict))
    }

    fn parse_str(&mut self) -> Result<Bencode, ParseError> {
        let mut str_size: usize = 0;
        while self.stream[self.pos] != b':' {
            if self.stream[self.pos].is_ascii_digit() {
                str_size = str_size * 10 + (self.stream[self.pos] - b'0') as usize;
            } else {
                return Err(ParseError::InvalidByte(self.pos));
            }
            self.pos += 1;
        }
        self.pos += 1;

        if self.pos + str_size > self.stream.len() {
            return Err(ParseError::UnexpectedEndOfStream);
        }

        let s = &self.stream[self.pos..self.pos + str_size];
        self.pos += str_size;

        Ok(Bencode::Str(s.to_vec()))
    }

    fn parse_int(&mut self) -> Result<Bencode, ParseError> {
        self.pos += 1; // Skip the 'i'

        let mut is_negative = false;
        if self.stream[self.pos] == b'-' {
            is_negative = true;
            self.pos += 1;
        }

        let mut curr_int: i64 = 0;
        while self.stream[self.pos] != b'e' {
            if self.stream[self.pos].is_ascii_digit() {
                curr_int = curr_int * 10 + (self.stream[self.pos] - b'0') as i64;
            } else {
                return Err(ParseError::InvalidByte(self.pos));
            }
            self.pos += 1;
        }

        self.pos += 1;

        if is_negative {
            curr_int = -curr_int;
        }

        Ok(Bencode::Int(curr_int))
    }
}

/// Decodes a bencode-encoded byte stream.
///
/// # Arguments
///
/// * `stream` - A byte slice containing the bencode-encoded data.
///
/// # Returns
///
/// A `Result` containing the decoded `Bencode` value or a `ParseError`.
pub fn decode(stream: &[u8]) -> Result<Bencode, ParseError> {
    let mut decoder = Decoder::new(stream);
    decoder.decode()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    /// Helper function to read a file into a byte vector.
    fn read_file(path: &str) -> Vec<u8> {
        let mut file = std::fs::File::open(path).unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();
        buffer
    }

    #[test]
    fn test_decode_str() {
        let mut decoder = Decoder::new(b"4:spam");
        let result = decoder.decode().unwrap();
        assert_eq!(result, Bencode::Str("spam".into()));
    }

    #[test]
    fn test_decode_invalid_str() {
        let invalid_utf8: Vec<u8> = vec![0xF0, 0x28, 0x8C, 0xBC];
        let mut decoder = Decoder::new(&invalid_utf8);
        let result = decoder.decode();
        assert_eq!(result, Err(ParseError::InvalidByte(0)));
    }

    #[test]
    fn test_decode_int() {
        let mut decoder = Decoder::new(b"i42e");
        let result = decoder.decode().unwrap();
        assert_eq!(result, Bencode::Int(42));
    }

    #[test]
    fn test_decode_negative_int() {
        let mut decoder = Decoder::new(b"i-42e");
        let result = decoder.decode().unwrap();
        assert_eq!(result, Bencode::Int(-42));
    }

    #[test]
    fn test_decode_invalid_int() {
        let mut decoder = Decoder::new(b"iae");
        let result = decoder.decode();
        assert_eq!(result, Err(ParseError::InvalidByte(1)));
    }

    #[test]
    fn test_decode_list() {
        let mut decoder = Decoder::new(b"l4:spam4:eggse");
        let result = decoder.decode().unwrap();
        assert_eq!(
            result,
            Bencode::List(vec![
                Bencode::Str("spam".into()),
                Bencode::Str("eggs".into())
            ])
        );
    }

    #[test]
    fn test_decode_dict() {
        let mut decoder = Decoder::new(b"d3:cow3:moo4:spam4:eggse");
        let result = decoder.decode().unwrap();
        let mut expected_dict = HashMap::new();
        expected_dict.insert("cow".to_string(), Bencode::Str("moo".into()));
        expected_dict.insert("spam".to_string(), Bencode::Str("eggs".into()));
        assert_eq!(result, Bencode::Dict(expected_dict));
    }

    #[test]
    fn test_decode_torrent() {
        // Read the file into a byte vector
        let path = "test_data/linuxmint.torrent";
        let torrent_stream = read_file(path);

        // Decode the torrent file (assuming you have a `decode` function handling Bencoding)
        let result = decode(&torrent_stream).expect("Failed to decode");

        // Check for required keys in the top-level dictionary
        let required_keys = [
            "announce",
            "created by",
            "creation date",
            "encoding",
            "info",
        ];
        for key in required_keys {
            assert!(result.as_dict().unwrap().contains_key(key));
        }

        // Check for required keys in the "info" dictionary
        let info_dict = result
            .as_dict()
            .unwrap()
            .get("info")
            .unwrap()
            .as_dict()
            .unwrap();
        let required_keys = ["name", "piece length", "pieces"];
        for key in required_keys {
            assert!(info_dict.contains_key(key));
        }
    }
}
