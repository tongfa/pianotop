<template>
  <div class="choose-device">
    <h1>Choose an ALSA MIDI input device:</h1>
    <ul>
      <li v-for="device in interfaces" :key="device.name" v-on:click="select(device)">{{ device.name }}</li>
    </ul>
  </div>
</template>

<script lang="ts">
import { Component, Vue } from 'vue-property-decorator'
import { PPAlsaDevice, ppConnection } from '../ws/pianoplay';


@Component
export default class ChooseDevice extends Vue {
  pp = ppConnection
  select(device: PPAlsaDevice): void {
    this.pp.listen(device)
  }
  get interfaces() {
    return this.$root.$data.interfaces
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
</style>
