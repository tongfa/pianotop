use futures_util::{SinkExt, StreamExt};
use log::*;
use std::{net::SocketAddr, time::Duration};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Error};
use tokio_tungstenite::tungstenite::{Message, Result};
//use futures_util::stream::stream::StreamExt;

async fn accept_connection(peer: SocketAddr, stream: TcpStream) {
    if let Err(e) = handle_connection(peer, stream).await {
        match e {
            Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
            err => error!("Error processing connection: {}", err),
        }
    }
}

async fn handle_connection(peer: SocketAddr, stream: TcpStream) -> Result<()> {
    let ws_stream = accept_async(stream).await.expect("Failed to accept");
    info!("New WebSocket connection: {}", peer);
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let mut interval = tokio::time::interval(Duration::from_millis(1000));

    // Echo incoming WebSocket messages and send a message periodically every second.

    loop {
        tokio::select! {
            msg = ws_receiver.next() => {
                match msg {
                    Some(msg) => {
                        let msg = msg?;
                        if msg.is_text() ||msg.is_binary() {
			    info!("got: {}", msg);
                            ws_sender.send(msg).await?;
                        } else if msg.is_close() {
                            break Ok(())
                        }
                    }
                    None => break Ok(()),
                }
            }
            _ = interval.tick() => {
                ws_sender.send(Message::Text("pong".to_owned())).await?;
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



