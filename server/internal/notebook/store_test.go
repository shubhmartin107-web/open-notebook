package notebook

import (
	"testing"
)

func TestNewStore(t *testing.T) {
	s := NewStore()
	if s == nil {
		t.Fatal("NewStore returned nil")
	}
}

func TestCreateAndGet(t *testing.T) {
	s := NewStore()
	nb := s.Create("test notebook")
	if nb.ID == "" {
		t.Fatal("expected non-empty ID")
	}
	if nb.Title != "test notebook" {
		t.Fatalf("expected title 'test notebook', got %q", nb.Title)
	}

	got, ok := s.Get(nb.ID)
	if !ok {
		t.Fatal("expected notebook to exist")
	}
	if got.ID != nb.ID {
		t.Fatalf("expected ID %q, got %q", nb.ID, got.ID)
	}
}

func TestGetMissing(t *testing.T) {
	s := NewStore()
	_, ok := s.Get("nonexistent")
	if ok {
		t.Fatal("expected false for missing notebook")
	}
}

func TestUpdate(t *testing.T) {
	s := NewStore()
	nb := s.Create("original")
	updated, ok := s.Update(nb.ID, "updated title")
	if !ok {
		t.Fatal("expected update to succeed")
	}
	if updated.Title != "updated title" {
		t.Fatalf("expected 'updated title', got %q", updated.Title)
	}

	got, _ := s.Get(nb.ID)
	if got.Title != "updated title" {
		t.Fatalf("expected updated title in store")
	}
}

func TestUpdateMissing(t *testing.T) {
	s := NewStore()
	_, ok := s.Update("nonexistent", "title")
	if ok {
		t.Fatal("expected false for missing notebook")
	}
}

func TestDelete(t *testing.T) {
	s := NewStore()
	nb := s.Create("to delete")
	if !s.Delete(nb.ID) {
		t.Fatal("expected delete to return true")
	}
	if s.Delete(nb.ID) {
		t.Fatal("expected second delete to return false")
	}
	_, ok := s.Get(nb.ID)
	if ok {
		t.Fatal("expected notebook to be deleted")
	}
}

func TestDeleteMissing(t *testing.T) {
	s := NewStore()
	if s.Delete("nonexistent") {
		t.Fatal("expected false for missing notebook")
	}
}

func TestAddCell(t *testing.T) {
	s := NewStore()
	nb := s.Create("cells")
	cell, ok := s.AddCell(nb.ID, CellKindMarkdown, "# Hello")
	if !ok {
		t.Fatal("expected AddCell to succeed")
	}
	if cell.Kind != CellKindMarkdown {
		t.Fatalf("expected markdown kind, got %q", cell.Kind)
	}
	if cell.Source != "# Hello" {
		t.Fatalf("expected source '# Hello', got %q", cell.Source)
	}
	if cell.ID == "" {
		t.Fatal("expected non-empty cell ID")
	}

	// Verify cell is in notebook
	if len(nb.Cells) != 1 {
		t.Fatalf("expected 1 cell, got %d", len(nb.Cells))
	}
}

func TestAddCellMissingNotebook(t *testing.T) {
	s := NewStore()
	_, ok := s.AddCell("nonexistent", CellKindMarkdown, "source")
	if ok {
		t.Fatal("expected false for missing notebook")
	}
}

func TestUpdateCell(t *testing.T) {
	s := NewStore()
	nb := s.Create("test")
	s.AddCell(nb.ID, CellKindCode, "old source")

	cellID := nb.Cells[0].ID
	updated, ok := s.UpdateCell(nb.ID, cellID, "new source")
	if !ok {
		t.Fatal("expected UpdateCell to succeed")
	}
	if updated.Source != "new source" {
		t.Fatalf("expected 'new source', got %q", updated.Source)
	}
}

func TestUpdateCellMissing(t *testing.T) {
	s := NewStore()
	nb := s.Create("test")
	_, ok := s.UpdateCell(nb.ID, "nonexistent", "source")
	if ok {
		t.Fatal("expected false for missing cell")
	}
}

func TestRemoveCell(t *testing.T) {
	s := NewStore()
	nb := s.Create("test")
	s.AddCell(nb.ID, CellKindCode, "cell 1")
	s.AddCell(nb.ID, CellKindMarkdown, "cell 2")

	if !s.RemoveCell(nb.ID, nb.Cells[0].ID) {
		t.Fatal("expected RemoveCell to succeed")
	}
	if len(nb.Cells) != 1 {
		t.Fatalf("expected 1 cell remaining, got %d", len(nb.Cells))
	}
	if nb.Cells[0].Source != "cell 2" {
		t.Fatalf("expected remaining cell to have source 'cell 2'")
	}
}

func TestRemoveCellMissingNotebook(t *testing.T) {
	s := NewStore()
	if s.RemoveCell("nonexistent", "cell_1") {
		t.Fatal("expected false for missing notebook")
	}
}

func TestSetCellOutput(t *testing.T) {
	s := NewStore()
	nb := s.Create("test")
	s.AddCell(nb.ID, CellKindCode, "source")
	cellID := nb.Cells[0].ID

	output := &CellOutput{Stdout: "hello", Stderr: "", ExitCode: 0}
	if !s.SetCellOutput(nb.ID, cellID, output) {
		t.Fatal("expected SetCellOutput to succeed")
	}
	if nb.Cells[0].Output == nil {
		t.Fatal("expected output to be set")
	}
	if nb.Cells[0].Output.Stdout != "hello" {
		t.Fatalf("expected stdout 'hello', got %q", nb.Cells[0].Output.Stdout)
	}
}

func TestSetCellOutputMissing(t *testing.T) {
	s := NewStore()
	if s.SetCellOutput("nonexistent", "cell_1", &CellOutput{}) {
		t.Fatal("expected false for missing notebook")
	}
}

func TestConcurrentAccess(t *testing.T) {
	s := NewStore()
	done := make(chan bool, 10)

	for i := 0; i < 10; i++ {
		go func() {
			nb := s.Create("concurrent")
			s.AddCell(nb.ID, CellKindCode, "source")
			s.Get(nb.ID)
			s.Update(nb.ID, "new title")
			s.Delete(nb.ID)
			done <- true
		}()
	}

	for i := 0; i < 10; i++ {
		<-done
	}
}
