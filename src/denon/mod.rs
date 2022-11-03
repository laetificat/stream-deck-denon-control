mod denon {
    use std::net::TcpStream;

    use websocket::sync::Writer;

    pub struct Client<'a> {
        pub ip: String,
        pub writer: &'a mut Writer<TcpStream>
    }

    impl Client<'_> {
        pub fn power_standby(&self) {
            match reqwest::blocking::get("http://".to_owned()+self.ip.as_str()+":8080/goform/formiPhoneAppDirect.xml?PWSTANDBY") {
                // Err(err) => log_message(sender, err.to_string()),
                Err(err) => {},
                _ => {}
            }
        }

        pub fn power_on(&self) {
            match reqwest::blocking::get("http://".to_owned()+self.ip.as_str()+":8080/goform/formiPhoneAppDirect.xml?PWON") {
                // Err(err) => log_message(sender, err.to_string()),
                Err(err) => {},
                _ => {}
            }
        }
    }
}

pub use denon::Client;