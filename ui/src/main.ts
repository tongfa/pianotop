import Vue from 'vue'
import App from './App.vue'
import { ppConnection } from './ws/pianoplay';

Vue.config.productionTip = false

const vm = new Vue({
  render: h => h(App),
  data: ppConnection.data
}).$mount('#app')
