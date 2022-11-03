mod models;

use std::{env, net::TcpStream, str};

use websocket::{ClientBuilder, Message, sync::Reader, sync::Writer, ws::dataframe::DataFrame};

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

    let registration_message = models::registration::RegistrationMessage {
        event: in_register_event,
        uuid: String::from(&in_plugin_uuid)
    };

    let message = Message::text(serde_json::to_string(&registration_message).unwrap());
    sender.send_message(&message).unwrap();

    listen_for_events(receiver, sender, String::from(""));
}

// TODO: rewrite this part to be called ran in a separate process, kill and start new process on config change
fn listen_for_events(mut receiver: Reader<TcpStream>, mut sender: Writer<TcpStream>, ip_address: String) {
    for incoming_message in receiver.incoming_messages() {
        let received_message = incoming_message.unwrap().take_payload();

        log_message(&mut sender, str::from_utf8(&received_message).unwrap().to_string());

        let msg = str::from_utf8(&received_message).unwrap();
        match serde_json::from_str::<models::event::Event>(msg) {
            Ok(event) => {
                match event.event.as_str() {
                    "keyUp" => {
                        match serde_json::from_str::<models::keyup::KeyUpAction>(msg) {
                            Ok(action) => {
                                match action.action.as_str() {
                                    "nl.kevinheruer.denon.power" => cmd_power(action, &mut sender, &ip_address),
                                    _ => log_message(&mut sender, String::from("could not handle action"))
                                }
                            },
                            Err(err) => log_message(&mut sender, String::from(err.to_string()))
                        }
                    },
                    "didReceiveGlobalSettings" => log_message(&mut sender, String::from("did receive global settings event")),
                    "deviceDidConnect" => log_message(&mut sender, String::from("device did connect event")),
                    _ => log_message(&mut sender, String::from("could not handle event"))
                }
            },
            Err(err) => log_message(&mut sender, err.to_string())
        }
    };
}

fn cmd_power(action: models::keyup::KeyUpAction, sender: &mut Writer<TcpStream>, ip_address: &String) {
    match action.payload.state {
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
}

fn log_message(sender: &mut Writer<TcpStream>, message: String) {
    let log = models::log::Message {
        event: "logMessage".to_owned(),
        payload: models::log::Payload { message }
    };

    let message = Message::text(serde_json::to_string(&log).unwrap());
    sender.send_message(&message).unwrap();
}
