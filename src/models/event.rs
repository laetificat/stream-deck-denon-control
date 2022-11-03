mod event {
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct Event {
        pub event: String
    }
}

pub use event::Event;