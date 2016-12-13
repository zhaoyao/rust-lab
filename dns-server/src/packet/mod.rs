
mod buffer;
pub mod error;

pub mod record_type;
pub mod class;
pub mod query;
pub mod message;

pub use self::message::Header;
pub use self::message::Message;
pub use self::query::Question;
