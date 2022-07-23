use futures_util::{SinkExt, StreamExt};
use log::*;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async};
use tokio_tungstenite::tungstenite::{Message, Result};
use tokio_tungstenite::WebSocketStream;

mod sequencer;

fn handle_lstn(data: &str) -> Message {
    let selected_port_handle: sequencer::PortHandle = serde_json::from_str(data).unwrap();
    sequencer::listen(selected_port_handle);
    Message::text("lstn\n{}")
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

fn handle_socket_message(msg: Message) -> Result<Message, &'static str> {
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


/* Safety
 *
 *
 */



// #[derive(PartialEq)]
// enum WsConnectionState {
//     Disconnected,
//     Accepted,
//     Connected,
// }


async fn ws_event_loop(mut ws_stream: WebSocketStream<TcpStream>) {
    loop {
        let msg = ws_stream.next().await;
        match msg {
            Some(msg) => {
                match msg {
                    Ok(msg) => {
                        if msg.is_close() {
                            error!("websocket closed");
                        }
                        let data = handle_socket_message(msg);
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


async fn accept_websocket(stream: TcpStream) {
    let s = accept_async(stream).await;
    match s {
        Ok(ws_stream) => {
            info!("New WebSocket connection");
            ws_event_loop(ws_stream).await;
        },
        _ => error!("Error processing connection"),
    }
}

async fn listen_websocket(listener: TcpListener) {
    loop {
        let s = listener.accept().await;
        match s {
            Ok(st) => {
                let stream = st.0;
                let peer = stream.peer_addr().expect("connected streams should have a peer address");
                info!("Peer address: {}", peer);
                accept_websocket(stream).await;
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
    listen_websocket(listener).await;
}

fn main() {
    env_logger::init();
    run();
    info!("main exiting");
}



