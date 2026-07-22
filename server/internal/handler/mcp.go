package handler

import (
	"encoding/json"
	"net/http"
)

type ToolDefinition struct {
	Name        string                 `json:"name"`
	Description string                 `json:"description"`
	InputSchema map[string]interface{} `json:"input_schema"`
}

type ExecuteToolRequest struct {
	Name string                 `json:"name"`
	Args map[string]interface{} `json:"args"`
}

func (h *Handler) ListTools(w http.ResponseWriter, r *http.Request) {
	tools := []ToolDefinition{
		{
			Name: "execute_sql", Description: "Execute a SQL query against the notebook database",
			InputSchema: map[string]interface{}{
				"type": "object", "properties": map[string]interface{}{
					"query": map[string]interface{}{"type": "string", "description": "SQL query"},
				}, "required": []string{"query"},
			},
		},
		{
			Name: "execute_python", Description: "Execute Python code in a new cell",
			InputSchema: map[string]interface{}{
				"type": "object", "properties": map[string]interface{}{
					"code": map[string]interface{}{"type": "string", "description": "Python code"},
				}, "required": []string{"code"},
			},
		},
		{
			Name: "get_cell_content", Description: "Get the source content of a specific cell",
			InputSchema: map[string]interface{}{
				"type": "object", "properties": map[string]interface{}{
					"cell_id": map[string]interface{}{"type": "string", "description": "Cell ID"},
				}, "required": []string{"cell_id"},
			},
		},
		{
			Name: "update_cell", Description: "Update the source of a specific cell",
			InputSchema: map[string]interface{}{
				"type": "object", "properties": map[string]interface{}{
					"cell_id": map[string]interface{}{"type": "string"},
					"source":  map[string]interface{}{"type": "string"},
				}, "required": []string{"cell_id", "source"},
			},
		},
		{
			Name: "insert_cell", Description: "Insert a new cell",
			InputSchema: map[string]interface{}{
				"type": "object", "properties": map[string]interface{}{
					"kind":   map[string]interface{}{"type": "string", "enum": []string{"python", "sql", "markdown", "raw"}},
					"source": map[string]interface{}{"type": "string"},
				}, "required": []string{"kind"},
			},
		},
	}
	writeJSON(w, http.StatusOK, tools)
}

func (h *Handler) ExecuteTool(w http.ResponseWriter, r *http.Request) {
	var req ExecuteToolRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "invalid request body", http.StatusBadRequest)
		return
	}

	result := map[string]interface{}{
		"tool":    req.Name,
		"status":  "executed",
		"message": "Tool executed",
	}

	writeJSON(w, http.StatusOK, result)
}
