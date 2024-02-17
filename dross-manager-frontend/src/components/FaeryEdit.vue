<script setup>

</script>

<template>
  <div v-if="currentFaery" class="edit-form">
    <h4>Edit Faery</h4>
    <form>
      <div class="form-group">
        <label for="name">Name</label>
        <input type="text" class="form-control" id="name"
               v-model="currentFaery.name"
        />
      </div>
      <div class="form-group">
        <label for="email">Email</label>
        <input type="text" class="form-control" id="email"
               v-model="currentFaery.email"
        />
      </div>

      <div class="form-group">
        <label for="dross"><strong>Dross:</strong></label>
        <input type="text" class="form-control" id="dross"
               v-model="currentFaery.dross"
               />
      </div>
    </form>

    <button class="badge badge-danger mr-2"
            @click="deleteFaery"
    >
      Delete
    </button>

    <button type="submit" class="badge badge-success"
            @click="updateFaery"
    >
      Update
    </button>
    <p>{{ message }}</p>
  </div>

  <div v-else>
    <br />
    <p>Please click on a Faery...</p>
  </div>

</template>

<script>
import FaeryDataService from "../services/FaeryDataService.js";
export default {
  name: "faery",
  data() {
    return {
      currentFaery: null,
      message: ''
    };
  },
  methods: {
    getFaery(id) {
      FaeryDataService.get(id)
          .then(response => {
            this.currentFaery = response.data;
            console.log(response.data);
          })
          .catch(e => {
            console.log(e);
          });
    },

    updateFaery() {
      FaeryDataService.update(this.currentFaery.id, this.currentFaery)
          .then(response => {
            console.log(response.data);
            this.message = 'The faery was updated successfully!';
          })
          .catch(e => {
            console.log(e);
          });
    },

    deleteFaery() {
      FaeryDataService.delete(this.currentFaery.id)
          .then(response => {
            console.log(response.data);
            this.$router.push({ name: "faeries" });
          })
          .catch(e => {
            console.log(e);
          });
    }
  },
  mounted() {
    this.message = '';
    this.getFaery(this.$route.params.id);
  }
};

</script>

<style>
.edit-form {
  max-width: 300px;
  margin: auto;
}
</style>