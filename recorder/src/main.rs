use futures_util::{SinkExt, StreamExt};
use log::*;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async};
use tokio_tungstenite::tungstenite::{Message, Result};
use tokio_tungstenite::WebSocketStream;
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

fn list_midi_ports(app_state: &Arc<Mutex<AlsaClient>>) -> Vec<PortHandle> {
    // Iterate over clients and clients' ports
    let alsa = app_state.lock().unwrap();
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

fn handle_lstn(alsa: &Arc<Mutex<AlsaClient>>, data: &str) -> Message {
    let port_handle: PortHandle = serde_json::from_str(data).unwrap();

    // let subs = alsa::seq::PortSubscribe::empty().unwrap();
    // subs.set_sender(alsa::seq::Addr { ..port_handle });
    // subs.set_dest(alsa.port);
    // println!("Reading from midi input {:?}", port);
    // alsa.sequencer.subscribe_port(&subs)?;
    Message::text("lstn\n{}")
}

fn handle_lsif(alsa: &Arc<Mutex<AlsaClient>>) -> Message {
    let port_list = list_midi_ports(&alsa);
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

fn handle_socket_message(msg: Message, alsa: &Arc<Mutex<AlsaClient>>) -> Result<Message, &str> {
    let packet = msg.to_text().unwrap();
    let (command, data) = grok_command(packet);
    match command {
	"lsif" => {
	    Ok(handle_lsif(alsa))
	}
	"lstn" => {
	    Ok(handle_lstn(alsa, data))
	}
	_ => {
            Err("unknown command")
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

    let result_room = sequencer.set_client_pool_output_room(SEQUENCER_OUTPUT_ROOM);
    match result_room {
        Ok(_r) => {},
        Err(err) => panic!("allocating sequencer client output room failed: {}", err),
    }

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

    let result = sequencer.set_queue_tempo(queue, &queue_tempo);
    match result {
        Ok(_r) => {},
        Err(err) => panic!("set queue tempo failed: {}", err),
    }

    let port = alsa::seq::Addr {client: sequencer.client_id().unwrap(), port: OUTPUT_PORT};
    AlsaClient {sequencer, port}

}

// #[derive(PartialEq)]
// enum WsConnectionState {
//     Disconnected,
//     Accepted,
//     Connected,
// }


async fn ws_event_loop(app_state: &Arc<Mutex<AlsaClient>>, mut ws_stream: WebSocketStream<TcpStream>) {
    loop {
        let msg = ws_stream.next().await;
        match msg {
            Some(msg) => {
                match msg {
                    Ok(msg) => {
                        if msg.is_close() {
                            error!("websocket closed");
                        }
                        let data = handle_socket_message(msg, app_state);
                        match data {
                            Ok(message) => ws_stream.send(message).await.expect("ws_stream.send failed"),
                            Err(err) => info!("error: {}", err),
                        }
                    }
                    Err(err) => info!("error: {}", err),
                }
            },
            None => info!("error: empty packet"),
        }
    }
}


async fn accept_websocket(app_state: &Arc<Mutex<AlsaClient>>, stream: TcpStream) {
    let s = accept_async(stream).await;
    match s {
        Ok(ws_stream) => {
            info!("New WebSocket connection");
            ws_event_loop(app_state, ws_stream).await;
        },
        _ => error!("Error processing connection"),
    }
}

async fn listen_websocket(app_state: &Arc<Mutex<AlsaClient>>, listener: TcpListener) {
    loop {
        let s = listener.accept().await;
        match s {
            Ok(st) => {
                let stream = st.0;
                let peer = stream.peer_addr().expect("connected streams should have a peer address");
                info!("Peer address: {}", peer);
                accept_websocket(app_state, stream).await;
            },
            Err(err) => info!("listen websocket error: {}", err),
        }
    }
}


#[tokio::main]
async fn run() {
    let addr = "127.0.0.1:8123";
    let listener = TcpListener::bind(&addr).await.expect("Can't listen");
    info!("Listening on: {}", addr);

    let app_state = Arc::new(Mutex::new(open_midi_seq()));

    listen_websocket(&app_state, listener).await;
}

fn main() {
    env_logger::init();

    run();

    info!("main exiting");
}



