package main

import (
	"github.com/labstack/echo/v4"
)

func RegisterLexicons() {
	// com.atproto.identity
	RegisterLexicon(
		"com.atproto.identity.ResolveHandle",
		func(c echo.Context) error {
			return c.String(200, "OK")
		},
	)
}
