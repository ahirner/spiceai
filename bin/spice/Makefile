BASE_PACKAGE_NAME := github.com/spiceai/spiceai

ifdef REL_VERSION
	SPICE_VERSION := $(REL_VERSION)
	SPICED_FEATURES := --features release
else
	SPICE_VERSION := local
endif

ifdef DEV
	SPICE_VERSION := local-dev
endif

LDFLAGS:="-X $(BASE_PACKAGE_NAME)/bin/spice/pkg/version.version=$(SPICE_VERSION)"

.PHONY: all
all:
ifeq ($(OS),Windows_NT)
	go build -v -ldflags=$(LDFLAGS) -o ..\..\target\release\spice.exe
else
	mkdir -p ../../target/release 2> /dev/null || true
	go build -v -ldflags=$(LDFLAGS) -o ../../target/release/spice
endif