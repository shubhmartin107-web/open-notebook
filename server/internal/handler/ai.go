package handler

import (
	"net/http"
)

type AiStatusResponse struct {
	Available bool   `json:"available"`
	Model     string `json:"model"`
	Provider  string `json:"provider"`
}

func (h *Handler) AiStatus(w http.ResponseWriter, r *http.Request) {
	available := true
	if err := h.kernel.Ping(); err != nil {
		available = false
	}
	writeJSON(w, http.StatusOK, AiStatusResponse{
		Available: available,
		Model:     "qwen2.5-coder:7b",
		Provider:  "ollama",
	})
}
