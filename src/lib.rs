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
    #[fail(display = "Invalid address (offset {}, start {})", _0, _1)]
    InvalidAddress(i16, u16),
    #[fail(display = "Output overflowed maximum size of 65535 bytes")]
    OutputSizeOverflow,
}

#[derive(Copy, Clone, Debug)]
pub struct BlockRef {
    offset: i16,
    length: u16,
}

pub fn decode<R: Read>(input: R) -> Result<Vec<u8>, Error> {
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
                    return Err(PremError::InvalidAddress(offset, start as u16).into());
                }

                for i in start..end {
                    let value = output[i];
                    output.push(value);
                }
            }
        }

        if output.len() > 0xffff {
            return Err(PremError::OutputSizeOverflow.into());
        }
    }

    Ok(output)
}
