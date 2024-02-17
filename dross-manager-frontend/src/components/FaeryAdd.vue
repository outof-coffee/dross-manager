<script setup>

</script>

<template>
  <div class="submit-form">
    <div v-if="!submitted">
      <div class="form-group">
        <label for="name">Faery Name</label>
        <input
            type="text"
            class="form-control"
            id="name"
            required
            v-model="faery.name"
            name="name"
        />
      </div>

      <div class="form-group">
        <label for="email">Email</label>
        <input
            class="form-control"
            id="email"
            required
            v-model="faery.email"
            name="email"
        />
      </div>

      <button @click="saveFaery" class="btn btn-success">Submit</button>
    </div>

    <div v-else>
      <h4>{{ message }}</h4>
      <button class="btn btn-success" @click="newFaery">Add Another</button>
    </div>
  </div>
</template>

<script>
import FaeryDataService from "../services/FaeryDataService.js";
export default {
  name: "faery-add",
  data() {
    return {
      faery: {
        id: null,
        name: "",
        email: "",
      },
      submitted: false,
      message: ""
    };
  },
  methods: {
    saveFaery() {
      const data = {
        name: this.faery.name,
        email: this.faery.email,
        dross: 0,
        is_admin: false
      };

      FaeryDataService.create(data)
          .then(response => {
            this.faery.id = response.data.id;
            console.log(response.data);
            this.submitted = true;
            this.message = "Faery created successfully!";
          })
          .catch(e => {
            console.log(e);
          });
    },

    newFaery() {
      this.submitted = false;
      this.faery = {};
      this.message = "";
    }
  }
};
</script>

<style>
.submit-form {
  max-width: 300px;
  margin: auto;
}
</style>