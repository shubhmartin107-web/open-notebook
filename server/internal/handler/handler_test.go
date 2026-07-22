package handler

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"github.com/open-notebook/server/internal/notebook"
)

func newTestHandler() *Handler {
	store := notebook.NewStore()
	return NewHandler(store, nil)
}

func TestHealth(t *testing.T) {
	r := httptest.NewRequest("GET", "/api/health", nil)
	w := httptest.NewRecorder()

	Health(w, r)

	resp := w.Result()
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("expected 200, got %d", resp.StatusCode)
	}

	var body map[string]string
	json.NewDecoder(resp.Body).Decode(&body)
	if body["status"] != "ok" {
		t.Fatalf("expected status ok, got %q", body["status"])
	}
}

func TestCreateNotebook(t *testing.T) {
	h := newTestHandler()
	body := `{"title":"test notebook"}`
	r := httptest.NewRequest("POST", "/api/notebooks", strings.NewReader(body))
	r.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()

	h.CreateNotebook(w, r)

	resp := w.Result()
	if resp.StatusCode != http.StatusCreated {
		t.Fatalf("expected 201, got %d", resp.StatusCode)
	}

	var nb notebook.Notebook
	json.NewDecoder(resp.Body).Decode(&nb)
	if nb.Title != "test notebook" {
		t.Fatalf("expected title 'test notebook', got %q", nb.Title)
	}
	if nb.ID == "" {
		t.Fatal("expected non-empty ID")
	}
}

func TestCreateNotebookInvalidBody(t *testing.T) {
	h := newTestHandler()
	r := httptest.NewRequest("POST", "/api/notebooks", strings.NewReader(`invalid`))
	r.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()

	h.CreateNotebook(w, r)

	resp := w.Result()
	if resp.StatusCode != http.StatusBadRequest {
		t.Fatalf("expected 400, got %d", resp.StatusCode)
	}
}

func TestGetNotebook(t *testing.T) {
	h := newTestHandler()

	nb := h.store.Create("get test")

	r := httptest.NewRequest("GET", "/api/notebooks/"+nb.ID, nil)
	r.SetPathValue("id", nb.ID)
	w := httptest.NewRecorder()
	h.GetNotebook(w, r)

	resp := w.Result()
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("expected 200, got %d", resp.StatusCode)
	}

	var got notebook.Notebook
	json.NewDecoder(resp.Body).Decode(&got)
	if got.ID != nb.ID {
		t.Fatalf("expected ID %q, got %q", nb.ID, got.ID)
	}
}

func TestGetNotebookNotFound(t *testing.T) {
	h := newTestHandler()
	r := httptest.NewRequest("GET", "/api/notebooks/nonexistent", nil)
	r.SetPathValue("id", "nonexistent")
	w := httptest.NewRecorder()

	h.GetNotebook(w, r)

	resp := w.Result()
	if resp.StatusCode != http.StatusNotFound {
		t.Fatalf("expected 404, got %d", resp.StatusCode)
	}
}

func TestDeleteNotebook(t *testing.T) {
	h := newTestHandler()
	nb := h.store.Create("delete test")

	r := httptest.NewRequest("DELETE", "/api/notebooks/"+nb.ID, nil)
	r.SetPathValue("id", nb.ID)
	w := httptest.NewRecorder()
	h.DeleteNotebook(w, r)

	resp := w.Result()
	if resp.StatusCode != http.StatusNoContent {
		t.Fatalf("expected 204, got %d", resp.StatusCode)
	}
}

func TestDeleteNotebookNotFound(t *testing.T) {
	h := newTestHandler()
	r := httptest.NewRequest("DELETE", "/api/notebooks/nonexistent", nil)
	r.SetPathValue("id", "nonexistent")
	w := httptest.NewRecorder()

	h.DeleteNotebook(w, r)

	resp := w.Result()
	if resp.StatusCode != http.StatusNotFound {
		t.Fatalf("expected 404, got %d", resp.StatusCode)
	}
}

