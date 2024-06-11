package database

import (
	"os"

	"github.com/surrealdb/surrealdb.go"
)

func Connect() *surrealdb.DB {
	db, err := surrealdb.New("ws://localhost:8000/rpc")
	if err != nil {
		panic(err)
	}

	if _, err = db.Signin(map[string]interface{}{
		"user": os.Getenv("DB_USER"),
		"pass": os.Getenv("DB_PASS"),
	}); err != nil {
		panic(err)
	}

	if _, err = db.Use(os.Getenv("DB_NAMESPACE"), os.Getenv("DB_DATABASE")); err != nil {
		panic(err)
	}

	return db
}
