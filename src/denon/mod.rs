mod denon {
    pub struct Client {
        pub ip: String,
    }

    impl Client {
        pub fn power_standby(&self) {
            match reqwest::blocking::get("http://".to_owned()+self.ip.as_str()+":8080/goform/formiPhoneAppDirect.xml?PWSTANDBY") {
                Err(_) => {},
                _ => {}
            }
        }

        pub fn power_on(&self) {
            match reqwest::blocking::get("http://".to_owned()+self.ip.as_str()+":8080/goform/formiPhoneAppDirect.xml?PWON") {
                Err(_) => {},
                _ => {}
            }
        }

        pub fn volume_specific(&self, volume_level: &String) {
            if volume_level.len() == 0 {
                return;
            }

            match reqwest::blocking::get("http://".to_owned()+self.ip.as_str()+":8080/goform/formiPhoneAppDirect.xml?MV"+&volume_level.replace("\"", "")) {
                Err(_) => {},
                _ => {}
            }
        }
    }
}

pub use denon::Client;