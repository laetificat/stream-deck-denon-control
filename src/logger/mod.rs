mod logger {
    use std::net::TcpStream;

    use websocket::{sync::Writer, Message};

    use crate::models;

    pub struct Client {
        pub sender: Writer<TcpStream>
    }

    impl Client {
        #[allow(dead_code)]
        pub fn debug(&mut self, message: String) {
            let log = models::log::Message {
                event: "logMessage".to_owned(),
                payload: models::log::Payload { message }
            };
        
            let message = Message::text(serde_json::to_string(&log).unwrap());
            self.sender.send_message(&message).unwrap();
        }

        #[allow(dead_code)]
        pub fn error(&mut self, message: String) {
            let log = models::log::Message {
                event: "logMessage".to_owned(),
                payload: models::log::Payload { message }
            };
        
            let message = Message::text(serde_json::to_string(&log).unwrap());
            self.sender.send_message(&message).unwrap();
        }
    }
}

pub use logger::Client;