RISC_V_GCC := $(shell which riscv64-linux-gnu-gcc 2>/dev/null || echo "")
ifeq ($(RISC_V_GCC),)
	RISC_V_GCC := $(shell which /root/riscv64-toolchain/riscv64-linux-gnu-gcc 2>/dev/null || echo "")
endif
ifeq ($(RISC_V_GCC),)
	RISC_V_GCC := $(shell which /opt/riscv64/bin/riscv64/riscv64-linux-gnu-gcc 2>/dev/null || echo "")
endif

USER_DIR := user
TARGET_USER_DIR := target/user_prog
PROGS_RS := src/progs/progs.rs
LOG_LEVEL ?= error

USER_PROGRAMS := $(wildcard $(USER_DIR)/*.c)
USER_PROGRAM_NAMES := $(notdir $(USER_PROGRAMS))
USER_PROGRAM_BASES := $(USER_PROGRAM_NAMES:.c=)

build_user_programs: | $(TARGET_USER_DIR)
ifneq ($(RISC_V_GCC),)
	@echo "Using RISC-V GCC: $(RISC_V_GCC)"
	@for file in $(USER_PROGRAM_NAMES); do \
		base_name=$${file%.c}; \
		echo "Compiling $$file -> $$base_name"; \
		$(RISC_V_GCC) -static -O2  $(USER_DIR)/$$file -o $(TARGET_USER_DIR)/$$base_name || exit 1; \
	done
else
	@echo "Warning: RISC-V GCC not found, skipping user program compilation"
endif

$(TARGET_USER_DIR):
	mkdir -p $(TARGET_USER_DIR)

generate_progs_rs: $(PROGS_RS)

blk_img:
	@dd if=/dev/zero of=blk.img bs=1M count=64
	# Write "Hello World from Image!" to the first sector
	@echo -n "Hello World from Image!" | dd of=blk.img bs=1 seek=0 conv=notrunc

$(PROGS_RS): $(USER_PROGRAMS) | $(TARGET_USER_DIR)
	@echo "Generating $(PROGS_RS)"
	@rm -f $(PROGS_RS)
	@#
	@for name in $(USER_PROGRAM_BASES); do \
		echo "const $$(echo $$name | tr '[:lower:]' '[:upper:]'): &[u8] =" >> $(PROGS_RS); \
		echo "    include_bytes_aligned::include_bytes_aligned!(32, \"../../target/user_prog/$$name\");" >> $(PROGS_RS); \
	done
	@
	@echo "" >> $(PROGS_RS)
	@echo "pub fn init() {" >> $(PROGS_RS)
	@echo "    super::USER_PROGS.call_once(|| {" >> $(PROGS_RS)
	@echo "        let mut user_progs = alloc::collections::btree_map::BTreeMap::new();" >> $(PROGS_RS)
	@for name in $(USER_PROGRAM_BASES); do \
		echo "        user_progs.insert(\"$$name\", $$(echo $$name | tr '[:lower:]' '[:upper:]'));" >> $(PROGS_RS); \
	done
	@echo "        user_progs" >> $(PROGS_RS)
	@echo "    });" >> $(PROGS_RS)
	@echo "}" >> $(PROGS_RS)

clean:
	rm -f $(PROGS_RS)
	cargo clean
	rm -f blk.img

run: build_user_programs generate_progs_rs blk_img
	cargo osdk run --target-arch=riscv64 --kcmd-args="ostd.log_level=$(LOG_LEVEL)" --release

debug: build_user_programs generate_progs_rs blk_img
	cargo osdk run --target-arch=riscv64 --kcmd-args="ostd.log_level=$(LOG_LEVEL)"

build: build_user_programs generate_progs_rs blk_img
	cargo osdk build --target-arch=riscv64 --release

test: build_user_programs generate_progs_rs blk_img
	cargo osdk test --target-arch=riscv64 --release

.PHONY: build_user_programs generate_progs_rs clean run
