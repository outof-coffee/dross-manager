<script setup>
import FaeryDetail from "./FaeryDetail.vue";
</script>

<template>
  <div class="list row">
<!-- TODO: Implement search by name -->
<!--    <div class="col-md-8">-->
<!--      <div class="input-group mb-3">-->
<!--        <input type="text" class="form-control" placeholder="Search by name"-->
<!--               v-model="name"/>-->
<!--        <div class="input-group-append">-->
<!--          <button class="btn btn-outline-secondary" type="button"-->
<!--                  @click="searchName"-->
<!--          >-->
<!--            Search-->
<!--          </button>-->
<!--        </div>-->
<!--      </div>-->
<!--    </div>-->
    <div class="col-md-6">
      <h4>Faery List</h4>
      <ul class="list-group">
        <li class="list-group-item"
            :class="{ active: index == currentIndex }"
            v-for="(faery, index) in faeries"
            :key="faery.id"
            @click="setActiveFaery(faery, index)"
        >
          {{ faery.name }}
        </li>
      </ul>
    </div>
    <div class="col-md-6">
      <div v-if="currentFaery">
        <FaeryDetail :faery="currentFaery" />
        <router-link :to="'/faeries/' + currentFaery.id" class="badge badge-warning">Edit</router-link>
      </div>
      <div v-else>
        <br />
        <p>Please click on a Faery...</p>
      </div>
    </div>
  </div>
</template>

<script>
import FaeryDataService from "../services/FaeryDataService.js";

export default {
  name: "faeries-list",
  data() {
    return {
      faeries: [],
      currentFaery: null,
      currentIndex: -1,
      name: ""
    }
  },
  mounted() {
    this.retrieveFaeries();
  },
  methods: {
    retrieveFaeries() {
      FaeryDataService.getAll()
        .then(response => {
          this.faeries = response.data;
        })
        .catch(e => {
          console.log(e);
        });
    },
    refreshList() {
      this.retrieveFaeries();
      this.currentFaery = null;
      this.currentIndex = -1;
    },
    setActiveFaery(faery, index) {
      this.currentFaery = faery;
      this.currentIndex = faery ? index : -1;
    }// , // TODO: Implement search by name
    // searchName() {
    //   FaeryDataService.findByName(this.name)
    //     .then(response => {
    //       this.faeries = response.data;
    //     })
    //     .catch(e => {
    //       console.log(e);
    //     });
    // }
  }
}
</script>

<style>
.list {
  text-align: left;
  max-width: 750px;
  margin: auto;
}
</style>