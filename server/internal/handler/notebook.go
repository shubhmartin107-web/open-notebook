package handler

import (
	"encoding/json"
	"net/http"

	"github.com/open-notebook/server/internal/kernel"
	"github.com/open-notebook/server/internal/notebook"
)

type Handler struct {
	store  *notebook.Store
	kernel *kernel.Client
}

func NewHandler(store *notebook.Store, k *kernel.Client) *Handler {
	return &Handler{store: store, kernel: k}
}

type CreateNotebookRequest struct {
	Title string `json:"title"`
}

type AddCellRequest struct {
	Kind   notebook.CellKind `json:"kind"`
	Source string            `json:"source"`
}

type UpdateCellRequest struct {
	Source string `json:"source"`
}

type SaveNotebookRequest struct {
	Path string `json:"path"`
}

func (h *Handler) CreateNotebook(w http.ResponseWriter, r *http.Request) {
	var req CreateNotebookRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "invalid request body", http.StatusBadRequest)
		return
	}
	nb := h.store.Create(req.Title)
	writeJSON(w, http.StatusCreated, nb)
}

func (h *Handler) ListNotebooks(w http.ResponseWriter, r *http.Request) {
	nbs := h.store.List()
	writeJSON(w, http.StatusOK, nbs)
}

func (h *Handler) GetNotebook(w http.ResponseWriter, r *http.Request) {
	id := r.PathValue("id")
	nb, ok := h.store.Get(id)
	if !ok {
		http.Error(w, "notebook not found", http.StatusNotFound)
		return
	}
	writeJSON(w, http.StatusOK, nb)
}

func (h *Handler) UpdateNotebook(w http.ResponseWriter, r *http.Request) {
	id := r.PathValue("id")
	var req CreateNotebookRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "invalid request body", http.StatusBadRequest)
		return
	}
	nb, ok := h.store.Update(id, req.Title)
	if !ok {
		http.Error(w, "notebook not found", http.StatusNotFound)
		return
	}
	writeJSON(w, http.StatusOK, nb)
}

func (h *Handler) DeleteNotebook(w http.ResponseWriter, r *http.Request) {
	id := r.PathValue("id")
	if !h.store.Delete(id) {
		http.Error(w, "notebook not found", http.StatusNotFound)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func (h *Handler) AddCell(w http.ResponseWriter, r *http.Request) {
	id := r.PathValue("id")
	var req AddCellRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "invalid request body", http.StatusBadRequest)
		return
	}
	cell, ok := h.store.AddCell(id, req.Kind, req.Source)
	if !ok {
		http.Error(w, "notebook not found", http.StatusNotFound)
		return
	}
	writeJSON(w, http.StatusCreated, cell)
}

func (h *Handler) UpdateCell(w http.ResponseWriter, r *http.Request) {
	nbID := r.PathValue("id")
	cellID := r.PathValue("cellId")
	var req UpdateCellRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "invalid request body", http.StatusBadRequest)
		return
	}
	cell, ok := h.store.UpdateCell(nbID, cellID, req.Source)
	if !ok {
		http.Error(w, "cell not found", http.StatusNotFound)
		return
	}
	writeJSON(w, http.StatusOK, cell)
}

