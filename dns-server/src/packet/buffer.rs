use super::error::*;

pub struct Buffer<'a> {
    b: &'a [u8],
    i: usize
}

impl<'a> Buffer<'a> {
    pub fn new(b: &'a [u8]) -> Self {
        Buffer { b: b, i: 0 }
    }

    fn check(&self, n: usize) -> Result<()> {
        match self.i + n >= self.b.len() {
            true => Err(ErrorKind::EOF.into()),
            false => Ok(())
        }
    }

    pub fn u8(&mut self) -> Result<u8> {
        self.check(1 as usize)?;
        self.i += 1;
        Ok(self.b[self.i - 1])
    }

    pub fn u16(&mut self) -> Result<u16> {
        self.check(2 as usize)?;
        let a = self.b[self.i];
        let b = self.b[self.i + 1];
        self.i += 2;
        Ok(((a as u16) << 8) | b as u16)
    }

    pub fn string(&mut self, n: usize) -> Result<String> {
        self.check(n)?;
        let mut vec: Vec<u8> = Vec::with_capacity(n);
        vec.extend(self.b[self.i..self.i + n].iter());
        self.i += n;
        Ok(String::from_utf8(vec)?)
    }

    pub fn seek(&mut self, i: usize) -> Result<()> {
        if i >= self.b.len() {
            return Err(ErrorKind::EOF.into())
        }
        self.i = i;
        Ok(())
    }
}