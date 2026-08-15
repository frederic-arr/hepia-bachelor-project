default: fmt install

build:
	go build -v ./...

install: build
	go install -v ./...

fmt:
	gofmt -s -w -e .

.PHONY: fmt build install
