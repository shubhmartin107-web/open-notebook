package handler

import (
	"fmt"
	"net/http"
	"time"
)

func (h *Handler) NotebookEvents(w http.ResponseWriter, r *http.Request) {
	flusher, ok := w.(http.Flusher)
	if !ok {
		http.Error(w, "streaming not supported", http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "text/event-stream")
	w.Header().Set("Cache-Control", "no-cache")
	w.Header().Set("Connection", "keep-alive")

	fmt.Fprintf(w, "data: {\"type\":\"connected\"}\n\n")
	flusher.Flush()

	notify := r.Context().Done()
	for {
		select {
		case <-notify:
			return
		default:
			fmt.Fprintf(w, ": heartbeat\n\n")
			flusher.Flush()
			select {
			case <-notify:
				return
			case <-time.After(30 * time.Second):
			}
		}
	}
}
