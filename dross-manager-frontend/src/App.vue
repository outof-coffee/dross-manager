<script setup>
// import Faeries from "@/components/FaeryList.vue";
</script>

<template>
  <div id="app">
    <nav class="navbar navbar-expand navbar-dark bg-dark">
      <router-link to="/" class="navbar-brand">dross manager</router-link>
      <div class="navbar-nav mr-auto">
        <li class="nav-item">
          <router-link to="/faeries" class="nav-link">Faeries List</router-link>
        </li>
        <li class="nav-item">
          <router-link to="/add" class="nav-link">Add Faery</router-link>
        </li>
      </div>
    </nav>
    <div class="container mt-3">
      <router-view />
    </div>
  </div>
</template>