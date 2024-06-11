module registry

go 1.22.0

require github.com/project-campground/backend/libs/database v0.0.0

require (
	github.com/bluesky-social/indigo v0.0.0-20240604221852-9815da964ae1 // indirect
	github.com/joho/godotenv v1.5.1 // indirect
	github.com/labstack/echo/v4 v4.12.0 // indirect
	github.com/labstack/gommon v0.4.2 // indirect
	github.com/mattn/go-colorable v0.1.13 // indirect
	github.com/mattn/go-isatty v0.0.20 // indirect
	github.com/valyala/bytebufferpool v1.0.0 // indirect
	github.com/valyala/fasttemplate v1.2.2 // indirect
	golang.org/x/crypto v0.22.0 // indirect
	golang.org/x/net v0.24.0 // indirect
	golang.org/x/sys v0.19.0 // indirect
	golang.org/x/text v0.14.0 // indirect
)

replace github.com/project-campground/backend/libs/database v0.0.0 => ../../libs/database
