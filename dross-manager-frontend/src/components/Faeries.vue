<script setup>
import Faery from "./Faery.vue";
</script>

<template>
  <Faery v-for="faery in faeries" :key="faery.id" :faery="faery" />
</template>

<script>
import FaeryDataService from "../services/FaeryDataService.js";

export default {
  name: "faeries-list",
  data() {
    return {
      faeries: []
    }
  },
  mounted() {
    FaeryDataService.getAll()
      .then(response => {
        this.faeries = response.data;
      })
      .catch(e => {
        console.log(e);
      });
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