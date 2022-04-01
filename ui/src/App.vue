<template>
  <div id="app">
    <ChooseDevice v-if="!device" />
    <EventDisplay v-if="device" />
  </div>
</template>

<script lang="ts">
import { Component, Vue } from 'vue-property-decorator';
import ChooseDevice from './components/ChooseDevice.vue'
import EventDisplay from './components/EventDisplay.vue'

@Component({
  components: {
    ChooseDevice,
    EventDisplay,
  },
})
export default class App extends Vue {
  get device() {
    return !!this.$root.$data.currentDevice
  }
}
</script>

<style>
#app {
  font-family: Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  color: #2c3e50;
  margin-top: 60px;
  display: flex;
  justify-content: center;
}
</style>
