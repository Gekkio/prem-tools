use byteorder::{LittleEndian, ReadBytesExt};
use failure::Error;
use std::io::Read;

use BlockRef;

#[derive(Copy, Clone, Debug)]
pub enum DecodedBlock {
    Literal(u8),
    Ref(BlockRef),
    EndOfFile,
}

#[derive(Clone, Debug)]
pub struct Decoder<R: Read> {
    input: R,
    cmds: Commands,
}

impl<R: Read> Decoder<R> {
    pub fn new(mut input: R) -> Result<Decoder<R>, Error> {
        let cmds = Commands::read_from(&mut input)?;
        Ok(Decoder {
            input: input,
            cmds: cmds,
        })
    }
    pub fn decode_block(&mut self) -> Result<DecodedBlock, Error> {
        if self.cmd_bool()? {
            let value = self.input.read_u8()?;
            Ok(DecodedBlock::Literal(value))
        } else {
            let is_big_ref = self.cmd_bool()?;
            let offset_l = self.input.read_u8()?;

            if is_big_ref {
                let block_ref = self.decode_big_ref(offset_l)?;
                Ok(DecodedBlock::Ref(block_ref))
            } else if self.cmd_bool()? {
                let block_ref = self.decode_small_ref(offset_l)?;
                Ok(DecodedBlock::Ref(block_ref))
            } else if offset_l == 0xff {
                Ok(DecodedBlock::EndOfFile)
            } else {
                Ok(DecodedBlock::Ref(BlockRef {
                    offset: (0xff00 | u16::from(offset_l)) as i16,
                    length: 2,
                }))
            }
        }
    }
    fn cmd_bool(&mut self) -> Result<bool, Error> { Ok(self.cmd_u8(1)? == 0x01) }
    fn cmd_u8(&mut self, bits: usize) -> Result<u8, Error> {
        let mut result = 0;
        for _ in 0..bits {
            let bit = self.cmds.shift_bit();
            if self.cmds.is_empty() {
                self.cmds = Commands::read_from(&mut self.input)?;
            }
            result = (result << 1) | bit;
        }
        Ok(result)
    }
    fn decode_big_ref(&mut self, offset_l: u8) -> Result<BlockRef, Error> {
        let mut offset_h = 0xfe | self.cmd_u8(1)?;
        if !self.cmd_bool()? {
            let mut tmp = 2u8;

            for _ in 0..3 {
                if self.cmd_bool()? {
                    break;
                }
                offset_h = (offset_h << 1) | self.cmd_u8(1)?;
                tmp <<= 1;
            }
            offset_h = (!(tmp.wrapping_sub(offset_h))).wrapping_add(1);
        }

        Ok(BlockRef {
            offset: ((u16::from(offset_h) << 8) | u16::from(offset_l)) as i16,
            length: self.decode_length()?,
        })
    }
    fn decode_small_ref(&mut self, offset_l: u8) -> Result<BlockRef, Error> {
        let bits = self.cmd_u8(3)?;
        let offset_h = (0xf8 | bits).wrapping_sub(1);
        Ok(BlockRef {
            offset: ((u16::from(offset_h) << 8) | u16::from(offset_l)) as i16,
            length: 2,
        })
    }
    fn decode_length(&mut self) -> Result<u16, Error> {
        for i in 0..4 {
            if self.cmd_bool()? {
                return Ok((3 + i) as u16);
            }
        }

        if self.cmd_bool()? {
            let bit = self.cmd_u8(1)?;
            Ok(u16::from(7 + bit))
        } else if self.cmd_bool()? {
            let value = self.input.read_u8()?;
            Ok(u16::from(value).wrapping_add(0x11))
        } else {
            let bits = self.cmd_u8(3)?;
            Ok(u16::from(bits.wrapping_add(0x09)) & 0x00ff)
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct Commands {
    data: u16,
    count: usize,
}

impl Commands {
    fn read_from<R: Read>(input: &mut R) -> Result<Commands, Error> {
        Ok(Commands {
            data: input.read_u16::<LittleEndian>()?,
            count: 16,
        })
    }
    fn shift_bit(&mut self) -> u8 {
        let bit = (self.data & 0x01) as u8;
        self.count -= 1;
        self.data >>= 1;
        bit
    }
    fn is_empty(&self) -> bool { self.count == 0 }
}
