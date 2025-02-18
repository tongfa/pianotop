# Pianotop

This is a simple web based MIDI recorder.

The UI is written in Vue. The backend is written in Rust.

This is very much a WIP project. It's not capable of sending notes to the UI yet. But
it is capable of:
* reading a list of midi interfaces in the backend, and presenting them in the UI
* selecting a midi interface in the UI, and the backend starts listening for events.
* the backend is able to see MIDI noteon and noteoff messagages, but does not yet aggregate
  them and does not send them to the UI.

I'm not sure what the project is going to be yet. I think what I want is to hook up a small
computer to my keyboard, would be nice if it was the size of a USB dongle. Then over Wifi
control it from my phone. Once its plugged in, it starts recording, and doesn't stop until
unplugged. It divides all the music into works separated by roughly 15 second breaks in time.

Then I'll have a quick way to record all my public peformances in MIDI with minimum effort!

Then would be nice to be able to edit the midi and play it back on the keyboard.

# Required dependencies

```
apt install build-essential libasound2-dev
```

# development

## if you do not have an ALSA device connected

First, you need some kind of ALSA device to connect to.  In case you don't have one
this will setup a virtual interface:

```
to setup 2 virtual interfaces:
`sudo modprobe snd_virmidi midi_devs=1`
```

There are scripts in ./test/ to send notes to a virtual ALSA device

## running the parts

```
(cd recorder ; RUST_LOG=info cargo run)
```

then in another shell:

```
(cd ui ; yarn serve)
```

Then open a browser on the URL from yarn and away you go!

## TODO list

* organize recordings as bundles, bundles defined as groups of notes separated by 15 seconds of silence
* each bundle is saved as file
* can select a file
* edits are "copy on save", files never overwritten.

* metronome
 * high hat based on click track
 * lead in 2 bars (one sound), then ta da da da
 * time sig
 * tempo

* piano roll display
 * start / end markers, can be adjusted
 * region start / end markers
 * playhead
 * can select notes
   * drag selection box
   * add to selection list with modifier
   * select by "take"
 * delete note(s) (cut)
 * copy / paste note(s)
 * subsequent pastes are pasted automatically at time intervals equal to first paste / original interval.
 * change pitch / velocity of note(s)
 * change start / end of a note (does not work with selection of > 1 note)
 * add note
 * insert space
 * stretch / shrink time over selected region - can use curve control

 * punch in / out - you just delete what you don't want to hear, then do your take.

* note list
