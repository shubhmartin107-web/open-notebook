package main

import (
	"context"
	"flag"
	"fmt"
	"log"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/open-notebook/server/internal/handler"
	"github.com/open-notebook/server/internal/kernel"
	"github.com/open-notebook/server/internal/notebook"
)

func main() {
	port := flag.Int("port", 8080, "HTTP server port")
	kernelPath := flag.String("kernel", "", "path to onb-kernel binary")
	flag.Parse()

	if *kernelPath == "" {
		*kernelPath = os.Getenv("KERNEL_PATH")
	}

	store := notebook.NewStore()
	kc := kernel.New(*kernelPath, 30*time.Second)

	if err := kc.Start(); err != nil {
		log.Fatalf("failed to start kernel: %v", err)
	}

	if err := kc.Ping(); err != nil {
		log.Fatalf("kernel not responding after start: %v", err)
	}
	log.Println("kernel process started and healthy")

	h := handler.NewHandler(store, kc)
	mux := http.NewServeMux()

	mux.HandleFunc("GET /api/health", handler.Health)

	mux.HandleFunc("GET /api/notebooks", h.ListNotebooks)
	mux.HandleFunc("POST /api/notebooks", h.CreateNotebook)
	mux.HandleFunc("GET /api/notebooks/{id}", h.GetNotebook)
	mux.HandleFunc("PUT /api/notebooks/{id}", h.UpdateNotebook)
	mux.HandleFunc("DELETE /api/notebooks/{id}", h.DeleteNotebook)

	mux.HandleFunc("POST /api/notebooks/{id}/cells", h.AddCell)
	mux.HandleFunc("PUT /api/notebooks/{id}/cells/{cellId}", h.UpdateCell)
	mux.HandleFunc("DELETE /api/notebooks/{id}/cells/{cellId}", h.RemoveCell)
	mux.HandleFunc("GET /api/notebooks/{id}/cells/{cellId}", h.GetCell)

	mux.HandleFunc("POST /api/notebooks/{id}/execute", h.ExecuteNotebook)
	mux.HandleFunc("POST /api/notebooks/{id}/cells/{cellId}/execute", h.ExecuteCell)

	mux.HandleFunc("GET /api/notebooks/{id}/dag", h.GetDAG)
	mux.HandleFunc("POST /api/notebooks/{id}/save", h.SaveNotebook)

	mux.HandleFunc("POST /api/notebooks/{id}/ai-generate", h.AiGenerate)
	mux.HandleFunc("GET /api/notebooks/{id}/sync", h.SyncNotebook)

	mux.HandleFunc("POST /api/notebooks/{id}/execute/stream", h.ExecuteNotebookStream)
	mux.HandleFunc("POST /api/notebooks/{id}/ai/generate/stream", h.AiGenerateStream)
	mux.HandleFunc("GET /api/notebooks/{id}/ai/status", h.AiStatus)
	mux.HandleFunc("POST /api/kernel/start", h.StartKernel)
	mux.HandleFunc("POST /api/kernel/stop", h.StopKernel)
	mux.HandleFunc("GET /api/kernel/status", h.GetKernelStatus)
	mux.HandleFunc("GET /api/mcp/tools", h.ListTools)
	mux.HandleFunc("POST /api/mcp/execute", h.ExecuteTool)
	mux.HandleFunc("GET /api/notebooks/{id}/events", h.NotebookEvents)

	srv := &http.Server{
		Addr:    fmt.Sprintf(":%d", *port),
		Handler: withLogging(withCORS(mux)),
	}

	quit := make(chan os.Signal, 1)
	signal.Notify(quit, syscall.SIGINT, syscall.SIGTERM)

	go func() {
		log.Printf("server starting on port %d", *port)
		if err := srv.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			log.Fatalf("server failed: %v", err)
		}
	}()

	<-quit
	log.Println("shutting down...")

	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	if err := srv.Shutdown(ctx); err != nil {
		log.Fatalf("shutdown error: %v", err)
	}

	if err := kc.Stop(); err != nil {
		log.Printf("kernel stop error: %v", err)
	}
}

func withLogging(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		log.Printf("%s %s", r.Method, r.URL.Path)
		next.ServeHTTP(w, r)
	})
}

func withCORS(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Access-Control-Allow-Origin", "*")
		w.Header().Set("Access-Control-Allow-Methods", "GET, POST, PUT, PATCH, DELETE, OPTIONS")
		w.Header().Set("Access-Control-Allow-Headers", "Content-Type, Authorization")
		if r.Method == "OPTIONS" {
			w.WriteHeader(http.StatusNoContent)
			return
		}
		next.ServeHTTP(w, r)
	})
}
