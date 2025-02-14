APP_NAME=sekrets
INSTALL_DIR=/usr/local/bin
DATA_DIR=$(HOME)/.local/share/sekrets
CONFIG_DIR=$(HOME)/.config/sekrets
SNAP_BASE_DIR=$(HOME)/snap/code/
SNAP_DATA_DIR=$(shell find $(SNAP_BASE_DIR) -type d -name "sekrets" 2>/dev/null)
BIN_PATH=$(INSTALL_DIR)/$(APP_NAME)

# Default target
all: build

# Compile the Rust application
build:
	cargo build --release

# Install the binary to /usr/local/bin
install: build
	sudo install -m 755 target/release/$(APP_NAME) $(BIN_PATH)
	@echo "✅ Installed $(APP_NAME) to $(BIN_PATH)"
	@if [ -d "$(DATA_DIR)" ]; then \
		echo "✅ Existing data directory found: $(DATA_DIR)"; \
	else \
		mkdir -p $(DATA_DIR); \
		echo "✅ Created new data directory: $(DATA_DIR)"; \
	fi
	@if [ -d "$(CONFIG_DIR)" ]; then \
		echo "✅ Existing config directory found: $(CONFIG_DIR)"; \
	else \
		mkdir -p $(CONFIG_DIR); \
		echo "✅ Created new config directory: $(CONFIG_DIR)"; \
	fi

# Uninstall the application
uninstall:
	@if [ -f "$(BIN_PATH)" ]; then \
		echo "Removing $(APP_NAME) from $(INSTALL_DIR)..."; \
		sudo rm -f $(BIN_PATH); \
		echo "✅ Uninstalled $(APP_NAME)"; \
	else \
		echo "❌ $(APP_NAME) is not installed"; \
	fi
	@read -p "Do you want to remove saved data in $(DATA_DIR) and $(CONFIG_DIR)? [y/N]: " confirm; \
	if [ "$$confirm" = "y" ] || [ "$$confirm" = "Y" ]; then \
		rm -rf $(DATA_DIR) $(CONFIG_DIR); \
		echo "✅ Removed saved data"; \
	else \
		echo "✅ Saved data kept for future use"; \
	fi
	@if [ ! -z "$(SNAP_DATA_DIR)" ]; then \
		read -p "Do you want to remove Snap sandboxed data at $(SNAP_DATA_DIR)? [y/N]: " confirm_snap; \
		if [ "$$confirm_snap" = "y" ] || [ "$$confirm_snap" = "Y" ]; then \
			rm -rf $(SNAP_DATA_DIR); \
			echo "✅ Removed Snap sandboxed data"; \
		else \
			echo "✅ Snap data kept for future use"; \
		fi \
	else \
		echo "No Snap sandboxed data found."; \
	fi

# Clean up build files
clean:
	cargo clean
	rm -rf $(DATA_DIR) $(CONFIG_DIR) $(SNAP_DATA_DIR)
	@echo "🧹 Cleaned up build and data directories"

gen-cov:
	 cargo tarpaulin --tests --all-targets --out html --output-dir ./coverage


install-deb:
	cargo clean
	cargo deb
	sudo dpkg -i $(shell ls -t target/debian/sekrets_*.deb | head -n 1)


clippy:
	cargo clippy --all --all-targets --all-features -- -D warnings

fmt_check:
	cargo fmt --all -- --check

lint: clippy fmt_check

.PHONY: all build install uninstall clean gen-cov install-deb clippy fmt_check
