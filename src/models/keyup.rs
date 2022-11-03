mod keyup {
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct KeyUpAction {
        pub action: String,
        pub event: String,
        pub context: String,
        pub device: String,
        pub payload: KeyUpActionPayload
    }
    
    #[derive(Deserialize)]
    pub struct KeyUpActionPayload {
        pub coordinates: KeyUpActionPayloadCoordinates,
        pub state: u32
    }
    
    #[derive(Deserialize)]
    pub struct KeyUpActionPayloadCoordinates {
        pub column: u32,
        pub row: u32
    }
}

pub use keyup::KeyUpAction;