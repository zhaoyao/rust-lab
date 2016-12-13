
use super::error::*;
use super::buffer::Buffer;
use super::query::*;

#[derive(Debug)]
pub enum MessageType {
    Query, Response
}

#[derive(Debug)]
pub struct Header {
    id: u16,
    //flags: u16,
    msg_type: MessageType,
    op_code: u8,
    authoritative_answer : bool, truncation: bool, recursion_desired: bool,
    recursion_available: bool, authentic_data: bool, checking_disabled: bool,
    response_code: u8,
    qd_count: u16,
    an_count: u16,
    ns_count: u16,
    ar_count: u16
}

impl Header {

    /*
    pub fn new() -> Self {
        Header {
            id: 0,
            flags: 0,
            qd_count: 0,
            an_count: 0,
            ns_count: 0,
            ar_count: 0
        }
    }
    */

    pub fn decode(b: &mut Buffer) -> Result<Self> {
        let id = b.u16()?;

        let f1 = b.u8()?;
        let f2 = b.u8()?;

        let mtype = if (0x80 & f1) == 0x80 {
            MessageType::Response
        } else {
            MessageType::Query
        };

        let op_code = (0x78 & f1) >> 3;

        let aa = (0x4&f1) == 0x4;
        let tc = (0x2&f1) == 0x2;
        let rd = (0x1&f1) == 0x1;

        let ra = (0b1000_0000 & f2) == 0b10000_0000;
        let ad = (0b0010_0000 & f2) == 0b0010_0000;
        let cd = (0b0001_0000 & f2) == 0b0001_0000;

        let rc = 0x0f & f2;

        let qd_count = b.u16()?;
        let an_count = b.u16()?;
        let ns_count = b.u16()?;
        let ar_count = b.u16()?;

        Ok(Header{
            id: id,
            msg_type: mtype,
            op_code: op_code,
            authoritative_answer: aa,
            truncation: tc,
            recursion_desired: rd,
            recursion_available: ra,
            authentic_data: ad,
            checking_disabled: cd,
            response_code: rc,
            qd_count: qd_count,
            an_count: an_count,
            ns_count: ns_count,
            ar_count: ar_count
        })
    }

    pub fn query_count(&self) -> usize {
        self.qd_count as usize
    }
}

#[derive(Debug)]
pub struct Message {
    // Header
    header: Header,
    questions: Vec<Question>,
    // Question
    // Answer
    // Authority
    // Additional
}

impl Message {

    pub fn read(b: &[u8]) -> Result<Message> {
        let mut buf = Buffer::new(b);

        let h = Header::decode(&mut buf)?;

        let qc = h.query_count();

        let mut q = Vec::with_capacity(qc);
        for _ in 0..qc {
            q.push(Question::read(&mut buf)?);
        }

        Ok(Message{header: h, questions: q})
    }

}
