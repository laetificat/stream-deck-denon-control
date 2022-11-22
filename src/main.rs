mod models;
mod denon;
mod logger;

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

    let get_global_settings_message = models::settings::GetGlobalSettings {
        event: String::from("getGlobalSettings"),
        context: in_plugin_uuid.to_owned()
    };

    let message = Message::text(serde_json::to_string(&get_global_settings_message).unwrap());
    sender.send_message(&message).unwrap();

    listen_for_events(receiver, sender, String::from(""));
}

fn listen_for_events(mut receiver: Reader<TcpStream>, sender: Writer<TcpStream>, ip_address: String) {
    let mut logger = logger::Client {
        sender
    };
    let mut client = denon::Client {
        ip: ip_address.to_owned(),
    };

    for incoming_message in receiver.incoming_messages() {
        let received_message = incoming_message.unwrap().take_payload();

        let msg = str::from_utf8(&received_message).unwrap();
        match serde_json::from_str::<models::event::Event>(msg) {
            Ok(event) => handle_event(event, msg, &mut client, &mut logger),
            Err(err) => logger.error(err.to_string())
        }
    };
}

fn handle_event(event: Event, msg: &str, client: &mut denon::Client, logger: &mut logger::Client) {
    match event.event.as_str() {
        "keyUp" => handle_keyup_action(msg, client, logger),
        "didReceiveGlobalSettings" => handle_global_settings(msg, client, logger),
        _ => logger.error(String::from("could not handle event: ".to_owned()+event.event.as_str()))
    }
}

fn handle_global_settings(msg: &str, client: &mut denon::Client, logger: &mut logger::Client) {
    match serde_json::from_str::<models::settings::Global>(msg) {
        Ok(global_settings) => {
            match global_settings.payload.settings.get("ipaddress") {
                Some(ip) => {
                    client.ip = ip.to_string().replace("\"", "");
                },
                None => logger.error(String::from("could not find ipaddress setting"))
            }
        },
        Err(err) => logger.error(err.to_string())
    };
}

fn handle_keyup_action(msg: &str, client: &denon::Client, logger: &mut logger::Client) {
    match serde_json::from_str::<models::keyup::KeyUpAction>(msg) {
        Ok(action) => {
            match action.action.as_str() {
                "nl.kevinheruer.denon.power" => cmd_power(action, client, logger),
                "nl.kevinheruer.denon.volumespecific" => cmd_volumespecific(action, client),
                "nl.kevinheruer.denon.volumeup" => client.volume_up(),
                "nl.kevinheruer.denon.volumedown" => client.volume_down(),
                "nl.kevinheruer.denon.mute" => cmd_mute(action, client, logger),
                _ => logger.error(String::from("could not handle action"))
            }
        },
        Err(err) => logger.error(err.to_string())
    }
}

fn cmd_power(action: models::keyup::KeyUpAction, client: &denon::Client, logger: &mut logger::Client) {
    match action.payload.state {
        0 => client.power_standby(),
        1 => client.power_on(),
        _ => logger.error("state out of range".to_string())
    }
}

fn cmd_volumespecific(action: models::keyup::KeyUpAction, client: &denon::Client) {
    match action.payload.settings.get("volumelevel") {
        Some(level) => client.volume_specific(&level.to_string()),
        None => {}
    }
}

fn cmd_mute(action: models::keyup::KeyUpAction, client: &denon::Client, logger: &mut logger::Client) {
    match action.payload.state {
        0 => client.volume_mute_off(),
        1 => client.volume_mute_on(),
        _ => logger.error("state out of range".to_string())
    }
}
