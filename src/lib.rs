extern crate byteorder;
extern crate failure;
#[macro_use]
extern crate failure_derive;

use failure::Error;
use std::io::Read;

use decoder::Decoder;

mod decoder;

#[derive(Fail, Debug)]
pub enum UncompressError {
    #[fail(display = "Invalid address (len {}, offset {}, start {})", len, offset, start)]
    InvalidAddress { len: usize, offset: i16, start: u16 },
    #[fail(display = "Output overflowed maximum size of 65536 bytes")]
    OutputSizeOverflow,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct BlockRef {
    offset: i16,
    length: u16,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Block {
    Literal(u8),
    Ref(BlockRef),
    EndOfFile,
}

pub fn decompress<R: Read>(input: R) -> Result<Vec<u8>, Error> {
    let mut decoder = Decoder::new(input)?;

    let mut output = vec![];

    loop {
        match decoder.decode_block()? {
            Block::EndOfFile => break,
            Block::Literal(value) => output.push(value),
            Block::Ref(BlockRef { offset, length }) => {
                let start = (output.len() as i16 + offset as i16) as u16 as usize;
                let end = start + length as usize;

                if start >= output.len() {
                    return Err(
                        UncompressError::InvalidAddress {
                            len: output.len(),
                            offset,
                            start: start as u16,
                        }.into(),
                    );
                }

                for i in start..end {
                    let value = output[i];
                    output.push(value);
                }
            }
        }

        if output.len() > 0x10000 {
            return Err(UncompressError::OutputSizeOverflow.into());
        }
    }

    Ok(output)
}

#[test]
fn test_decompress_literal_only() {
    let mut literals = vec![];
    let mut data: Vec<u8> = vec![];
    data.push(0b1111_1111); // 8 literal values
    data.push(0b0001_1111); // 5 literal values + EOF
    for value in 0..13 {
        data.push(value);
        literals.push(value);
    }
    data.push(0xff); // EOF byte

    data.push(0xff); // Dummy command block
    data.push(0xff);
    assert_eq!(decompress(data.as_slice()).unwrap(), literals);
}
