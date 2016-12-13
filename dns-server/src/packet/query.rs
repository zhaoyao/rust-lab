use std::string::String;
use super::buffer::Buffer;
use super::error::*;
use super::record_type::RecordType;
use super::class::DnsClass;

#[derive(Debug)]
pub struct Question {
    q_name: Vec<String>,
    q_type: RecordType,
    q_class: DnsClass,
}


pub fn read_labels(b: &mut Buffer) -> Result<Vec<String>> {
    let c = 0;
    let mut v = Vec::new();
    loop {
        if c > 255 {
            return Err(ErrorKind::InvalidMessage("label too long".to_string()).into())
        }

        let mut l = b.u8()?;
        l = l & 0b0011_1111; // TODO: ref
        if l == 0 {
            break
        }

        v.push(b.string(l as usize)?);
    }
    Ok(v)
}

impl Question {
    pub fn read(b: &mut Buffer) -> Result<Self> {
        Ok(Question{ q_name: read_labels(b)?, q_type: RecordType::from_u16(b.u16()?)?, q_class: DnsClass::from_u16(b.u16()?)?})
    }
}