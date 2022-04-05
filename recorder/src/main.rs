use futures_util::{SinkExt, StreamExt};
use log::*;
use std::{net::SocketAddr};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Error};
use tokio_tungstenite::tungstenite::{Message, Result};
use alsa;
use std::ffi::CString;
use std::vec::Vec;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct PortHandle {
    name: String,
    client: i32,
    port: i32,
}

fn list_midi_ports(alsaService: &Arc<Mutex<AlsaClient>>) -> Vec<PortHandle> {
    // Iterate over clients and clients' ports
    let alsa = alsaService.lock().unwrap();
    let ci = alsa::seq::ClientIter::new(&alsa.sequencer);
    let mut port_list = Vec::new();
    for client in ci {
        if client.get_client() == alsa.port.client { continue; } // Skip ourselves
        let pi = alsa::seq::PortIter::new(&alsa.sequencer, client.get_client());
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


async fn handle_lstn(alsa: &Arc<Mutex<AlsaClient>>, data: &str) -> Message {
    let port_handle: PortHandle = serde_json::from_str(data).unwrap();

    // let subs = alsa::seq::PortSubscribe::empty().unwrap();
    // subs.set_sender(alsa::seq::Addr { ..port_handle });
    // subs.set_dest(alsa.port);
    // println!("Reading from midi input {:?}", port);
    // alsa.sequencer.subscribe_port(&subs)?;
    Message::text("lstn\n{}")
}

async fn handle_lsif(alsa: &Arc<Mutex<AlsaClient>>) -> Message {
    let port_list = list_midi_ports(&alsa);
    let mut response = String::from("lsif\n");
    response.push_str(serde_json::to_string(&port_list).unwrap().as_str());
    Message::text(response)
}

fn grok_command(s: &str) -> (&str, &str) {
    match s.chars().next() {
        Some(c) => s.split_at(c.len_utf8() * 4),
        None => s.split_at(0),
    }
}

async fn handle_connection(peer: SocketAddr, stream: TcpStream, alsa: &Arc<Mutex<AlsaClient>>) -> Result<()> {
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
				let data = handle_lsif(alsa).await;
				ws_sender.send(data).await?;			
			    }
			    "lstn" => {
				let data = handle_lstn(alsa, data).await;
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

async fn accept_connection(peer: SocketAddr, stream: TcpStream, alsa: Arc<Mutex<AlsaClient>>) {
    if let Err(e) = handle_connection(peer, stream, &alsa).await {
        match e {
            Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
            err => error!("Error processing connection: {}", err),
        }
    }
}

/* Safety
 * 
 * 
 */

struct AlsaClient {
    sequencer: alsa::Seq,
    port: alsa::seq::Addr,
}

const SEQUENCER_OUTPUT_ROOM: u32 = 500;
const SEQUENCER_BEATS: u32 = 120; // in theory should be variable
const SEQUENCER_TICKS: i32 = 384;
const METRONOME_PORT: i32 = 0;
const OUTPUT_PORT: i32 = 1;

fn open_midi_seq() -> AlsaClient {
    let sequencer = alsa::Seq::open(None, Some(alsa::Direction::Capture), true).unwrap();
    let cstr = CString::new("PianoTop Sequencer").unwrap();
    sequencer.set_client_name(&cstr).unwrap();

    sequencer.set_client_pool_output_room(SEQUENCER_OUTPUT_ROOM);
    // Create a destination port we can read from
    // let mut sub = seq::PortSubscribe::empty().unwrap();
    // dinfo.set_capability(seq::PortCap::WRITE | seq::PortCap::SUBS_WRITE);
    // dinfo.set_type(seq::PortType::MIDI_GENERIC | seq::PortType::APPLICATION);
    // dinfo.set_name(&cstr);
    // dinfo.set_timestamp_queue(1);
    
    // sequencer.create_port(&dinfo).unwrap();
    // let port = dinfo.get_port();    

    let qcstr = CString::new("PianoTop Sequencer").unwrap();
    let queue = sequencer.alloc_named_queue(&qcstr).unwrap();
    let queue_tempo = alsa::seq::QueueTempo::empty().unwrap();
    queue_tempo.set_tempo(6000000 / SEQUENCER_BEATS);
    queue_tempo.set_ppq(SEQUENCER_TICKS);

    sequencer.set_queue_tempo(queue, &queue_tempo);
    let port = alsa::seq::Addr {client: sequencer.client_id().unwrap(), port: OUTPUT_PORT};
    
    AlsaClient {sequencer, port}
}

#[tokio::main]
async fn event_loop() {
    let addr = "127.0.0.1:8123";
    let listener = TcpListener::bind(&addr).await.expect("Can't listen");
    info!("Listening on: {}", addr);

    let alsaClient = Arc::new(Mutex::new(open_midi_seq()));
    
    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream.peer_addr().expect("connected streams should have a peer address");
        info!("Peer address: {}", peer);

        let client = Arc::clone(&alsaClient);
        tokio::spawn(accept_connection(peer, stream, client));
    }
}

fn main() {
    env_logger::init();

    event_loop();

    info!("main exiting");
}



