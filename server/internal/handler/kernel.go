package handler

import (
	"net/http"
)

type KernelStatusResponse struct {
	Running bool   `json:"running"`
	Pid     int    `json:"pid,omitempty"`
	Version string `json:"version,omitempty"`
}

func (h *Handler) StartKernel(w http.ResponseWriter, r *http.Request) {
	if err := h.kernel.Restart(); err != nil {
		http.Error(w, "kernel start failed: "+err.Error(), http.StatusInternalServerError)
		return
	}
	writeJSON(w, http.StatusOK, map[string]string{"status": "started"})
}

func (h *Handler) StopKernel(w http.ResponseWriter, r *http.Request) {
	if err := h.kernel.Stop(); err != nil {
		http.Error(w, "kernel stop failed: "+err.Error(), http.StatusInternalServerError)
		return
	}
	writeJSON(w, http.StatusOK, map[string]string{"status": "stopped"})
}

func (h *Handler) GetKernelStatus(w http.ResponseWriter, r *http.Request) {
	status := KernelStatusResponse{Running: true}
	if err := h.kernel.Ping(); err != nil {
		status = KernelStatusResponse{Running: false}
	}
	writeJSON(w, http.StatusOK, status)
}
