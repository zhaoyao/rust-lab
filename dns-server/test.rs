#![feature(prelude_import)]
#![no_std]
#![recursion_limit = "1024"]
#[prelude_import]
use std::prelude::v1::*;
#[macro_use]
extern crate std as std;

#[macro_use]
extern crate error_chain;

mod packet {




    // read from the socket



    // send a reply to the socket we received data from


    mod buffer {
        use super::error::{Result, Error};
        pub struct Buffer<'a> {
            b: &'a [u8],
            i: usize,
        }
        impl <'a> Buffer<'a> {
            pub fn new(b: &'a [u8]) -> Self { Buffer{b: b, i: 0,} }
            fn check(&self, n: usize) -> Option<Error> {
                match self.i + n >= self.b.len() {
                    true => Some(Err(Error::new(ErrorKind::Other, "EOF"))),
                    false => None,
                }
            }
            pub fn u8(&mut self) -> Result<u8> {
                match self.check(1) {
                    None => { self.i += 1; Ok(self.b[self.i - 1]) }
                    Some(err) => err,
                }
            }
            pub fn u16(&mut self) -> Result<u16> {
                match self.check(2) {
                    None => {
                        let a = self.b[self.i];
                        let b = self.b[self.i + 1];
                        self.i += 1;
                        Ok(((a as u16) << 8) | (b as u16))
                    }
                    Some(err) => err,
                }
            }
            pub fn slice(&mut self, n: usize) -> Result<[u8]> {
                match self.check(n) {
                    None => Ok(self.b[self.i..self.i + n]),
                    Some(err) => err,
                }
            }
            pub fn seek(&mut self, i: usize) -> Option<Error> {
                if i >= self.b.len() {
                    return Some(Error::new(ErrorKind::Other, "out of bounds"))
                }
                self.i = i;
                None
            }
        }
    }
    pub mod error {
        /// The Error type.
        ///
        /// This struct is made of three things:
        ///
        /// - an `ErrorKind` which is used to determine the type of the error.
        /// - a backtrace, generated when the error is created.
        /// - an error chain, used for the implementation of `Error::cause()`.
        pub struct Error(
                         /// The kind of the error.
                         #[doc(hidden)]
                         pub ErrorKind,
                         /// Contains the error chain and the backtrace.
                         #[doc(hidden)]
                         pub ::error_chain::State);
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::std::fmt::Debug for Error {
            fn fmt(&self, __arg_0: &mut ::std::fmt::Formatter)
             -> ::std::fmt::Result {
                match *self {
                    Error(ref __self_0_0, ref __self_0_1) => {
                        let mut builder = __arg_0.debug_tuple("Error");
                        let _ = builder.field(&&(*__self_0_0));
                        let _ = builder.field(&&(*__self_0_1));
                        builder.finish()
                    }
                }
            }
        }
        impl ::error_chain::ChainedError for Error {
            type
            ErrorKind
            =
            ErrorKind;
            fn new(kind: ErrorKind, state: ::error_chain::State) -> Error {
                Error(kind, state)
            }
            fn extract_backtrace(e: &(::std::error::Error+ Send + 'static))
             -> Option<::std::sync::Arc<::error_chain::Backtrace>> {
                if let Some(e) = e.downcast_ref::<Error>() {
                    return e.1.backtrace.clone();
                }
                None
            }
        }
        #[allow(dead_code)]
        impl Error {
            /// Constructs an error from a kind, and generates a backtrace.
            pub fn from_kind(kind: ErrorKind) -> Error {
                Error(kind, ::error_chain::State::default())
            }
            /// Returns the kind of the error.
            pub fn kind(&self) -> &ErrorKind { &self.0 }
            /// Iterates over the error chain.
            pub fn iter(&self) -> ::error_chain::ErrorChainIter {
                ::error_chain::ErrorChainIter(Some(self))
            }
            /// Returns the backtrace associated with this error.
            pub fn backtrace(&self) -> Option<&::error_chain::Backtrace> {
                self.1.backtrace()
            }
        }
        impl ::std::error::Error for Error {
            fn description(&self) -> &str { self.0.description() }
            fn cause(&self) -> Option<&::std::error::Error> {
                match self.1.next_error {
                    Some(ref c) => Some(&**c),
                    None => { match self.0 { _ => None, } }
                }
            }
        }
        impl ::std::fmt::Display for Error {
            fn fmt(&self, f: &mut ::std::fmt::Formatter)
             -> ::std::fmt::Result {
                ::std::fmt::Display::fmt(&self.0, f)
            }
        }
        impl From<ErrorKind> for Error {
            fn from(e: ErrorKind) -> Self { Error::from_kind(e) }
        }
        impl <'a> From<&'a str> for Error {
            fn from(s: &'a str) -> Self { Error::from_kind(s.into()) }
        }
        impl From<String> for Error {
            fn from(s: String) -> Self { Error::from_kind(s.into()) }
        }
        impl ::std::ops::Deref for Error {
            type
            Target
            =
            ErrorKind;
            fn deref(&self) -> &Self::Target { &self.0 }
        }
        #[doc = r" The kind of an error."]
        pub enum ErrorKind {

            #[doc = r" A convenient variant for String."]
            Msg(String),
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::std::fmt::Debug for ErrorKind {
            fn fmt(&self, __arg_0: &mut ::std::fmt::Formatter)
             -> ::std::fmt::Result {
                match (&*self,) {
                    (&ErrorKind::Msg(ref __self_0),) => {
                        let mut builder = __arg_0.debug_tuple("Msg");
                        let _ = builder.field(&&(*__self_0));
                        builder.finish()
                    }
                }
            }
        }
        #[allow(unused)]
        impl ::std::fmt::Display for ErrorKind {
            fn fmt(&self, fmt: &mut ::std::fmt::Formatter)
             -> ::std::fmt::Result {
                match *self
                    {
                     #[doc = r" A convenient variant for String."]
                     ErrorKind::Msg(ref s) => {
                         let display_fn = |_, f: &mut ::std::fmt::Formatter| {
                             f.write_fmt(::std::fmt::Arguments::new_v1({
                                                                           static __STATIC_FMTSTR:
                                                                                  &'static [&'static str]
                                                                                  =
                                                                               &[""];
                                                                           __STATIC_FMTSTR
                                                                       },
                                                                       &match (&s,)
                                                                            {
                                                                            (__arg0,)
                                                                            =>
                                                                            [::std::fmt::ArgumentV1::new(__arg0,
                                                                                                         ::std::fmt::Display::fmt)],
                                                                        })) };
                         display_fn(self, fmt)
                     }
                }
            }
        }
        #[allow(unused)]
        impl ErrorKind {
            /// A string describing the error kind.
            pub fn description(&self) -> &str {
                match *self
                    {
                     #[doc = r" A convenient variant for String."]
                     ErrorKind::Msg(ref s) => {
                         &s
                     }
                }
            }
        }
        impl <'a> From<&'a str> for ErrorKind {
            fn from(s: &'a str) -> Self { ErrorKind::Msg(s.to_string()) }
        }
        impl From<String> for ErrorKind {
            fn from(s: String) -> Self { ErrorKind::Msg(s) }
        }
        impl From<Error> for ErrorKind {
            fn from(e: Error) -> Self { e.0 }
        }
        /// Additionnal methods for `Result`, for easy interaction with this crate.
        pub trait ResultExt<T, E> {
            /// If the `Result` is an `Err` then `chain_err` evaluates the closure,
            /// which returns *some type that can be converted to `ErrorKind`*, boxes
            /// the original error to store as the cause, then returns a new error
            /// containing the original error.
            fn chain_err<F, EK>(self, callback: F)
            -> ::std::result::Result<T, Error>
            where
            F: FnOnce()
            ->
            EK,
            EK: Into<ErrorKind>;
        }
        impl <T, E> ResultExt<T, E> for ::std::result::Result<T, E> where
         E: ::std::error::Error + Send + 'static {
            fn chain_err<F, EK>(self, callback: F)
             -> ::std::result::Result<T, Error> where F: FnOnce() -> EK,
             EK: Into<ErrorKind> {
                self.map_err(move |e| {
                             let state =
                                 ::error_chain::State::new::<Error>(Box::new(e));
                             ::error_chain::ChainedError::new(callback().into(),
                                                              state) })
            }
        }
        /// Convenient wrapper around `std::Result`.
        pub type Result<T> = ::std::result::Result<T, Error>;
    }
    pub mod q_type {
        pub enum QType {
            A,
            NS,
            MD,
            MF,
            CNAME,
            SOA,
            MB,
            MG,
            MR,
            NULL,
            WKS,
            PTR,
            HINFO,
            MINFO,
            MX,
            TXT,
            AXFR,
            MAILB,
            MAILA,
            Any,
        }
        impl QType {
            pub fn from_u8(i: u8) -> Result<QType, String> {
                match i {
                    1 => A,
                    2 => NS,
                    3 => MD,
                    4 => MF,
                    5 => CNAME,
                    6 => SOA,
                    7 => MB,
                    8 => MG,
                    9 => MR,
                    10 => NULL,
                    11 => WKS,
                    12 => PTR,
                    13 => HINFO,
                    14 => MINFO,
                    15 => MX,
                    16 => TXT,
                    252 => AXFR,
                    253 => MAILA,
                    255 => Any,
                }
            }
        }
    }
    pub mod query {
        use std::String;
        use std::io::{Result, Error, ErrorKind};
        use super::Buffer;
        struct Question {
            q_name: Vec<[u8]>,
            q_type: u16,
            q_class: u16,
        }
        pub fn read_labels(b: &mut Buffer) -> Result<Vec<[u8]>> {
            let c = 0;
            let v = Vec::new();
            loop  {
                if c > 255 {
                    return Err(Error::new(ErrorKind::Other, "label too long"))
                }
                let l = b.u8()?;
                l = l & 63;
                if l == 0 { break  }
                v.push(b.slice(l)?);
            }
            v
        }
        impl Question {
            pub fn read(b: &mut Buffer) -> Result<Self> {
                Ok(Question{q_name: read_labels(b),
                            q_type: b.u16()?,
                            q_class: b.u16()?,})
            }
        }
    }
    pub mod message {
        use std::io::Result;
        use std::slice::Iter;
        use std::fmt::Display;
        use super::buffer::Buffer;
        pub enum MessageType { Query, Response, }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::std::fmt::Debug for MessageType {
            fn fmt(&self, __arg_0: &mut ::std::fmt::Formatter)
             -> ::std::fmt::Result {
                match (&*self,) {
                    (&MessageType::Query,) => {
                        let mut builder = __arg_0.debug_tuple("Query");
                        builder.finish()
                    }
                    (&MessageType::Response,) => {
                        let mut builder = __arg_0.debug_tuple("Response");
                        builder.finish()
                    }
                }
            }
        }
        pub struct Header {
            id: u16,
            msg_type: MessageType,
            op_code: u8,
            authoritative_answer: bool,
            truncation: bool,
            recursion_desired: bool,
            recursion_available: bool,
            authentic_data: bool,
            checking_disabled: bool,
            response_code: u8,
            qd_count: u16,
            an_count: u16,
            ns_count: u16,
            ar_count: u16,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::std::fmt::Debug for Header {
            fn fmt(&self, __arg_0: &mut ::std::fmt::Formatter)
             -> ::std::fmt::Result {
                match *self {
                    Header {
                    id: ref __self_0_0,
                    msg_type: ref __self_0_1,
                    op_code: ref __self_0_2,
                    authoritative_answer: ref __self_0_3,
                    truncation: ref __self_0_4,
                    recursion_desired: ref __self_0_5,
                    recursion_available: ref __self_0_6,
                    authentic_data: ref __self_0_7,
                    checking_disabled: ref __self_0_8,
                    response_code: ref __self_0_9,
                    qd_count: ref __self_0_10,
                    an_count: ref __self_0_11,
                    ns_count: ref __self_0_12,
                    ar_count: ref __self_0_13 } => {
                        let mut builder = __arg_0.debug_struct("Header");
                        let _ = builder.field("id", &&(*__self_0_0));
                        let _ = builder.field("msg_type", &&(*__self_0_1));
                        let _ = builder.field("op_code", &&(*__self_0_2));
                        let _ =
                            builder.field("authoritative_answer",
                                          &&(*__self_0_3));
                        let _ = builder.field("truncation", &&(*__self_0_4));
                        let _ =
                            builder.field("recursion_desired",
                                          &&(*__self_0_5));
                        let _ =
                            builder.field("recursion_available",
                                          &&(*__self_0_6));
                        let _ =
                            builder.field("authentic_data", &&(*__self_0_7));
                        let _ =
                            builder.field("checking_disabled",
                                          &&(*__self_0_8));
                        let _ =
                            builder.field("response_code", &&(*__self_0_9));
                        let _ = builder.field("qd_count", &&(*__self_0_10));
                        let _ = builder.field("an_count", &&(*__self_0_11));
                        let _ = builder.field("ns_count", &&(*__self_0_12));
                        let _ = builder.field("ar_count", &&(*__self_0_13));
                        builder.finish()
                    }
                }
            }
        }
        impl Header {
            pub fn decode(b: &mut Buffer) -> Result<Self> {
                let id = b.u16()?;
                let f1 = b.u8()?;
                let f2 = b.u8()?;
                let mtype =
                    if (128 & f1) == 128 {
                        MessageType::Response
                    } else { MessageType::Query };
                let op_code = (120 & f1) >> 3;
                let aa = (4 & f1) == 4;
                let tc = (2 & f1) == 2;
                let rd = (1 & f1) == 1;
                let ra = (128 & f2) == 256;
                let ad = (32 & f2) == 32;
                let cd = (16 & f2) == 16;
                let rc = 15 & f2;
                let qd_count = b.u16()?;
                let an_count = b.u16()?;
                let ns_count = b.u16()?;
                let ar_count = b.u16()?;
                Ok(Header{id: id,
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
                          ar_count: ar_count,})
            }
            pub fn query_count(&self) -> usize { self.qd_count as usize }
        }
        pub struct Message {
            header: Header,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::std::fmt::Debug for Message {
            fn fmt(&self, __arg_0: &mut ::std::fmt::Formatter)
             -> ::std::fmt::Result {
                match *self {
                    Message { header: ref __self_0_0 } => {
                        let mut builder = __arg_0.debug_struct("Message");
                        let _ = builder.field("header", &&(*__self_0_0));
                        builder.finish()
                    }
                }
            }
        }
        impl Message {
            pub fn read(b: &[u8]) -> Result<Message> {
                let mut buf = Buffer::new(b);
                let h = Header::decode(&mut buf)?;
                let qc = &h.query_count();
                Ok(Message{header: h,})
            }
        }
    }
    pub use self::message::Header;
    pub use self::message::Message;
    pub use self::query::Question;
}
use std::net::UdpSocket;
use std::io::Result;
fn bind_udp() -> Result<()> {
    let mut socket = UdpSocket::bind("0.0.0.0:53")?;
    let mut buf = [0; 4096];
    let (amt, src) = socket.recv_from(&mut buf)?;
    let p = packet::Message::read(&buf[..amt])?;
    ::std::io::_print(::std::fmt::Arguments::new_v1({
                                                        static __STATIC_FMTSTR:
                                                               &'static [&'static str]
                                                               =
                                                            &["", "\n"];
                                                        __STATIC_FMTSTR
                                                    },
                                                    &match (&p,) {
                                                         (__arg0,) =>
                                                         [::std::fmt::ArgumentV1::new(__arg0,
                                                                                      ::std::fmt::Debug::fmt)],
                                                     }));
    Ok(())
}
fn main() { bind_udp().unwrap(); }
