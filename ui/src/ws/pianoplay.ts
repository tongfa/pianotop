
export interface PPAlsaDevice {
  handle: string
  name: string
}

export enum PPMidiMessage {
  NoteOn = 'note_on',
  NoteOff = 'note_off'
}

export interface PPMidiEvent {
  time: number,
  message: PPMidiMessage,
  pitch: number,
  velocity: number,
}

interface everything {
  currentDevice: PPAlsaDevice| null
  interfaces: Array<PPAlsaDevice>
  events: Array<PPMidiEvent>
}

class PianoPlayService {
  data: everything = {
    currentDevice: null,
    interfaces: new Array<PPAlsaDevice>(),
    events: new Array<PPMidiEvent>()
  }

  constructor(private ws: WebSocket) {
    this.ws.onmessage = (event) => {
      const message: string = event.data
      console.log(message)
      const messageType = message.substring(0, 4)
      const payload = message.substring(5)
      switch(messageType) {
        case 'pong':
          console.log(message)
          break
        case 'lsif':
          this.data.interfaces = JSON.parse(payload)
          break
        case 'midi':
          console.log('got midi', payload)
          this.data.events.push(JSON.parse(payload))
          break
        case 'lstn':
          console.log('listening on', JSON.parse(payload))
          break
        default:
          console.log('unknown ws event', event.data);
      }
    }
    this.ws.onopen = () => {
      this.ws.send('lsif')
    }
  }
  listen(device: PPAlsaDevice) {
    console.log('setting device')
    this.data.currentDevice = device
    this.ws.send(`lstn\n${JSON.stringify([device.handle])}`)
  }
  play() {
    let timeoffset = 0;
    this.data.events.forEach(e => {
      if (timeoffset === 0 ) {
        timeoffset = e.time;
      }
      setTimeout(() => {
        this.ws.send(`midi\n${JSON.stringify(e)}`)
      }, (e.time - timeoffset) * 39 )
    })
  }
  transpose(pitchDelta: number) {
    this.data.events.forEach(e => {
      e.pitch += pitchDelta
    });
  }
  clear() {
    this.data.events = new Array<PPMidiEvent>();
  }
}

const ppConnection = new PianoPlayService(new WebSocket('ws://localhost:8123/', 'pianoplay-protocol'))
export { ppConnection }