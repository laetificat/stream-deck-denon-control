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
        #[serde(default)]
        pub state: u32,
        #[serde(default)]
        pub settings: serde_json::Map<std::string::String, serde_json::Value>
    }
    
    #[derive(Deserialize)]
    pub struct KeyUpActionPayloadCoordinates {
        pub column: u32,
        pub row: u32
    }
}

pub use keyup::KeyUpAction;