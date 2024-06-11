package main

import (
	"net/http"

	"github.com/labstack/echo/v4"
)

type LexiconHandler func(c echo.Context) error
type RegisterLexiconFunc func(nsid string, handler LexiconHandler)
type RegisterFunc func(register RegisterLexiconFunc)

var LexiconHandlers = map[string]LexiconHandler{}

func RegisterLexicon(nsid string, handler LexiconHandler) {
	LexiconHandlers[nsid] = handler
}

func HandleXRPC(c echo.Context) error {
	NSID := c.Param("nsid")
	if handler, ok := LexiconHandlers[NSID]; ok {
		return handler(c)
	} else {
		return c.JSON(http.StatusBadRequest, map[string]string{
			"error":   "invalid nsid",
			"message": "invalid nsid",
		})
	}
}
