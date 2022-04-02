use futures_util::{SinkExt, StreamExt};
use log::*;
use std::{net::SocketAddr};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Error};
use tokio_tungstenite::tungstenite::{Message, Result};
use alsa;
use std::ffi::CString;
use std::vec::Vec;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct PortHandle {
    name: String,
    client: i32,
    port: i32,
}

fn open_midi_seq() -> alsa::Seq {
    let s = alsa::Seq::open(None, Some(alsa::Direction::Capture), true).unwrap();
    let cstr = CString::new("PianoTop Sequencer").unwrap();
    s.set_client_name(&cstr).unwrap();
    s
}

fn list_midi_ports(s: &alsa::Seq) -> Vec<PortHandle> {
    // Iterate over clients and clients' ports
    let our_id = s.client_id().unwrap();
    let ci = alsa::seq::ClientIter::new(&s);
    let mut port_list = Vec::new();
    for client in ci {
        if client.get_client() == our_id { continue; } // Skip ourselves
        let pi = alsa::seq::PortIter::new(&s, client.get_client());
        for port in pi {
            let caps = port.get_capability();

            // Check that it's a normal input port
            if !caps.contains(alsa::seq::PortCap::READ) || !caps.contains(alsa::seq::PortCap::SUBS_READ) { continue; }
            if !port.get_type().contains(alsa::seq::PortType::MIDI_GENERIC) { continue; }

            info!("Found port: {:?}", port);

            let ph = PortHandle {
                name: String::from(port.get_name().unwrap()),
                client: port.get_client(),
                port: port.get_port(),
            };
            port_list.push(ph)
            
            // Connect source and dest ports
            // let subs = seq::PortSubscribe::empty()?;
            // subs.set_sender(seq::Addr { client: port.get_client(), port: port.get_port() });
            // subs.set_dest(seq::Addr { client: our_id, port: our_port });
            // println!("Reading from midi input {:?}", port);
            // s.subscribe_port(&subs)?;
        }
    }
    port_list
}


async fn handle_lsif() -> Message {
    let s = open_midi_seq();
    let port_list = list_midi_ports(&s);
    let mut response = String::from("lsif\n");
    response.push_str(serde_json::to_string(&port_list).unwrap().as_str());
    Message::text(response)
}

async fn accept_connection(peer: SocketAddr, stream: TcpStream) {
    if let Err(e) = handle_connection(peer, stream).await {
        match e {
            Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
            err => error!("Error processing connection: {}", err),
        }
    }
}

fn grok_command(s: &str) -> (&str, &str) {
    match s.chars().next() {
        Some(c) => s.split_at(c.len_utf8() * 4),
        None => s.split_at(0),
    }
}

async fn handle_connection(peer: SocketAddr, stream: TcpStream) -> Result<()> {
    let ws_stream = accept_async(stream).await.expect("Failed to accept");
    info!("New WebSocket connection: {}", peer);
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    
    // Echo incoming WebSocket messages and send a message periodically every second.

    loop {
        tokio::select! {
            msg = ws_receiver.next() => {
                match msg {
                    Some(msg) => {
                        let msg = msg?;
			if msg.is_close() {
			    info!("closed");
                            break Ok(())
                        }
			let packet = msg.to_text().unwrap();
			let (command, data) = grok_command(packet);
			match command {
			    "lsif" => {
				let data = handle_lsif().await;
				ws_sender.send(data).await?;			
			    }
			    _ => {
				info!("unknown command");
				break Ok(())
			    }
                        }
                    }
                    None => break Ok(()),
                }
            }
        }
    }
}

#[tokio::main]
async fn event_loop() {
    let addr = "127.0.0.1:8123";
    let listener = TcpListener::bind(&addr).await.expect("Can't listen");
    info!("Listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream.peer_addr().expect("connected streams should have a peer address");
        info!("Peer address: {}", peer);

        tokio::spawn(accept_connection(peer, stream));
    }
}

fn main() {
    env_logger::init();

    event_loop();

    info!("main exiting");
}



