use byteorder::{LittleEndian, ReadBytesExt};
use failure::Error;
use std::io::Read;

use {Block, BlockRef};

#[derive(Clone, Debug)]
pub struct Decoder<R: Read> {
    input: R,
    cmd_block: CommandBlock,
}

impl<R: Read> Decoder<R> {
    pub fn new(mut input: R) -> Result<Decoder<R>, Error> {
        let cmd_block = CommandBlock::read_from(&mut input)?;
        Ok(Decoder {
            input: input,
            cmd_block: cmd_block,
        })
    }
    pub fn decode_block(&mut self) -> Result<Block, Error> {
        if self.next_cmd_bit()? {
            let value = self.input.read_u8()?;
            Ok(Block::Literal(value))
        } else {
            let is_big_ref = self.next_cmd_bit()?;
            let offset_l = self.input.read_u8()?;

            if is_big_ref {
                let block_ref = self.decode_big_ref(offset_l)?;
                Ok(Block::Ref(block_ref))
            } else if self.next_cmd_bit()? {
                let block_ref = self.decode_small_ref(offset_l)?;
                Ok(Block::Ref(block_ref))
            } else if offset_l == 0xff {
                Ok(Block::EndOfFile)
            } else {
                Ok(Block::Ref(BlockRef {
                    offset: (0xff00 | u16::from(offset_l)) as i16,
                    length: 2,
                }))
            }
        }
    }
    fn next_cmd_bit(&mut self) -> Result<bool, Error> { Ok(self.next_cmd_bits(1)? == 0x01) }
    fn next_cmd_bits(&mut self, bits: usize) -> Result<u8, Error> {
        let mut result = 0;
        for _ in 0..bits {
            // Order of shifting first and then fetching more is important!
            let bit = self.cmd_block.shift_bit();
            if self.cmd_block.is_empty() {
                self.cmd_block = CommandBlock::read_from(&mut self.input)?;
            }
            result = (result << 1) | bit;
        }
        Ok(result)
    }
    fn decode_big_ref(&mut self, offset_l: u8) -> Result<BlockRef, Error> {
        let mut offset_h = 0xfe | self.next_cmd_bits(1)?;
        if !self.next_cmd_bit()? {
            let mut tmp = 2u8;

            for _ in 0..3 {
                if self.next_cmd_bit()? {
                    break;
                }
                offset_h = (offset_h << 1) | self.next_cmd_bits(1)?;
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
        let bits = self.next_cmd_bits(3)?;
        let offset_h = (0xf8 | bits).wrapping_sub(1);
        Ok(BlockRef {
            offset: ((u16::from(offset_h) << 8) | u16::from(offset_l)) as i16,
            length: 2,
        })
    }
    fn decode_length(&mut self) -> Result<u16, Error> {
        for i in 3..7 {
            if self.next_cmd_bit()? {
                return Ok(i as u16);
            }
        }

        if self.next_cmd_bit()? {
            let bit = self.next_cmd_bits(1)?;
            Ok(u16::from(7 + bit))
        } else if self.next_cmd_bit()? {
            let value = self.input.read_u8()?;
            Ok(u16::from(value).wrapping_add(0x11))
        } else {
            let bits = self.next_cmd_bits(3)?;
            Ok(u16::from(bits.wrapping_add(0x09)))
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct CommandBlock {
    data: u16,
    count: usize,
}

impl CommandBlock {
    fn read_from<R: Read>(input: &mut R) -> Result<CommandBlock, Error> {
        Ok(CommandBlock {
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
