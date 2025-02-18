use tokio_tungstenite::tungstenite::{Message, Result};
use serde_json;

use crate::sequencer;

fn handle_lstn(data: &str) -> Message {
    let selected_port_handle: sequencer::PortHandle = serde_json::from_str(data).unwrap();
    sequencer::listen(selected_port_handle);
    Message::text(format!("lstn\n{}", data).as_str())
}

fn handle_lsif() -> Message {
    let port_list = sequencer::list_midi_ports();
    let mut response = String::from("lsif\n");
    response.push_str(serde_json::to_string(&port_list).unwrap().as_str());
    Message::text(response)
}

fn grok_command(s: &str) -> (&str, &str) {
    match s.chars().next() {
        Some(c) => {
            s.split_at(c.len_utf8() * 4)
        },
        None => s.split_at(0),
    }
}

pub fn dispatch_socket_message(msg: Message) -> Result<Message, &'static str> {
    let packet = msg.to_text().unwrap();
    let (command, data) = grok_command(packet);
    match command {
        "lsif" => {
            Ok(handle_lsif())
        }
        "lstn" => {
            Ok(handle_lstn(data))
        }
        _ => {
                Err("unknown command")
        }
    }
}
