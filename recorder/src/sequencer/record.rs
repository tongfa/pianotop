use std::future::Future;
use alsa::seq::Event;
use log::info;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::sequencer::CLIENT;

pub struct AlsaEvent { }

impl Future for AlsaEvent {
    type Output = &'static str;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>)
        -> Poll<&'static str>
    {
        let client = CLIENT.lock().unwrap();
        let mut input = client.sequencer.input();
        match input.event_input() {
            Ok(event) => {
                consume_event(event);
                Poll::Ready("done")
            },
            Err(_) => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}

fn consume_event(event: Event) {
    match event.get_type() {
        alsa::seq::EventType::Noteon => {
            info!("noteon: {:?}", event);
        },
        alsa::seq::EventType::Noteoff => {
            info!("noteoff: {:?}", event);
        },
        _ => {},
    }
}
