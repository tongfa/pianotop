use std::{sync::{Mutex}, ffi::CString};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use log::*;

#[derive(Serialize, Deserialize)]
pub struct PortHandle {
    name: String,
    client: i32,
    port: i32,
}


const SEQUENCER_OUTPUT_ROOM: u32 = 500;
const SEQUENCER_BEATS: u32 = 120; // in theory should be variable
const SEQUENCER_TICKS: i32 = 384;
// const METRONOME_PORT: i32 = 0;
const OUTPUT_PORT: i32 = 1;

pub struct AlsaClient {
    pub sequencer: alsa::Seq,
    output_port: alsa::seq::Addr,
    input_port: alsa::seq::Addr,
}

pub static CLIENT: Lazy<Mutex<AlsaClient>> = Lazy::new(|| Mutex::new(create_client()));

fn create_client() -> AlsaClient {
    let sequencer = alsa::Seq::open(None, Some(alsa::Direction::Capture), true).unwrap();
    let cstr = CString::new("PianoTop Sequencer").unwrap();
    sequencer.set_client_name(&cstr).unwrap();

    // allocate a kernel side output buffer for MIDI events
    let result_room = sequencer.set_client_pool_output_room(SEQUENCER_OUTPUT_ROOM);
    match result_room {
        Ok(_r) => {},
        Err(err) => panic!("allocating sequencer client output room failed: {}", err),
    }

    // Create a queue for our input port.
    let qcstr = CString::new("PianoTop Sequencer").unwrap();
    let queue = sequencer.alloc_named_queue(&qcstr).unwrap();

    // Set queue tempo, just setting to an arbitrarily high resolution for now.
    let queue_tempo = alsa::seq::QueueTempo::empty().unwrap();
    queue_tempo.set_tempo(6000000 / SEQUENCER_BEATS);
    queue_tempo.set_ppq(SEQUENCER_TICKS);
    let result = sequencer.set_queue_tempo(queue, &queue_tempo);
    match result {
        Ok(_r) => {},
        Err(err) => panic!("set queue tempo failed: {}", err),
    }


    // Create our input port
    let mut dinfo = alsa::seq::PortInfo::empty().unwrap();
    dinfo.set_capability(
        alsa::seq::PortCap::WRITE |
        alsa::seq::PortCap::SUBS_WRITE |
        alsa::seq::PortCap::READ |
        alsa::seq::PortCap::SUBS_READ);
    dinfo.set_type(
        alsa::seq::PortType::MIDI_GENERIC |
        alsa::seq::PortType::APPLICATION);
    dinfo.set_midi_channels(16);
    dinfo.set_timestamping(true);
    dinfo.set_timestamp_real(true);
    dinfo.set_timestamp_queue(queue);
    dinfo.set_name(&cstr);

    sequencer.create_port(&dinfo).unwrap();
    let input_port = alsa::seq::Addr {client: sequencer.client_id().unwrap(), port: dinfo.get_port()};

    let output_port = alsa::seq::Addr {client: sequencer.client_id().unwrap(), port: OUTPUT_PORT};
    AlsaClient {sequencer, output_port, input_port}

}

pub fn list_midi_ports() -> Vec<PortHandle> {
    // Iterate over clients and clients' ports
    let alsa = CLIENT.lock().unwrap();
    let ci = alsa::seq::ClientIter::new(&alsa.sequencer);
    let mut port_list = Vec::new();
    for client in ci {
        if client.get_client() == alsa.output_port.client { continue; } // Skip ourselves
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

pub fn listen(selected_port_handle: PortHandle) {
    let selected_port_addr = alsa::seq::Addr {
        client: selected_port_handle.client,
        port: selected_port_handle.port,
    };
    let alsa = CLIENT.lock().unwrap();
    let subs = alsa::seq::PortSubscribe::empty().unwrap();
    subs.set_sender(selected_port_addr);
    // dest is "us"
    subs.set_dest(alsa.input_port);
    subs.set_time_update(true);
    subs.set_time_real(true);
    println!("Reading from midi output {:?}", selected_port_addr);
    let result = alsa.sequencer.subscribe_port(&subs);
    match result {
        Ok(()) => {},
        Err(err) => {
            panic!("failed to subscribe to ALSA ports: {}", err)
        }
    }
}
