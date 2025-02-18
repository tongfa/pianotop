pub mod alsa_client;

mod record;
pub async fn record_servicer() {
    loop {
        let f = record::AlsaEvent {};
        f.await;
    }
}

// static void record_event_realtime(const snd_seq_event_t *ev)
// {
//   long time;
//   struct sequencer_work_message message;

// 	switch (ev->type) {
// 	case SND_SEQ_EVENT_NOTEON:
// 	case SND_SEQ_EVENT_NOTEOFF:
//     message.time = ev->time.time.tv_nsec + SEQR_NS_PER_SEC * ev->time.time.tv_sec;
//     tell_ui(
//       SND_SEQ_EVENT_NOTEON == ev->type ? "note_on" : "note_off",
//       message.time,
//       ev->data.note.note & 0x7f,
//       ev->data.note.velocity & 0x7f
//     );
//     message.type = SND_SEQ_EVENT_NOTEON == ev->type ? SQWK_NOTE_ON : SQWK_NOTE_OFF,
//     message.channel = (ev->data.note.channel & 0xf);
//     message.note.pitch = (ev->data.note.note & 0x7f);
//     message.note.velocity = (ev->data.note.velocity & 0x7f);
//     sequencer_work_insert_message(&message);
//     sqfl_write(&message);
// 		break;
//   }
// }
