# 定义变量
dtc = dtc
arch = qemu-virt-arm

# 设备树编译规则
%.dtb: %.dts
	@echo Generating device tree file $@
	@$(dtc) -I dts -O dtb -o $@ $<
	# @rm -rf src/arch/${arch}/boot/dtb.gen.s

# 默认目标
all: qemu-virt-arm.dtb

# 清理规则
clean:
	@rm -f *.dtb
	# @rm -rf src/arch/${arch}/boot/dtb.gen.s
