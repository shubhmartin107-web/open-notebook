package kernel

import (
	"bufio"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"log"
	"os/exec"
	"sync"
	"time"

	"github.com/open-notebook/server/internal/notebook"
)

type CellResult struct {
	CellID   string `json:"cell_id"`
	Stdout   string `json:"stdout"`
	Stderr   string `json:"stderr"`
	ExitCode int    `json:"exit_code"`
}

type ExecuteResponse struct {
	Results []CellResult `json:"results"`
}

type Client struct {
	mu         sync.Mutex
	kernelPath string
	cmd        *exec.Cmd
	stdin      io.WriteCloser
	stdout     io.ReadCloser
	enc        *json.Encoder
	dec        *json.Decoder
	timeout    time.Duration
	logger     *log.Logger
}

func New(kernelPath string, timeout time.Duration) *Client {
	if kernelPath == "" {
		kernelPath = "onb-kernel"
	}
	if timeout == 0 {
		timeout = 30 * time.Second
	}
	return &Client{
		kernelPath: kernelPath,
		timeout:    timeout,
		logger:     log.Default(),
	}
}

func (c *Client) Start() error {
	c.mu.Lock()
	defer c.mu.Unlock()

	if c.cmd != nil {
		return errors.New("kernel already running")
	}

	cmd := exec.Command(c.kernelPath, "serve")
	stdin, err := cmd.StdinPipe()
	if err != nil {
		return fmt.Errorf("stdin pipe: %w", err)
	}
	stdout, err := cmd.StdoutPipe()
	if err != nil {
		return fmt.Errorf("stdout pipe: %w", err)
	}
	cmd.Stderr = nil

	if err := cmd.Start(); err != nil {
		return fmt.Errorf("start kernel: %w", err)
	}

	c.cmd = cmd
	c.stdin = stdin
	c.stdout = stdout
	c.enc = json.NewEncoder(stdin)
	c.dec = json.NewDecoder(bufio.NewReader(stdout))

	c.logger.Printf("kernel process started (pid %d)", cmd.Process.Pid)
	return nil
}

func (c *Client) Stop() error {
	c.mu.Lock()
	defer c.mu.Unlock()

	if c.cmd == nil {
		return nil
	}
	if c.stdin != nil {
		c.stdin.Close()
	}
	if c.cmd != nil && c.cmd.Process != nil {
		c.cmd.Process.Kill()
	}
	err := c.cmd.Wait()
	c.cmd = nil
	c.stdin = nil
	c.stdout = nil
	c.enc = nil
	c.dec = nil
	c.logger.Printf("kernel process stopped")
	return err
}

func (c *Client) Restart() error {
	c.Stop()
	return c.Start()
}

func (c *Client) sendCommand(cmdName string, fields map[string]interface{}) (map[string]interface{}, error) {
	c.mu.Lock()
	defer c.mu.Unlock()

	if c.cmd == nil {
		return nil, errors.New("kernel not running")
	}

	cmd := map[string]interface{}{"cmd": cmdName}
	for k, v := range fields {
		cmd[k] = v
	}

	if err := c.enc.Encode(cmd); err != nil {
		return nil, fmt.Errorf("encode command: %w", err)
	}

	type result struct {
		resp map[string]interface{}
		err  error
	}

	ch := make(chan result, 1)
	go func() {
		var raw json.RawMessage
		if err := c.dec.Decode(&raw); err != nil {
			ch <- result{nil, fmt.Errorf("decode response: %w", err)}
			return
		}
		var resp map[string]interface{}
		if err := json.Unmarshal(raw, &resp); err != nil {
			ch <- result{nil, fmt.Errorf("unmarshal response: %w", err)}
			return
		}
		ch <- result{resp, nil}
	}()

	select {
	case r := <-ch:
		if r.err != nil {
			return nil, r.err
		}
		if ok, _ := r.resp["ok"].(bool); !ok {
			errMsg := "kernel error"
			if e, ok := r.resp["error"].(string); ok && e != "" {
				errMsg = e
			}
			return nil, fmt.Errorf("kernel error: %s", errMsg)
		}
		return r.resp, nil
	case <-time.After(c.timeout):
		return nil, fmt.Errorf("kernel command %q timed out after %v", cmdName, c.timeout)
	}
}

