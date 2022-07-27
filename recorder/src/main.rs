use futures_util::{SinkExt, StreamExt};
use log::*;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async};
use tokio_tungstenite::WebSocketStream;

mod sequencer;
mod dispatcher;

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
                        let data = dispatcher::dispatch_socket_message(msg);
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


async fn run_webservice() {
    let addr = "127.0.0.1:8123";
    let listener = TcpListener::bind(&addr).await.expect("Can't listen");
    info!("Listening on: {}", addr);
    listen_websocket(listener).await
}

async fn run_sequencer() {
    info!("run sequencer");
    sequencer::record_servicer().await
}

#[tokio::main]
async fn run() {
    let webservice = tokio::spawn(run_webservice());
    let sequencer = tokio::spawn(run_sequencer());
    let (_r1, _r2) = (webservice.await, sequencer.await);
}

fn main() {
    env_logger::init();
    run();
    info!("main exiting");
}