func (h *Handler) RemoveCell(w http.ResponseWriter, r *http.Request) {
	nbID := r.PathValue("id")
	cellID := r.PathValue("cellId")
	if !h.store.RemoveCell(nbID, cellID) {
		http.Error(w, "cell not found", http.StatusNotFound)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func (h *Handler) GetCell(w http.ResponseWriter, r *http.Request) {
	nbID := r.PathValue("id")
	cellID := r.PathValue("cellId")
	nb, ok := h.store.Get(nbID)
	if !ok {
		http.Error(w, "notebook not found", http.StatusNotFound)
		return
	}
	for _, c := range nb.Cells {
		if c.ID == cellID {
			writeJSON(w, http.StatusOK, c)
			return
		}
	}
	http.Error(w, "cell not found", http.StatusNotFound)
}

func (h *Handler) ExecuteNotebook(w http.ResponseWriter, r *http.Request) {
	id := r.PathValue("id")
	nb, ok := h.store.Get(id)
	if !ok {
		http.Error(w, "notebook not found", http.StatusNotFound)
		return
	}

	cellIDs := make([]string, len(nb.Cells))
	for i, c := range nb.Cells {
		cellIDs[i] = c.ID
	}

	result, err := h.kernel.Execute(nb, cellIDs)
	if err != nil {
		http.Error(w, "execution failed: "+err.Error(), http.StatusInternalServerError)
		return
	}

	for _, cr := range result.Results {
		h.store.SetCellOutput(id, cr.CellID, &notebook.CellOutput{
			Stdout:   cr.Stdout,
			Stderr:   cr.Stderr,
			ExitCode: cr.ExitCode,
		})
	}

	nb, _ = h.store.Get(id)
	writeJSON(w, http.StatusOK, nb)
}

func (h *Handler) ExecuteCell(w http.ResponseWriter, r *http.Request) {
	nbID := r.PathValue("id")
	cellID := r.PathValue("cellId")

	nb, ok := h.store.Get(nbID)
	if !ok {
		http.Error(w, "notebook not found", http.StatusNotFound)
		return
	}

	found := false
	for _, c := range nb.Cells {
		if c.ID == cellID {
			found = true
			break
		}
	}
	if !found {
		http.Error(w, "cell not found", http.StatusNotFound)
		return
	}

	result, err := h.kernel.Execute(nb, []string{cellID})
	if err != nil {
		http.Error(w, "execution failed: "+err.Error(), http.StatusInternalServerError)
		return
	}

	for _, cr := range result.Results {
		h.store.SetCellOutput(nbID, cr.CellID, &notebook.CellOutput{
			Stdout:   cr.Stdout,
			Stderr:   cr.Stderr,
			ExitCode: cr.ExitCode,
		})
	}

	nb, _ = h.store.Get(nbID)
	writeJSON(w, http.StatusOK, nb)
}

func (h *Handler) GetDAG(w http.ResponseWriter, r *http.Request) {
	id := r.PathValue("id")
	nb, ok := h.store.Get(id)
	if !ok {
		http.Error(w, "notebook not found", http.StatusNotFound)
		return
	}

	dag, err := h.kernel.Dag(nb)
	if err != nil {
		http.Error(w, "dag computation failed: "+err.Error(), http.StatusInternalServerError)
		return
	}

	writeJSON(w, http.StatusOK, dag)
}

func (h *Handler) SaveNotebook(w http.ResponseWriter, r *http.Request) {
	id := r.PathValue("id")
	nb, ok := h.store.Get(id)
	if !ok {
		http.Error(w, "notebook not found", http.StatusNotFound)
		return
	}

	var req SaveNotebookRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "invalid request body", http.StatusBadRequest)
		return
	}

	if err := h.kernel.Save(req.Path, nb); err != nil {
		http.Error(w, "save failed: "+err.Error(), http.StatusInternalServerError)
		return
	}

	writeJSON(w, http.StatusOK, map[string]string{"status": "saved"})
}

func (h *Handler) SyncNotebook(w http.ResponseWriter, r *http.Request) {
	http.Error(w, "not implemented", http.StatusNotImplemented)
}

func (h *Handler) AiGenerate(w http.ResponseWriter, r *http.Request) {
	id := r.PathValue("id")
	nb, ok := h.store.Get(id)
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

	result, err := h.kernel.AiGenerate(nb, req.CellIndex, req.Prompt)
	if err != nil {
		http.Error(w, "AI generation failed: "+err.Error(), http.StatusInternalServerError)
		return
	}

	writeJSON(w, http.StatusOK, map[string]string{"result": result})
}

func writeJSON(w http.ResponseWriter, status int, v interface{}) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	json.NewEncoder(w).Encode(v)
}
