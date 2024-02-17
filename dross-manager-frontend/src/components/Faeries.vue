<script setup>
</script>

<template>
  <div v-for="faery in faeries" :key="faery.id">
    <h2>{{ faery.name }}</h2>
    <p><a :href="'mailto:' + faery.email"><em>e-mail</em></a></p>
    <p><em>dross:</em>{{ faery.dross }}</p>
  </div></template>

<script>
import axios from 'axios'

export default {
  data() {
    return {
      faeries: []
    }
  },
  mounted() {
    axios.get('/faeries')
        .then(response => {
          this.faeries = response.data
        })
        .catch(error => {
          console.log(error)
        })
  }
}
</script>

<style scoped>
h1 {
  font-weight: 500;
  font-size: 2.6rem;
  position: relative;
  top: -10px;
}

h3 {
  font-size: 1.2rem;
}

.greetings h1,
.greetings h3 {
  text-align: center;
}

@media (min-width: 1024px) {
  .greetings h1,
  .greetings h3 {
    text-align: left;
  }
}

</style>