mod settings {
    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    #[derive(Deserialize)]
    pub struct Global {
        pub event: String,
        pub payload: GlobalPayloadSettings
    }

    #[derive(Deserialize)]
    pub struct GlobalPayloadSettings {
        pub settings: serde_json::Map<String, Value>
    }

    #[derive(Serialize)]
    pub struct GetGlobalSettings {
        pub event: String,
        pub context: String
    }
}

pub use settings::{Global, GetGlobalSettings};