func (c *Client) Ping() error {
	resp, err := c.sendCommand("ping", nil)
	if err != nil {
		return err
	}
	c.logger.Printf("kernel ping OK (version: %v)", resp["version"])
	return nil
}

func (c *Client) Execute(nb *notebook.Notebook, cellIDs []string) (*ExecuteResponse, error) {
	resp, err := c.sendCommand("execute", map[string]interface{}{
		"notebook": nb,
		"cell_ids": cellIDs,
	})
	if err != nil {
		return nil, err
	}

	result := &ExecuteResponse{}
	if outputs, ok := resp["outputs"].(map[string]interface{}); ok {
		for cellID, out := range outputs {
			outMap, _ := out.(map[string]interface{})
			cr := CellResult{CellID: cellID}
			if items, ok := outMap["items"].([]interface{}); ok {
				for _, item := range items {
					itemMap, _ := item.(map[string]interface{})
					if text, ok := itemMap["text"].(string); ok {
						cr.Stdout += text + "\n"
					}
				}
			}
			if errStr, ok := outMap["error"].(string); ok {
				cr.Stderr = errStr
			}
			result.Results = append(result.Results, cr)
		}
	}
	return result, nil
}

func (c *Client) Dag(nb *notebook.Notebook) (*notebook.DAG, error) {
	resp, err := c.sendCommand("dag", map[string]interface{}{
		"notebook": nb,
	})
	if err != nil {
		return nil, err
	}

	dag := &notebook.DAG{}
	if edgesRaw, ok := resp["edges"].([]interface{}); ok {
		for _, e := range edgesRaw {
			edgeMap, _ := e.(map[string]interface{})
			from, _ := edgeMap["from_cell_id"].(string)
			to, _ := edgeMap["to_cell_id"].(string)
			dag.Edges = append(dag.Edges, notebook.DAGEdge{From: from, To: to})
		}
	}
	return dag, nil
}

func (c *Client) ExportMd(nb *notebook.Notebook) (string, error) {
	resp, err := c.sendCommand("export_md", map[string]interface{}{
		"notebook": nb,
	})
	if err != nil {
		return "", err
	}
	md, _ := resp["markdown"].(string)
	return md, nil
}

func (c *Client) Load(path string) (*notebook.Notebook, error) {
	resp, err := c.sendCommand("load", map[string]interface{}{
		"path": path,
	})
	if err != nil {
		return nil, err
	}

	nbData, ok := resp["notebook"].(map[string]interface{})
	if !ok {
		return nil, errors.New("no notebook in response")
	}
	data, err := json.Marshal(nbData)
	if err != nil {
		return nil, fmt.Errorf("marshal notebook: %w", err)
	}
	var nb notebook.Notebook
	if err := json.Unmarshal(data, &nb); err != nil {
		return nil, fmt.Errorf("unmarshal notebook: %w", err)
	}
	return &nb, nil
}

func (c *Client) Save(path string, nb *notebook.Notebook) error {
	_, err := c.sendCommand("save", map[string]interface{}{
		"path":     path,
		"notebook": nb,
	})
	return err
}

func (c *Client) AiGenerate(nb *notebook.Notebook, cellIndex int, prompt string) (string, error) {
	resp, err := c.sendCommand("ai_generate", map[string]interface{}{
		"notebook":   nb,
		"cell_index": cellIndex,
		"prompt":     prompt,
	})
	if err != nil {
		return "", err
	}
	code, _ := resp["generated_code"].(string)
	if code == "" {
		return "", errors.New("empty response from kernel")
	}
	return code, nil
}
