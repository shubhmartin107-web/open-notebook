package notebook

import (
	"fmt"
	"sync"
)

type CellKind string

const (
	CellKindMarkdown CellKind = "markdown"
	CellKindCode     CellKind = "code"
	CellKindPython   CellKind = "python"
	CellKindSQL      CellKind = "sql"
	CellKindR        CellKind = "r"
	CellKindRaw      CellKind = "raw"
)

type CellOutput struct {
	Stdout   string `json:"stdout,omitempty"`
	Stderr   string `json:"stderr,omitempty"`
	ExitCode int    `json:"exit_code"`
}

type Cell struct {
	ID     string      `json:"id"`
	Kind   CellKind    `json:"kind"`
	Source string      `json:"source"`
	Output *CellOutput `json:"output,omitempty"`
}

type Notebook struct {
	ID    string  `json:"id"`
	Title string  `json:"title"`
	Cells []*Cell `json:"cells"`
}

type DAGNode struct {
	ID     string   `json:"id"`
	Label  string   `json:"label"`
	Kind   CellKind `json:"kind"`
	Status string   `json:"status"`
}

type DAGEdge struct {
	From string `json:"from"`
	To   string `json:"to"`
}

type DAG struct {
	Nodes []DAGNode `json:"nodes"`
	Edges []DAGEdge `json:"edges"`
}

type Store struct {
	mu        sync.RWMutex
	notebooks map[string]*Notebook
	counter   int64
}

func NewStore() *Store {
	return &Store{
		notebooks: make(map[string]*Notebook),
	}
}

func (s *Store) Create(title string) *Notebook {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.counter++
	nb := &Notebook{
		ID:    fmt.Sprintf("nb_%d", s.counter),
		Title: title,
		Cells: make([]*Cell, 0),
	}
	s.notebooks[nb.ID] = nb
	return nb
}

func (s *Store) Get(id string) (*Notebook, bool) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	nb, ok := s.notebooks[id]
	return nb, ok
}

func (s *Store) List() []*Notebook {
	s.mu.RLock()
	defer s.mu.RUnlock()
	result := make([]*Notebook, 0, len(s.notebooks))
	for _, nb := range s.notebooks {
		result = append(result, nb)
	}
	return result
}

func (s *Store) Update(id, title string) (*Notebook, bool) {
	s.mu.Lock()
	defer s.mu.Unlock()
	nb, ok := s.notebooks[id]
	if !ok {
		return nil, false
	}
	nb.Title = title
	return nb, true
}

func (s *Store) Delete(id string) bool {
	s.mu.Lock()
	defer s.mu.Unlock()
	_, ok := s.notebooks[id]
	if !ok {
		return false
	}
	delete(s.notebooks, id)
	return true
}

func (s *Store) AddCell(notebookID string, kind CellKind, source string) (*Cell, bool) {
	s.mu.Lock()
	defer s.mu.Unlock()
	nb, ok := s.notebooks[notebookID]
	if !ok {
		return nil, false
	}
	s.counter++
	cell := &Cell{
		ID:     fmt.Sprintf("cell_%d", s.counter),
		Kind:   kind,
		Source: source,
	}
	nb.Cells = append(nb.Cells, cell)
	return cell, true
}

func (s *Store) UpdateCell(notebookID, cellID, source string) (*Cell, bool) {
	s.mu.Lock()
	defer s.mu.Unlock()
	nb, ok := s.notebooks[notebookID]
	if !ok {
		return nil, false
	}
	for _, c := range nb.Cells {
		if c.ID == cellID {
			c.Source = source
			return c, true
		}
	}
	return nil, false
}

func (s *Store) RemoveCell(notebookID, cellID string) bool {
	s.mu.Lock()
	defer s.mu.Unlock()
	nb, ok := s.notebooks[notebookID]
	if !ok {
		return false
	}
	for i, c := range nb.Cells {
		if c.ID == cellID {
			nb.Cells = append(nb.Cells[:i], nb.Cells[i+1:]...)
			return true
		}
	}
	return false
}

func (s *Store) SetCellOutput(notebookID, cellID string, output *CellOutput) bool {
	s.mu.Lock()
	defer s.mu.Unlock()
	nb, ok := s.notebooks[notebookID]
	if !ok {
		return false
	}
	for _, c := range nb.Cells {
		if c.ID == cellID {
			c.Output = output
			return true
		}
	}
	return false
}
