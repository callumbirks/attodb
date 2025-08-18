pub enum Value<'a> {
    Int(i32),
    String(&'a str),
}

#[repr(u8)]
enum Variant {
    Int = 0,
    String = 1,
}

#[derive(Debug)]
pub enum Error {
    UnknownValueType(u8),
    Invalid,
}

impl TryFrom<u8> for Variant {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Int),
            1 => Ok(Self::String),
            _ => Err(Error::UnknownValueType(value)),
        }
    }
}

impl<'a> Value<'a> {
    pub fn parse(buf: &'a [u8]) -> crate::Result<Value<'a>> {
        if buf.is_empty() {
            return Err(crate::Error::ParseValue(Error::Invalid));
        }
        let variant = match Variant::try_from(buf[0]) {
            Ok(v) => v,
            Err(e) => return Err(crate::Error::ParseValue(e)),
        };
        match variant {
            Variant::Int => read_int(buf).map(Value::Int),
            Variant::String => read_string(buf).map(Value::String),
        }
    }

    pub fn write(&self, buf: &mut Vec<u8>) {
        match self {
            Value::Int(int) => {
                buf.resize(5, 0u8);
                buf[0] = Variant::Int as u8;
                buf[1..5].copy_from_slice(&i32::to_be_bytes(*int));
            }
            Value::String(string) => {
                let string_bytes = string.as_bytes();
                buf.resize(1 + string_bytes.len(), 0u8);
                buf[0] = Variant::String as u8;
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        string_bytes.as_ptr(),
                        buf[1..=string_bytes.len()].as_mut_ptr(),
                        string_bytes.len(),
                    );
                }
            }
        }
    }

    pub fn into_vec(&self) -> Vec<u8> {
        let len = match self {
            Value::Int(_) => 5,
            Value::String(string) => 1 + string.len(),
        };
        let mut buf = vec![0u8; len];
        self.write(&mut buf);
        buf
    }
}

fn read_int(buf: &[u8]) -> crate::Result<i32> {
    if buf.len() < 5 {
        return Err(crate::Error::ParseValue(Error::Invalid));
    }
    Ok(i32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]))
}

fn read_string<'a>(buf: &'a [u8]) -> crate::Result<&'a str> {
    match str::from_utf8(&buf[1..]) {
        Ok(s) => Ok(s),
        Err(_) => Err(crate::Error::InvalidUtf8),
    }
}
