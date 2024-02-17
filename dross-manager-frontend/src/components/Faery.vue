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
</div>
</template>

<style scoped>

</style>