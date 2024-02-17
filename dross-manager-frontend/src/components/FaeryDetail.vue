<script setup>
const props = defineProps({
  faery: {
    type: Object,
    required: true
  }
})
</script>

<template>
<div id="{{ faery.id }}">
  <h2>{{ faery.name }}</h2>
  <p><a :href="'mailto:' + faery.email"><em>e-mail</em></a></p>
  <p><em>dross:</em>{{ faery.dross }}</p>
  <router-link :to="'/faeries/' + faery.id" class="btn btn-primary">Edit</router-link>
</div>
</template>

<style>

</style>