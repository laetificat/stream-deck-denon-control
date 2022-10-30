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
    user_desired_state: u32,
    settings: ReceivedGlobalSettings
}

#[derive(Serialize, Deserialize, Default)]
struct GetGlobalSettingsMessage {
    event: String,
    context: String
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
struct ReceivedGlobalSettings {
    event: String,
    payload: ReceivedPayloadGlobalSettings
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
struct ReceivedPayloadGlobalSettings {
    settings: ReceivedPayloadGlobalSettingsIPAddress
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
struct ReceivedPayloadGlobalSettingsIPAddress {
    ip_address: ReceivedPayloadGlobalSettingsIPAddressValue
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
struct ReceivedPayloadGlobalSettingsIPAddressValue {
    value: String
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
        uuid: String::from(&in_plugin_uuid)
    };

    let message = Message::text(serde_json::to_string(&registration_message).unwrap());
    sender.send_message(&message).unwrap();

    listen_for_events(receiver, sender, in_plugin_uuid, String::from(""));
}

// TODO: rewrite this part to be called ran in a separate process, kill and start new process on config change
fn listen_for_events(mut receiver: Reader<TcpStream>, mut sender: Writer<TcpStream>, plugin_uuid: String, ip_address: String) {
    for incoming_message in receiver.incoming_messages() {
        let received_message = incoming_message.unwrap().take_payload();

        log_message(&mut sender, str::from_utf8(&received_message).unwrap().to_string());

        let msg = str::from_utf8(&received_message).unwrap();
        let event: ReceivedEvent = match serde_json::from_str(msg) {
            Ok(e) => e,
            Err(err) => {
                log_message(&mut sender, err.to_string());
                ReceivedEvent {..Default::default()}
            }
        };

        match event.action.as_str() {
            "nl.kevinheruer.denon.power" => cmd_power(event, &mut sender, &plugin_uuid, &ip_address),
            _ => {
                log_message(&mut sender, String::from("could not handle action"));
                match event.event.as_str() {
                    "didReceiveGlobalSettings" => {
                        log_message(&mut sender, String::from("did receive global settings event"));
                        let ip = event.payload.settings.payload.settings.ip_address.value;
                        log_message(&mut sender, ip);
                    },
                    "deviceDidConnect" => {
                        log_message(&mut sender, String::from("device did connect event"));
                        let get_settings_message = GetGlobalSettingsMessage {
                            event: String::from("getGlobalSettings"),
                            context: plugin_uuid.to_string()
                        };
                        let message = Message::text(serde_json::to_string(&get_settings_message).unwrap());
                        sender.send_message(&message).unwrap();
                    },
                    _ => {
                        log_message(&mut sender, String::from("could not handle event"));
                    }
                }
            }
        };
    };
}

fn cmd_power(event: ReceivedEvent, sender: &mut Writer<TcpStream>, plugin_uuid: &String, ip_address: &String) {
    match event.event.as_str() {
        "keyUp" => {
            match event.payload.state {
                0 => {
                    match reqwest::blocking::get("http://".to_owned()+ip_address+":8080/goform/formiPhoneAppDirect.xml?PWSTANDBY") {
                        Err(err) => log_message(sender, err.to_string()),
                        _ => {}
                    }
                },
                1 => {
                    match reqwest::blocking::get("http://".to_owned()+ip_address+":8080/goform/formiPhoneAppDirect.xml?PWON") {
                        Err(err) => log_message(sender, err.to_string()),
                        _ => {}
                    }
                },
                _ => log_message(sender, "state out of range".to_string())
            }
        },
        "willAppear" => {
            log_message(sender, String::from("will appear event"));
            let get_settings_message = GetGlobalSettingsMessage {
                event: String::from("getGlobalSettings"),
                context: plugin_uuid.to_string()
            };
            let message = Message::text(serde_json::to_string(&get_settings_message).unwrap());
            sender.send_message(&message).unwrap()
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
