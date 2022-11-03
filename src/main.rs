mod models;
mod denon;

use std::{env, net::TcpStream, str};

use models::event::Event;
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
            Ok(event) => handle_event(event, msg, &mut sender, &ip_address),
            Err(err) => log_message(&mut sender, err.to_string())
        }
    };
}

fn handle_event(event: Event, msg: &str, sender: &mut Writer<TcpStream>, ip_address: &String) {
    match event.event.as_str() {
        "keyUp" => handle_keyup_action(msg, sender, ip_address),
        "didReceiveGlobalSettings" => log_message(sender, String::from("did receive global settings event")),
        "deviceDidConnect" => log_message(sender, String::from("device did connect event")),
        _ => log_message(sender, String::from("could not handle event"))
    }
}

fn handle_keyup_action(msg: &str, sender: &mut Writer<TcpStream>, ip_address: &String) {
    match serde_json::from_str::<models::keyup::KeyUpAction>(msg) {
        Ok(action) => {
            match action.action.as_str() {
                "nl.kevinheruer.denon.power" => cmd_power(action, sender, &ip_address),
                _ => log_message(sender, String::from("could not handle action"))
            }
        },
        Err(err) => log_message(sender, String::from(err.to_string()))
    }
}

fn cmd_power(action: models::keyup::KeyUpAction, sender: &mut Writer<TcpStream>, ip_address: &String) {
    let client = denon::Client {
        ip: ip_address.to_owned(),
        writer: sender
    };

    match action.payload.state {
        0 => client.power_standby(),
        1 => client.power_on(),
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
