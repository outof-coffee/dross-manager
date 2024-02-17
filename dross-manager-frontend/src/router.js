import { createWebHistory, createRouter } from "vue-router";

const routes =  [
    {
        path: "/",
        alias: "/faeries",
        name: "faeries",
        component: () => import("./components/FaeryList.vue")
    },
    {
        path: "/faeries/:id",
        name: "faery-edit",
        component: () => import("./components/FaeryEdit.vue")
    },
    {
        path: "/add",
        name: "faery-add",
        component: () => import("./components/FaeryAdd.vue")
    }
];

const router = createRouter({
    history: createWebHistory(),
    routes,
});

export default router;