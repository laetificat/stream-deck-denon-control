mod registration {
    use serde::Serialize;

    #[derive(Serialize)]
    pub struct RegistrationMessage {
        pub event: String,
        pub uuid: String
    }
}

pub use registration::RegistrationMessage;