import http from "../http-common";

class FaeryDataService {
    getAll() {
        return http.get("/faeries");
    }

    get(id) {
        return http.get(`/faeries/${id}`);
    }

    create(data) {
        return http.post("/faeries", data);
    }

    update(id, data) {
        return http.put(`/faeries/${id}`, data);
    }

    delete(id) {
        return http.delete(`/faeries/${id}`);
    }

    deleteAll() {
        return http.delete(`/faeries`);
    }
}

export default new FaeryDataService();