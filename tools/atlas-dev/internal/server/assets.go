package server

import (
	"embed"
	"io/fs"
	"net/http"
)

//go:embed web/*
var embedFS embed.FS

// GetFileSystem returns the embedded filesystem for web assets
func GetFileSystem() (http.FileSystem, error) {
	webFS, err := fs.Sub(embedFS, "web")
	if err != nil {
		return nil, err
	}
	return http.FS(webFS), nil
}
