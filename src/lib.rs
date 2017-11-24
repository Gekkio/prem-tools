extern crate byteorder;
extern crate failure;
#[macro_use]
extern crate failure_derive;

use failure::Error;
use std::io::Read;

use decoder::{DecodedBlock, Decoder};

mod decoder;

#[derive(Fail, Debug)]
pub enum PremError {
    #[fail(display = "Invalid address (len {}, offset {}, start {})", len, offset, start)]
    InvalidAddress { len: usize, offset: i16, start: u16 },
    #[fail(display = "Output overflowed maximum size of 65536 bytes")]
    OutputSizeOverflow,
}

#[derive(Copy, Clone, Debug)]
pub struct BlockRef {
    offset: i16,
    length: u16,
}

pub fn uncompress<R: Read>(input: R) -> Result<Vec<u8>, Error> {
    let mut decoder = Decoder::new(input)?;

    let mut output = vec![];

    loop {
        match decoder.decode_block()? {
            DecodedBlock::EndOfFile => break,
            DecodedBlock::Literal(value) => output.push(value),
            DecodedBlock::Ref(BlockRef { offset, length }) => {
                let start = (output.len() as i16 + offset as i16) as u16 as usize;
                let end = start + length as usize;

                if start >= output.len() {
                    return Err(
                        PremError::InvalidAddress {
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
            return Err(PremError::OutputSizeOverflow.into());
        }
    }

    Ok(output)
}

#[test]
fn test_literal_only() {
    let mut literals = vec![];
    let mut data: Vec<u8> = vec![];
    data.push(0b1111_1111); // 8 literal values
    data.push(0b0001_1111); // 5 literal values + EOF
    for value in 0..13 {
        data.push(value);
        literals.push(value);
    }
    data.push(0xff); // EOF byte
    assert_eq!(uncompress(data.as_slice()).unwrap(), literals);
}
