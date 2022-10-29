use std::{env, net::TcpStream, str};

use serde::{Serialize, Deserialize};
use websocket::{ClientBuilder, Message, sync::Reader, sync::Writer, ws::dataframe::DataFrame};

#[derive(Serialize)]
struct RegistrationMessage {
    event: String,
    uuid: String
}

#[derive(Serialize)]
struct LogMessage {
    event: String,
    payload: LogMessagePayload
}

#[derive(Serialize)]
struct LogMessagePayload {
    message: String
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
struct ReceivedEvent {
    action: String,
    event: String,
    payload: ReceivedEventPayload
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct ReceivedEventPayload {
    state: u32,
    user_desired_state: u32
}

fn main() {
    let args: Vec<String> = env::args().collect();

    connect_elgato_stream_deck_socket(
        args.get(2).unwrap().to_owned(), 
        args.get(4).unwrap().to_owned(), 
        args.get(6).unwrap().to_owned(), 
        args.get(8).unwrap().to_owned()
    );
}

fn connect_elgato_stream_deck_socket(in_port: String, in_plugin_uuid: String, in_register_event: String, _in_info: String) {
    let address = "ws://localhost:".to_owned() + &in_port;
    let client = ClientBuilder::new(address.as_str())
    .unwrap()
    .connect_insecure()
    .unwrap();

    let (receiver, mut sender) = client.split().unwrap();

    let registration_message = RegistrationMessage {
        event: in_register_event,
        uuid: in_plugin_uuid
    };

    let message = Message::text(serde_json::to_string(&registration_message).unwrap());
    sender.send_message(&message).unwrap();

    listen_for_events(receiver, sender);
}

fn listen_for_events(mut receiver: Reader<TcpStream>, mut sender: Writer<TcpStream>) {
    for incoming_message in receiver.incoming_messages() {
        let received_message = incoming_message.unwrap().take_payload();

        log_message(&mut sender, str::from_utf8(&received_message).unwrap().to_string());

        let msg = str::from_utf8(&received_message).unwrap();
        let event: ReceivedEvent = match serde_json::from_str(msg) {
            Ok(e) => e,
            Err(err) => {
                log_message(&mut sender, err.to_string());
                ReceivedEvent {action: "".to_string(), event: "".to_string(), payload: ReceivedEventPayload { state: 0, user_desired_state: 0 }}
            }
        };

        match event.action.as_str() {
            "nl.kevinheruer.denon.power" => cmd_power(event, &mut sender),
            _ => {}
        };

        
    };
}

fn cmd_power(event: ReceivedEvent, sender: &mut Writer<TcpStream>) {
    match event.event.as_str() {
        "keyUp" => {
            match event.payload.state {
                0 => {
                    match reqwest::blocking::get("http://192.168.1.120:8080/goform/formiPhoneAppDirect.xml?PWSTANDBY") {
                        Err(err) => log_message(sender, err.to_string()),
                        _ => {}
                    }
                },
                1 => {
                    match reqwest::blocking::get("http://192.168.1.120:8080/goform/formiPhoneAppDirect.xml?PWON") {
                        Err(err) => log_message(sender, err.to_string()),
                        _ => {}
                    }
                },
                _ => log_message(sender, "state out of range".to_string())
            }
            
        },
        _ => {}
    }
}

fn log_message(sender: &mut Writer<TcpStream>, message: String) {
    let log = LogMessage {
        event: "logMessage".to_owned(),
        payload: LogMessagePayload { message }
    };

    let message = Message::text(serde_json::to_string(&log).unwrap());
    sender.send_message(&message).unwrap();
}
