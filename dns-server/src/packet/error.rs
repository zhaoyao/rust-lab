
error_chain! {

    foreign_links {
        Utf8(::std::string::FromUtf8Error);
        Io(::std::io::Error);
    }

    errors {
        EOF {
            description("unexpected end of input reached")
        }

        InvalidRecordType(n: u16) {
            description("invalid record_type")
            display("invalid record_type: {}", n)
        }

        InvalidDnsClass(n: u16) {
            description("invalid dns class")
            display("invalid dns class: {}", n)
        }

        InvalidMessage(msg: String) {
            description("invalid message")
            display("invalid message: '{}'", msg)
        }
    }


}

