<template>
  <div class="events">
    <h1>Events</h1>
    <div class="side-by-side">
      <div>
        <button v-on:click="play()">play</button>
        <button v-on:click="transpose(1)">transpose up</button>
        <button v-on:click="transpose(-1)">transpose down</button>
        <button v-on:click="clear()">clear</button>
      </div>
      <div>
        <div class="note-list" v-for="event in events" :key="event.index">
          {{ event.message }}
          {{ event.time }}
          {{ event.pitch }}
          {{ event.velocity }}
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { Component, Vue } from 'vue-property-decorator'
import { PPAlsaDevice, ppConnection, PPMidiEvent } from '../ws/pianoplay';


@Component
export default class ChooseDevice extends Vue {
  pp = ppConnection
  select(device: PPAlsaDevice): void {
    this.pp.listen(device)
  }
  get events(): PPMidiEvent[] {
    return this.$root.$data.events
  }
  play() {
    this.pp.play()
  }
  transpose(v: any) {
    this.pp.transpose(v)
  }
  clear() {
    this.pp.clear()
  }
}
</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped>
ul {
  list-style-type: none;
  padding: 0;
}
li {
  margin: 0 10px;
  color: #42b983;
}
.side-by-side {
  display: flex;
  flex-flow: column nowrap;
}
</style>
