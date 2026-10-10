CARGO ?= cargo
VERSION := $(shell sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n1)

.PHONY: help build run test fmt clippy size clean \
        linux-install linux-uninstall appimage \
        windows-installer mac-bundle mac-sign

help:
	@echo "Targets / Target disponibili:"
	@echo "  make build             Release build / Build release"
	@echo "  make run               Run from source / Avvia dal sorgente"
	@echo "  make test              Run tests / Esegui i test"
	@echo "  make appimage          Build Linux AppImage"
	@echo "  make linux-install     Install for current user / Installa per l'utente"
	@echo "  make windows-installer Build Windows installer (Inno Setup)"
	@echo "  make mac-bundle        Build macOS .app bundle"
	@echo "  make mac-sign          Sign + notarize macOS bundle"
	@echo "  make clean             Remove build artifacts"

build:
	$(CARGO) build --release

run:
	$(CARGO) run --release

test:
	$(CARGO) test

fmt:
	$(CARGO) fmt --all

clippy:
	$(CARGO) clippy --release -- -D warnings

size: build
	@ls -lh target/release/microscope-rust* 2>/dev/null || ls -lh target/release/microscope-rust

appimage:
	sh packaging/linux/make-appimage.sh

linux-install: build
	sh packaging/linux/install.sh

linux-uninstall:
	sh packaging/linux/uninstall.sh

windows-installer: build
	ISCC packaging/windows/installer.iss || \
		"$${ProgramFiles(x86)}/Inno Setup 6/ISCC.exe" packaging/windows/installer.iss

mac-bundle:
	sh packaging/macos/bundle.sh

mac-sign:
	sh packaging/macos/sign_and_notarize.sh

clean:
	$(CARGO) clean
	rm -rf dist
