use super::error::*;

#[derive(Debug)]
pub enum DnsClass {
    IN,
    CS,
    CH,
    HS,
    Any
}

impl DnsClass {

    pub fn from_u16(i: u16) -> Result<DnsClass> {
        match i {
            1 => Ok(DnsClass::IN),
            2 => Ok(DnsClass::CS),
            3 => Ok(DnsClass::CH),
            4 => Ok(DnsClass::HS),
            255 => Ok(DnsClass::Any),
            _ => Err(ErrorKind::InvalidDnsClass(i).into())
        }
    }


}