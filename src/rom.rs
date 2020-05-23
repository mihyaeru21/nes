use std::{error::Error, io::Read, result::Result};

#[derive(Debug, PartialEq, Eq)]
pub struct Rom {
    pub program: Vec<u8>,
    pub character: Vec<u8>,
}

impl Rom {
    pub fn load<R: Read>(reader: &mut R) -> Result<Self, Box<dyn Error>> {
        let mut header = [0; 16];
        reader.read_exact(&mut header)?;
        if header[0] != 0x4e || header[1] != 0x45 || header[2] != 0x53 || header[3] != 0x1a {
            return Err("Invalid header constant.".into());
        }
        let mut program: Vec<u8> = vec![0; (header[4] as usize) * 0x4000];
        let mut character: Vec<u8> = vec![0; (header[5] as usize) * 0x2000];
        reader.read_exact(&mut program)?;
        reader.read_exact(&mut character)?;

        Ok(Self { program, character })
    }
}

#[cfg(test)]
mod test {
    use super::Rom;
    use std::{
        fs::File,
        io::{BufReader, Cursor},
    };

    #[test]
    fn test_load() {
        let mut reader = BufReader::new(File::open("./rom/sample1.nes").unwrap());
        let _ = Rom::load(&mut reader).unwrap();
    }

    #[test]
    fn test_load_invalid_header() {
        let mut reader = Cursor::new(vec![
            0xaa, 0x22, 0x11, 0xff, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0xaa, 0x22, 0x11, 0xff,
            0x00, 0x01,
        ]);
        let err = Rom::load(&mut reader).unwrap_err();
        assert_eq!("Invalid header constant.", err.to_string());
    }
}
