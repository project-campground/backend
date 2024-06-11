package main

import (
	"log"

	"github.com/joho/godotenv"
	"github.com/labstack/echo/v4"
)

func main() {
	err := godotenv.Load()
	if err != nil {
		log.Fatal("Error loading .env file")
	}

	e := echo.New()

	e.GET("/xrpc/:nsid", HandleXRPC)
	e.POST("/xrpc/:nsid", HandleXRPC)
	RegisterLexicons()

	log.Fatal(e.Start(":1313"))
}
