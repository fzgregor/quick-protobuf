extern crate quick_protobuf;

use std::io::{Write};
use quick_protobuf::{BytesReader, Writer, MessageWrite, Result};
use quick_protobuf::sizeofs::*;

macro_rules! write_read_primitive {
    ($name:ident, $read:ident, $write:ident) => (write_read_primitive!($name, $read, $write, 145););
    ($name:ident, $read:ident, $write:ident, $def:expr) => (
#[test]
fn $name(){
    let v = $def;
    let mut buf = Vec::new();
    {
        let mut w = Writer::new(&mut buf);
        w.$write(v).unwrap();
    }
    let mut r = BytesReader::from_bytes(&*buf);
    assert_eq!(v, r.$read(&buf).unwrap());
}
    );
}

write_read_primitive!(wr_int32, read_int32, write_int32);
write_read_primitive!(wr_int64, read_int64, write_int64);
write_read_primitive!(wr_uint32, read_uint32, write_uint32);
write_read_primitive!(wr_uint64, read_uint64, write_uint64);
write_read_primitive!(wr_sint32, read_sint32, write_sint32);
write_read_primitive!(wr_sint64, read_sint64, write_sint64);
write_read_primitive!(wr_bool, read_bool, write_bool, true);
write_read_primitive!(wr_fixed32, read_fixed32, write_fixed32);
write_read_primitive!(wr_fixed64, read_fixed64, write_fixed64);
write_read_primitive!(wr_sfixed32, read_sfixed32, write_sfixed32);
write_read_primitive!(wr_sfixed64, read_sfixed64, write_sfixed64);
write_read_primitive!(wr_float, read_float, write_float, 5.8);
write_read_primitive!(wr_double, read_double, write_double, 5.8);

#[test]
fn wr_bytes(){
    let v = b"test_write_read";
    let mut buf = Vec::new();
    {
        let mut w = Writer::new(&mut buf);
        w.write_bytes(v).unwrap();
    }
    let mut r = BytesReader::from_bytes(&*buf);
    assert_eq!(v, r.read_bytes(&buf).unwrap());
}

#[test]
fn wr_string(){
    let v = "test_write_read";
    let mut buf = Vec::new();
    {
        let mut w = Writer::new(&mut buf);
        w.write_string(v).unwrap();
    }
    let mut r = BytesReader::from_bytes(&buf);
    assert_eq!(v, r.read_string(&buf).unwrap());
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum TestEnum {
    A = 0,
    B = 1,
    C = 2,
}

impl From<i32> for TestEnum {
    fn from(v: i32) -> TestEnum {
        match v {
            0 => TestEnum::A,
            1 => TestEnum::B,
            2 => TestEnum::C,
            _ => unreachable!(),
        }
    }
}

#[test]
fn wr_enum(){
    let v = TestEnum::C;
    let mut buf = Vec::new();
    {
        let mut w = Writer::new(&mut buf);
        w.write_enum(v as i32).unwrap();
    }
    let mut r = BytesReader::from_bytes(&buf);
    assert_eq!(v, r.read_enum(&buf).unwrap());
}

#[derive(PartialEq, Eq, Debug, Clone, Default)]
struct TestMessage {
    id: Option<u32>,
    val: Vec<i64>,
}

// impl MessageRead for TestMessage {
impl TestMessage {
    fn from_reader(r: &mut BytesReader, bytes: &[u8]) -> Result<TestMessage> {
        let mut msg = TestMessage::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.id = Some(r.read_uint32(bytes)?),
                Ok(18) => msg.val.push(r.read_sint64(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for TestMessage {
    fn get_size(&self) -> usize {
        self.id.as_ref().map_or(0, |m| 1 + sizeof_uint32(*m))
        + self.val.iter().map(|m| 1 + sizeof_sint64(*m)).sum::<usize>()
    }

    fn write_message<W: Write>(&self, r: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.id { r.write_with_tag(10, |r| r.write_uint32(*s))?; }
        for s in &self.val { r.write_with_tag(18, |r| r.write_sint64(*s))?; }
        Ok(())
    }
}

#[test]
fn wr_message(){

    let v = TestMessage {
        id: Some(63),
        val: vec![53, 5, 76, 743, 23, 753],
    };
    let mut buf = Vec::new();
    {
        let mut w = Writer::new(&mut buf);
        w.write_message(&v).unwrap();
    }
    let mut r = BytesReader::from_bytes(&buf);
    assert_eq!(v, r.read_message(&buf, TestMessage::from_reader).unwrap());

    // test get_size!
    assert_eq!(buf.len(), sizeof_varint(8) + v.get_size());
}

#[derive(PartialEq, Eq, Debug, Clone, Default)]
struct TestMessageBorrow<'a> {
    id: Option<u32>,
    val: Vec<&'a str>,
}

impl<'a> TestMessageBorrow<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a[u8]) -> Result<TestMessageBorrow<'a>> {
        let mut msg = TestMessageBorrow::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.id = Some(r.read_uint32(bytes)?),
                Ok(18) => msg.val.push(r.read_string(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
            println!("{:?}", msg);
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for TestMessageBorrow<'a> {
    fn get_size(&self) -> usize {
        self.id.as_ref().map_or(0, |m| 1 + sizeof_uint32(*m))
        + self.val.iter().map(|m| 1 + sizeof_len(m.len())).sum::<usize>()
    }

    fn write_message<W: Write>(&self, r: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.id { r.write_with_tag(10, |r| r.write_uint32(*s))?; }
        for s in &self.val { r.write_with_tag(18, |r| r.write_string(*s))?; }
        Ok(())
    }
}

#[test]
fn wr_message_borrow(){

    let test = "eajhawbdkjblncljbdskjbclas";

    let v = TestMessageBorrow {
        id: Some(63),
        val: vec![&test[0..2], &test[3..7], &test[7..10]],
    };
    let mut buf = Vec::new();
    {
        let mut w = Writer::new(&mut buf);
        w.write_message(&v).unwrap();
    }
    let mut r = BytesReader::from_bytes(&buf);
    assert_eq!(v, r.read_message(&buf, TestMessageBorrow::from_reader).unwrap());

    // test get_size!
    assert_eq!(buf.len(), sizeof_varint(8) + v.get_size());
}

#[test]
fn wr_packed_uint32(){
    let v = vec![43, 54, 64, 234, 6123, 643];
    let mut buf = Vec::new();
    {
        let mut w = Writer::new(&mut buf);
        w.write_packed(&v, |r, m| r.write_uint32(*m), &|m| sizeof_uint32(*m)).unwrap();
    }
    let mut r = BytesReader::from_bytes(&buf);
    assert_eq!(v, r.read_packed(&buf, |r, b| r.read_uint32(b)).unwrap());
}

#[test]
fn wr_map(){
    let v = {
        let mut v = HasgMap::new();
        v.insert(Cow::Borrowed("foo"), 1u32);
        v.insert(Cow::Borrowed("bar"), 2);
        v
    };
    let mut buf = Vec::new();
    {
        let mut w = Writer::new(&mut buf);
        w.write_map(v, 
                    sizeof_len(v.iter().map(|(k, v)| 2 + sizeof_len(k.len()) + sizeof_uint32(v)).sum::<usize>()), 


                

        w.write_packed(&v, |r, m| r.write_uint32(*m), &|m| sizeof_uint32(*m)).unwrap();
    }
    let mut r = BytesReader::from_bytes(&buf);
    assert_eq!(v, r.read_packed(&buf, |r, b| r.read_uint32(b)).unwrap());
}
