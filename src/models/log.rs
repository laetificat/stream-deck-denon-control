mod log {
    use serde::Serialize;

    #[derive(Serialize)]
    pub struct Message {
        pub event: String,
        pub payload: Payload
    }
    
    #[derive(Serialize)]
    pub struct Payload {
        pub message: String
    }
}

pub use log::Message;
pub use log::Payload;