func TestAddCell(t *testing.T) {
	h := newTestHandler()
	nb := h.store.Create("cell test")

	cellBody := `{"kind":"code","source":"print(1)"}`
	r := httptest.NewRequest("POST", "/api/notebooks/"+nb.ID+"/cells", strings.NewReader(cellBody))
	r.Header.Set("Content-Type", "application/json")
	r.SetPathValue("id", nb.ID)
	w := httptest.NewRecorder()
	h.AddCell(w, r)

	resp := w.Result()
	if resp.StatusCode != http.StatusCreated {
		t.Fatalf("expected 201, got %d", resp.StatusCode)
	}

	var cell notebook.Cell
	json.NewDecoder(resp.Body).Decode(&cell)
	if cell.Source != "print(1)" {
		t.Fatalf("expected source 'print(1)', got %q", cell.Source)
	}
	if cell.Kind != notebook.CellKindCode {
		t.Fatalf("expected kind 'code', got %q", cell.Kind)
	}
}

func TestAddCellNotFound(t *testing.T) {
	h := newTestHandler()
	cellBody := `{"kind":"code","source":"test"}`
	r := httptest.NewRequest("POST", "/api/notebooks/nonexistent/cells", strings.NewReader(cellBody))
	r.Header.Set("Content-Type", "application/json")
	r.SetPathValue("id", "nonexistent")
	w := httptest.NewRecorder()

	h.AddCell(w, r)

	resp := w.Result()
	if resp.StatusCode != http.StatusNotFound {
		t.Fatalf("expected 404, got %d", resp.StatusCode)
	}
}

func TestListNotebooks(t *testing.T) {
	h := newTestHandler()

	h.store.Create("first")
	h.store.Create("second")

	r := httptest.NewRequest("GET", "/api/notebooks", nil)
	w := httptest.NewRecorder()
	h.ListNotebooks(w, r)

	resp := w.Result()
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("expected 200, got %d", resp.StatusCode)
	}

	var nbs []*notebook.Notebook
	json.NewDecoder(resp.Body).Decode(&nbs)
	if len(nbs) != 2 {
		t.Fatalf("expected 2 notebooks, got %d", len(nbs))
	}
}

func TestListNotebooksEmpty(t *testing.T) {
	h := newTestHandler()
	r := httptest.NewRequest("GET", "/api/notebooks", nil)
	w := httptest.NewRecorder()
	h.ListNotebooks(w, r)

	resp := w.Result()
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("expected 200, got %d", resp.StatusCode)
	}

	var nbs []*notebook.Notebook
	json.NewDecoder(resp.Body).Decode(&nbs)
	if len(nbs) != 0 {
		t.Fatalf("expected 0 notebooks, got %d", len(nbs))
	}
}

func TestGetCell(t *testing.T) {
	h := newTestHandler()

	nb := h.store.Create("cell test")
	cell, _ := h.store.AddCell(nb.ID, notebook.CellKindCode, "print(42)")

	r := httptest.NewRequest("GET", "/api/notebooks/"+nb.ID+"/cells/"+cell.ID, nil)
	r.SetPathValue("id", nb.ID)
	r.SetPathValue("cellId", cell.ID)
	w := httptest.NewRecorder()
	h.GetCell(w, r)

	resp := w.Result()
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("expected 200, got %d", resp.StatusCode)
	}

	var got notebook.Cell
	json.NewDecoder(resp.Body).Decode(&got)
	if got.ID != cell.ID {
		t.Fatalf("expected cell ID %q, got %q", cell.ID, got.ID)
	}
}

func TestGetCellNotFound(t *testing.T) {
	h := newTestHandler()
	nb := h.store.Create("cell test")

	r := httptest.NewRequest("GET", "/api/notebooks/"+nb.ID+"/cells/nonexistent", nil)
	r.SetPathValue("id", nb.ID)
	r.SetPathValue("cellId", "nonexistent")
	w := httptest.NewRecorder()
	h.GetCell(w, r)

	resp := w.Result()
	if resp.StatusCode != http.StatusNotFound {
		t.Fatalf("expected 404, got %d", resp.StatusCode)
	}
}

func TestGetCellNotebookNotFound(t *testing.T) {
	h := newTestHandler()
	r := httptest.NewRequest("GET", "/api/notebooks/nonexistent/cells/cell_1", nil)
	r.SetPathValue("id", "nonexistent")
	r.SetPathValue("cellId", "cell_1")
	w := httptest.NewRecorder()
	h.GetCell(w, r)

	resp := w.Result()
	if resp.StatusCode != http.StatusNotFound {
		t.Fatalf("expected 404, got %d", resp.StatusCode)
	}
}
