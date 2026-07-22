package handler

import (
	"encoding/json"
	"fmt"
	"net/http"
	"time"

	"github.com/open-notebook/server/internal/notebook"
)

func (h *Handler) ExecuteNotebookStream(w http.ResponseWriter, r *http.Request) {
	id := r.PathValue("id")
	nb, ok := h.store.Get(id)
	if !ok {
		http.Error(w, "notebook not found", http.StatusNotFound)
		return
	}

	flusher, ok := w.(http.Flusher)
	if !ok {
		http.Error(w, "streaming not supported", http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "text/event-stream")
	w.Header().Set("Cache-Control", "no-cache")
	w.Header().Set("Connection", "keep-alive")

	cellIDs := make([]string, len(nb.Cells))
	for i, c := range nb.Cells {
		cellIDs[i] = c.ID
	}

	for _, cellID := range cellIDs {
		select {
		case <-r.Context().Done():
			return
		default:
		}

		fmt.Fprintf(w, "data: %s\n\n", jsonBytes(map[string]interface{}{
			"cell_id": cellID, "status": "running",
		}))
		flusher.Flush()

		result, err := h.kernel.Execute(nb, []string{cellID})
		if err != nil {
			fmt.Fprintf(w, "data: %s\n\n", jsonBytes(map[string]interface{}{
				"cell_id": cellID, "status": "error", "error": err.Error(),
			}))
			flusher.Flush()
			continue
		}

		for _, cr := range result.Results {
			h.store.SetCellOutput(id, cr.CellID, &notebook.CellOutput{
				Stdout: cr.Stdout, Stderr: cr.Stderr, ExitCode: cr.ExitCode,
			})
			status := "success"
			if cr.ExitCode != 0 {
				status = "error"
			}
			fmt.Fprintf(w, "data: %s\n\n", jsonBytes(map[string]interface{}{
				"cell_id": cr.CellID, "status": status,
				"stdout": cr.Stdout, "stderr": cr.Stderr,
			}))
			flusher.Flush()
		}
	}

	fmt.Fprintf(w, "data: %s\n\n", jsonBytes(map[string]interface{}{"done": true}))
	flusher.Flush()
}

func (h *Handler) AiGenerateStream(w http.ResponseWriter, r *http.Request) {
	id := r.PathValue("id")
	_, ok := h.store.Get(id)
	if !ok {
		http.Error(w, "notebook not found", http.StatusNotFound)
		return
	}

	var req struct {
		CellIndex int    `json:"cell_index"`
		Prompt    string `json:"prompt"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "invalid request body", http.StatusBadRequest)
		return
	}

	flusher, ok := w.(http.Flusher)
	if !ok {
		http.Error(w, "streaming not supported", http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "text/event-stream")
	w.Header().Set("Cache-Control", "no-cache")
	w.Header().Set("Connection", "keep-alive")

	result, err := h.kernel.AiGenerate(nil, req.CellIndex, req.Prompt)
	if err != nil {
		fmt.Fprintf(w, "data: %s\n\n", jsonBytes(map[string]interface{}{"error": err.Error()}))
		flusher.Flush()
		return
	}

	for i := 0; i < len(result); i += 50 {
		select {
		case <-r.Context().Done():
			return
		default:
		}
		end := i + 50
		if end > len(result) {
			end = len(result)
		}
		fmt.Fprintf(w, "data: %s\n\n", jsonBytes(map[string]interface{}{"token": result[i:end]}))
		flusher.Flush()
		time.Sleep(10 * time.Millisecond)
	}

	fmt.Fprintf(w, "data: %s\n\n", jsonBytes(map[string]interface{}{"done": true}))
	flusher.Flush()
}

func jsonBytes(v interface{}) []byte {
	b, _ := json.Marshal(v)
	return b
}
