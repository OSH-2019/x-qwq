CC=gcc
CPP=cpp
CFLAGS=-std=c99 -Wall -O2 -mcmodel=kernel -nostdinc -nostdlib -mtune=generic -fno-pic -fno-pie -fno-stack-protector -fno-asynchronous-unwind-tables -m64 -mno-mmx -mno-sse -mno-sse2 -mno-3dnow -ffreestanding -Wa,--64 -D__KERNEL_64__ -DHAVE_AUTOCONF -DDEBUG -g -ggdb -DCONFIG_ARCH_X86_64 -m64 -march=nehalem
INCLUDE=-Iinclude -Iinclude/64 -Iinclude/arch/x86 -Iinclude/arch/x86/arch/64 -Iinclude/plat/pc99 -Iinclude/plat/pc99/plat/64 -Igen_config -Igenerated -Iautoconf -Igen_headers -Ilibsel4/include
OBJECT_DIR=object
RUSTC=rustc
ASM=as
ASMFLAGS=--64
LD=ld
LDFLAGS=-n -static
OBJECTS=$(OBJECT_DIR)/inlines.o $(OBJECT_DIR)/assert.o $(OBJECT_DIR)/util.o $(OBJECT_DIR)/string.o $(OBJECT_DIR)/fastpath/fastpath.o $(OBJECT_DIR)/api/syscall.o $(OBJECT_DIR)/api/faults.o $(OBJECT_DIR)/kernel/cspace.o $(OBJECT_DIR)/kernel/faulthandler.o $(OBJECT_DIR)/kernel/thread.o $(OBJECT_DIR)/kernel/boot.o $(OBJECT_DIR)/kernel/stack.o $(OBJECT_DIR)/object/notification.o $(OBJECT_DIR)/object/cnode.o $(OBJECT_DIR)/object/endpoint.o $(OBJECT_DIR)/object/interrupt.o $(OBJECT_DIR)/object/objecttype.o $(OBJECT_DIR)/object/tcb.o $(OBJECT_DIR)/object/untyped.o $(OBJECT_DIR)/model/preemption.o $(OBJECT_DIR)/model/statedata.o $(OBJECT_DIR)/model/smp.o $(OBJECT_DIR)/machine/io.o $(OBJECT_DIR)/machine/registerset.o $(OBJECT_DIR)/machine/fpu.o $(OBJECT_DIR)/benchmark/benchmark_track.o $(OBJECT_DIR)/benchmark/benchmark_utilisation.o $(OBJECT_DIR)/smp/lock.o $(OBJECT_DIR)/smp/ipi.o $(OBJECT_DIR)/arch/x86/c_traps.o $(OBJECT_DIR)/arch/x86/idle.o $(OBJECT_DIR)/arch/x86/api/faults.o $(OBJECT_DIR)/arch/x86/object/interrupt.o $(OBJECT_DIR)/arch/x86/object/ioport.o $(OBJECT_DIR)/arch/x86/object/objecttype.o $(OBJECT_DIR)/arch/x86/object/tcb.o $(OBJECT_DIR)/arch/x86/object/iospace.o $(OBJECT_DIR)/arch/x86/object/vcpu.o $(OBJECT_DIR)/arch/x86/kernel/vspace.o $(OBJECT_DIR)/arch/x86/kernel/apic.o $(OBJECT_DIR)/arch/x86/kernel/xapic.o $(OBJECT_DIR)/arch/x86/kernel/x2apic.o $(OBJECT_DIR)/arch/x86/kernel/boot_sys.o $(OBJECT_DIR)/arch/x86/kernel/smp_sys.o $(OBJECT_DIR)/arch/x86/kernel/boot.o $(OBJECT_DIR)/arch/x86/kernel/cmdline.o $(OBJECT_DIR)/arch/x86/kernel/ept.o $(OBJECT_DIR)/arch/x86/kernel/thread.o $(OBJECT_DIR)/arch/x86/model/statedata.o $(OBJECT_DIR)/arch/x86/machine/hardware.o $(OBJECT_DIR)/arch/x86/machine/fpu.o $(OBJECT_DIR)/arch/x86/machine/cpu_identification.o $(OBJECT_DIR)/arch/x86/machine/breakpoint.o $(OBJECT_DIR)/arch/x86/machine/registerset.o $(OBJECT_DIR)/arch/x86/machine/capdl.o $(OBJECT_DIR)/arch/x86/benchmark/benchmark.o $(OBJECT_DIR)/arch/x86/smp/ipi.o $(OBJECT_DIR)/arch/x86/64/c_traps.o $(OBJECT_DIR)/arch/x86/64/object/objecttype.o $(OBJECT_DIR)/arch/x86/64/kernel/thread.o $(OBJECT_DIR)/arch/x86/64/kernel/vspace.o $(OBJECT_DIR)/arch/x86/64/kernel/elf.o $(OBJECT_DIR)/arch/x86/64/model/statedata.o $(OBJECT_DIR)/arch/x86/64/model/smp.o $(OBJECT_DIR)/arch/x86/64/machine/registerset.o $(OBJECT_DIR)/arch/x86/64/smp/ipi.o $(OBJECT_DIR)/arch/x86/multiboot.o $(OBJECT_DIR)/arch/x86/64/machine_asm.o $(OBJECT_DIR)/arch/x86/64/traps.o $(OBJECT_DIR)/arch/x86/64/head.o $(OBJECT_DIR)/plat/pc99/machine/acpi.o $(OBJECT_DIR)/plat/pc99/machine/hardware.o $(OBJECT_DIR)/plat/pc99/machine/pic.o $(OBJECT_DIR)/plat/pc99/machine/ioapic.o $(OBJECT_DIR)/plat/pc99/machine/pit.o $(OBJECT_DIR)/plat/pc99/machine/io.o $(OBJECT_DIR)/plat/pc99/machine/intel-vtd.o $(OBJECT_DIR)/config/default_domain.o

ALL: DIR $(OBJECTS) kernel.elf kernel

DIR:
	mkdir $(OBJECT_DIR)
	mkdir $(OBJECT_DIR)/api
	mkdir $(OBJECT_DIR)/object
	mkdir $(OBJECT_DIR)/config
	mkdir $(OBJECT_DIR)/plat
	mkdir $(OBJECT_DIR)/plat/pc99
	mkdir $(OBJECT_DIR)/plat/pc99/machine
	mkdir $(OBJECT_DIR)/fastpath
	mkdir $(OBJECT_DIR)/kernel
	mkdir $(OBJECT_DIR)/arch
	mkdir $(OBJECT_DIR)/arch/x86
	mkdir $(OBJECT_DIR)/arch/x86/64
	mkdir $(OBJECT_DIR)/arch/x86/64/kernel
	mkdir $(OBJECT_DIR)/arch/x86/64/object
	mkdir $(OBJECT_DIR)/arch/x86/64/machine
	mkdir $(OBJECT_DIR)/arch/x86/64/model
	mkdir $(OBJECT_DIR)/arch/x86/64/smp
	mkdir $(OBJECT_DIR)/arch/x86/object
	mkdir $(OBJECT_DIR)/arch/x86/benchmark
	mkdir $(OBJECT_DIR)/arch/x86/kernel
	mkdir $(OBJECT_DIR)/arch/x86/machine
	mkdir $(OBJECT_DIR)/arch/x86/model
	mkdir $(OBJECT_DIR)/arch/x86/smp
	mkdir $(OBJECT_DIR)/arch/x86/api
	mkdir $(OBJECT_DIR)/benchmark
	mkdir $(OBJECT_DIR)/machine
	mkdir $(OBJECT_DIR)/model
	mkdir $(OBJECT_DIR)/smp

$(OBJECT_DIR)/inlines.o: src/inlines.c
	$(CC) $(CFLAGS) $(INCLUDE) src/inlines.c -c -o $(OBJECT_DIR)/inlines.o
$(OBJECT_DIR)/assert.o: src/assert.c
	$(CC) $(CFLAGS) $(INCLUDE) src/assert.c -c -o $(OBJECT_DIR)/assert.o
$(OBJECT_DIR)/util.o: src/util.c
	$(CC) $(CFLAGS) $(INCLUDE) src/util.c -c -o $(OBJECT_DIR)/util.o
$(OBJECT_DIR)/string.o: src/string.c
	$(CC) $(CFLAGS) $(INCLUDE) src/string.c -c -o $(OBJECT_DIR)/string.o
$(OBJECT_DIR)/fastpath/fastpath.o: src/fastpath/fastpath.c
	$(CC) $(CFLAGS) $(INCLUDE) src/fastpath/fastpath.c -c -o $(OBJECT_DIR)/fastpath/fastpath.o
$(OBJECT_DIR)/api/syscall.o: src/api/syscall.c
	$(CC) $(CFLAGS) $(INCLUDE) src/api/syscall.c -c -o $(OBJECT_DIR)/api/syscall.o
$(OBJECT_DIR)/api/faults.o: src/api/faults.c
	$(CC) $(CFLAGS) $(INCLUDE) src/api/faults.c -c -o $(OBJECT_DIR)/api/faults.o
$(OBJECT_DIR)/kernel/cspace.o: src/kernel/cspace.c
	$(CC) $(CFLAGS) $(INCLUDE) src/kernel/cspace.c -c -o $(OBJECT_DIR)/kernel/cspace.o
$(OBJECT_DIR)/kernel/faulthandler.o: src/kernel/faulthandler.c
	$(CC) $(CFLAGS) $(INCLUDE) src/kernel/faulthandler.c -c -o $(OBJECT_DIR)/kernel/faulthandler.o
$(OBJECT_DIR)/kernel/thread.o: src/kernel/thread.c
	$(CC) $(CFLAGS) $(INCLUDE) src/kernel/thread.c -c -o $(OBJECT_DIR)/kernel/thread.o
$(OBJECT_DIR)/kernel/boot.o: src/kernel/boot.c
	$(CC) $(CFLAGS) $(INCLUDE) src/kernel/boot.c -c -o $(OBJECT_DIR)/kernel/boot.o
$(OBJECT_DIR)/kernel/stack.o: src/kernel/stack.c
	$(CC) $(CFLAGS) $(INCLUDE) src/kernel/stack.c -c -o $(OBJECT_DIR)/kernel/stack.o
$(OBJECT_DIR)/object/notification.o: src/$(OBJECT_DIR)/notification.c
	$(CC) $(CFLAGS) $(INCLUDE) src/$(OBJECT_DIR)/notification.c -c -o $(OBJECT_DIR)/object/notification.o
$(OBJECT_DIR)/object/cnode.o: src/$(OBJECT_DIR)/cnode.c
	$(CC) $(CFLAGS) $(INCLUDE) src/$(OBJECT_DIR)/cnode.c -c -o $(OBJECT_DIR)/object/cnode.o
$(OBJECT_DIR)/object/endpoint.o: src/$(OBJECT_DIR)/endpoint.c
	$(CC) $(CFLAGS) $(INCLUDE) src/$(OBJECT_DIR)/endpoint.c -c -o $(OBJECT_DIR)/object/endpoint.o
$(OBJECT_DIR)/object/interrupt.o: src/$(OBJECT_DIR)/interrupt.c
	$(CC) $(CFLAGS) $(INCLUDE) src/$(OBJECT_DIR)/interrupt.c -c -o $(OBJECT_DIR)/object/interrupt.o
$(OBJECT_DIR)/object/$(OBJECT_DIR)type.o: src/$(OBJECT_DIR)/objecttype.c
	$(CC) $(CFLAGS) $(INCLUDE) src/$(OBJECT_DIR)/objecttype.c -c -o $(OBJECT_DIR)/object/$(OBJECT_DIR)type.o
$(OBJECT_DIR)/object/tcb.o: src/$(OBJECT_DIR)/tcb.c
	$(CC) $(CFLAGS) $(INCLUDE) src/$(OBJECT_DIR)/tcb.c -c -o $(OBJECT_DIR)/object/tcb.o
$(OBJECT_DIR)/object/untyped.o: src/$(OBJECT_DIR)/untyped.c
	$(CC) $(CFLAGS) $(INCLUDE) src/$(OBJECT_DIR)/untyped.c -c -o $(OBJECT_DIR)/object/untyped.o
$(OBJECT_DIR)/model/preemption.o: src/model/preemption.c
	$(CC) $(CFLAGS) $(INCLUDE) src/model/preemption.c -c -o $(OBJECT_DIR)/model/preemption.o
$(OBJECT_DIR)/model/statedata.o: src/model/statedata.c
	$(CC) $(CFLAGS) $(INCLUDE) src/model/statedata.c -c -o $(OBJECT_DIR)/model/statedata.o
$(OBJECT_DIR)/model/smp.o: src/model/smp.c
	$(CC) $(CFLAGS) $(INCLUDE) src/model/smp.c -c -o $(OBJECT_DIR)/model/smp.o
$(OBJECT_DIR)/machine/io.o: src/machine/io.c
	$(CC) $(CFLAGS) $(INCLUDE) src/machine/io.c -c -o $(OBJECT_DIR)/machine/io.o
$(OBJECT_DIR)/machine/registerset.o: src/machine/registerset.c
	$(CC) $(CFLAGS) $(INCLUDE) src/machine/registerset.c -c -o $(OBJECT_DIR)/machine/registerset.o
$(OBJECT_DIR)/machine/fpu.o: src/machine/fpu.c
	$(CC) $(CFLAGS) $(INCLUDE) src/machine/fpu.c -c -o $(OBJECT_DIR)/machine/fpu.o
$(OBJECT_DIR)/benchmark/benchmark_track.o: src/benchmark/benchmark_track.c
	$(CC) $(CFLAGS) $(INCLUDE) src/benchmark/benchmark_track.c -c -o $(OBJECT_DIR)/benchmark/benchmark_track.o
$(OBJECT_DIR)/benchmark/benchmark_utilisation.o: src/benchmark/benchmark_utilisation.c
	$(CC) $(CFLAGS) $(INCLUDE) src/benchmark/benchmark_utilisation.c -c -o $(OBJECT_DIR)/benchmark/benchmark_utilisation.o
$(OBJECT_DIR)/smp/lock.o: src/smp/lock.c
	$(CC) $(CFLAGS) $(INCLUDE) src/smp/lock.c -c -o $(OBJECT_DIR)/smp/lock.o
$(OBJECT_DIR)/smp/ipi.o: src/smp/ipi.c
	$(CC) $(CFLAGS) $(INCLUDE) src/smp/ipi.c -c -o $(OBJECT_DIR)/smp/ipi.o

$(OBJECT_DIR)/arch/x86/c_traps.o: src/arch/x86/c_traps.c
	$(CC) $(CFLAGS) $(INCLUDE) src/arch/x86/c_traps.c -c -o $(OBJECT_DIR)/arch/x86/c_traps.o
$(OBJECT_DIR)/arch/x86/idle.o: src/arch/x86/idle.c
	$(CC) $(CFLAGS) $(INCLUDE) src/arch/x86/idle.c -c -o $(OBJECT_DIR)/arch/x86/idle.o
$(OBJECT_DIR)/arch/x86/api/faults.o: src/arch/x86/api/faults.c
	$(CC) $(CFLAGS) $(INCLUDE) src/arch/x86/api/faults.c -c -o $(OBJECT_DIR)/arch/x86/api/faults.o
$(OBJECT_DIR)/arch/x86/object/interrupt.o: src/arch/x86/object/interrupt.c
	$(CC) $(CFLAGS) $(INCLUDE) src/arch/x86/object/interrupt.c -c -o $(OBJECT_DIR)/arch/x86/object/interrupt.o
$(OBJECT_DIR)/arch/x86/object/ioport.o: src/arch/x86/object/ioport.c
	$(CC) $(CFLAGS) $(INCLUDE) src/arch/x86/object/ioport.c -c -o $(OBJECT_DIR)/arch/x86/object/ioport.o
$(OBJECT_DIR)/arch/x86/object/objecttype.o: src/arch/x86/object/objecttype.c
	$(CC) $(CFLAGS) $(INCLUDE) src/arch/x86/object/objecttype.c -c -o $(OBJECT_DIR)/arch/x86/object/objecttype.o
$(OBJECT_DIR)/arch/x86/object/tcb.o: src/arch/x86/object/tcb.c
	$(CC) $(CFLAGS) $(INCLUDE) src/arch/x86/object/tcb.c -c -o $(OBJECT_DIR)/arch/x86/object/tcb.o
$(OBJECT_DIR)/arch/x86/object/iospace.o: src/arch/x86/object/iospace.c
	$(CC) $(CFLAGS) $(INCLUDE) src/arch/x86/object/iospace.c -c -o $(OBJECT_DIR)/arch/x86/object/iospace.o
$(OBJECT_DIR)/arch/x86/object/vcpu.o: src/arch/x86/object/vcpu.c
	$(CC) $(CFLAGS) $(INCLUDE) src/arch/x86/object/vcpu.c -c -o $(OBJECT_DIR)/arch/x86/object/vcpu.o
$(OBJECT_DIR)/arch/x86/kernel/vspace.o: src/arch/x86/kernel/vspace.c
	$(CC) $(CFLAGS) $(INCLUDE) src/arch/x86/kernel/vspace.c -c -o $(OBJECT_DIR)/arch/x86/kernel/vspace.o
$(OBJECT_DIR)/arch/x86/kernel/apic.o: src/arch/x86/kernel/apic.c
	$(CC) $(CFLAGS) $(INCLUDE) src/arch/x86/kernel/apic.c -c -o $(OBJECT_DIR)/arch/x86/kernel/apic.o
$(OBJECT_DIR)/arch/x86/kernel/xapic.o: src/arch/x86/kernel/xapic.c
	$(CC) $(CFLAGS) $(INCLUDE) src/arch/x86/kernel/xapic.c -c -o $(OBJECT_DIR)/arch/x86/kernel/xapic.o
$(OBJECT_DIR)/arch/x86/kernel/x2apic.o: src/arch/x86/kernel/x2apic.c
	$(CC) $(CFLAGS) $(INCLUDE) src/arch/x86/kernel/x2apic.c -c -o $(OBJECT_DIR)/arch/x86/kernel/x2apic.o
$(OBJECT_DIR)/arch/x86/kernel/boot_sys.o: src/arch/x86/kernel/boot_sys.c
	$(CC) $(CFLAGS) $(INCLUDE) src/arch/x86/kernel/boot_sys.c -c -o $(OBJECT_DIR)/arch/x86/kernel/boot_sys.o
$(OBJECT_DIR)/arch/x86/kernel/smp_sys.o: src/arch/x86/kernel/smp_sys.c
	$(CC) $(CFLAGS) $(INCLUDE) src/arch/x86/kernel/smp_sys.c -c -o $(OBJECT_DIR)/arch/x86/kernel/smp_sys.o
$(OBJECT_DIR)/arch/x86/kernel/boot.o: src/arch/x86/kernel/boot.c
	$(CC) $(CFLAGS) $(INCLUDE) src/arch/x86/kernel/boot.c -c -o $(OBJECT_DIR)/arch/x86/kernel/boot.o
$(OBJECT_DIR)/arch/x86/kernel/cmdline.o: src/arch/x86/kernel/cmdline.c
	$(CC) $(CFLAGS) $(INCLUDE) src/arch/x86/kernel/cmdline.c -c -o $(OBJECT_DIR)/arch/x86/kernel/cmdline.o
$(OBJECT_DIR)/arch/x86/kernel/ept.o: src/arch/x86/kernel/ept.c
	$(CC) $(CFLAGS) $(INCLUDE) src/arch/x86/kernel/ept.c -c -o $(OBJECT_DIR)/arch/x86/kernel/ept.o
$(OBJECT_DIR)/arch/x86/kernel/thread.o: src/arch/x86/kernel/thread.c
	$(CC) $(CFLAGS) $(INCLUDE) src/arch/x86/kernel/thread.c -c -o $(OBJECT_DIR)/arch/x86/kernel/thread.o
$(OBJECT_DIR)/arch/x86/model/statedata.o: src/arch/x86/model/statedata.c
	$(CC) $(CFLAGS) $(INCLUDE) src/arch/x86/model/statedata.c -c -o $(OBJECT_DIR)/arch/x86/model/statedata.o
$(OBJECT_DIR)/arch/x86/machine/hardware.o: src/arch/x86/machine/hardware.c
	$(CC) $(CFLAGS) $(INCLUDE) src/arch/x86/machine/hardware.c -c -o $(OBJECT_DIR)/arch/x86/machine/hardware.o
$(OBJECT_DIR)/arch/x86/machine/fpu.o: src/arch/x86/machine/fpu.c
	$(CC) $(CFLAGS) $(INCLUDE) src/arch/x86/machine/fpu.c -c -o $(OBJECT_DIR)/arch/x86/machine/fpu.o
$(OBJECT_DIR)/arch/x86/machine/cpu_identification.o: src/arch/x86/machine/cpu_identification.c
	$(CC) $(CFLAGS) $(INCLUDE) src/arch/x86/machine/cpu_identification.c -c -o $(OBJECT_DIR)/arch/x86/machine/cpu_identification.o
$(OBJECT_DIR)/arch/x86/machine/breakpoint.o: src/arch/x86/machine/breakpoint.c
	$(CC) $(CFLAGS) $(INCLUDE) src/arch/x86/machine/breakpoint.c -c -o $(OBJECT_DIR)/arch/x86/machine/breakpoint.o
$(OBJECT_DIR)/arch/x86/machine/registerset.o: src/arch/x86/machine/registerset.c
	$(CC) $(CFLAGS) $(INCLUDE) src/arch/x86/machine/registerset.c -c -o $(OBJECT_DIR)/arch/x86/machine/registerset.o
$(OBJECT_DIR)/arch/x86/machine/capdl.o: src/arch/x86/machine/capdl.c
	$(CC) $(CFLAGS) $(INCLUDE) src/arch/x86/machine/capdl.c -c -o $(OBJECT_DIR)/arch/x86/machine/capdl.o
$(OBJECT_DIR)/arch/x86/benchmark/benchmark.o: src/arch/x86/benchmark/benchmark.c
	$(CC) $(CFLAGS) $(INCLUDE) src/arch/x86/benchmark/benchmark.c -c -o $(OBJECT_DIR)/arch/x86/benchmark/benchmark.o
$(OBJECT_DIR)/arch/x86/smp/ipi.o: src/arch/x86/smp/ipi.c
	$(CC) $(CFLAGS) $(INCLUDE) src/arch/x86/smp/ipi.c -c -o $(OBJECT_DIR)/arch/x86/smp/ipi.o

$(OBJECT_DIR)/arch/x86/64/c_traps.o: src/arch/x86/64/c_traps.c
	$(CC) $(CFLAGS) $(INCLUDE) src/arch/x86/64/c_traps.c -c -o $(OBJECT_DIR)/arch/x86/64/c_traps.o
$(OBJECT_DIR)/arch/x86/64/object/objecttype.o: src/arch/x86/64/object/objecttype.c
	$(CC) $(CFLAGS) $(INCLUDE) src/arch/x86/64/object/objecttype.c -c -o $(OBJECT_DIR)/arch/x86/64/object/objecttype.o
$(OBJECT_DIR)/arch/x86/64/kernel/thread.o: src/arch/x86/64/kernel/thread.c
	$(CC) $(CFLAGS) $(INCLUDE) src/arch/x86/64/kernel/thread.c -c -o $(OBJECT_DIR)/arch/x86/64/kernel/thread.o
$(OBJECT_DIR)/arch/x86/64/kernel/vspace.o: src/arch/x86/64/kernel/vspace.c
	$(CC) $(CFLAGS) $(INCLUDE) src/arch/x86/64/kernel/vspace.c -c -o $(OBJECT_DIR)/arch/x86/64/kernel/vspace.o
$(OBJECT_DIR)/arch/x86/64/kernel/elf.o: src/arch/x86/64/kernel/elf.c
	$(CC) $(CFLAGS) $(INCLUDE) src/arch/x86/64/kernel/elf.c -c -o $(OBJECT_DIR)/arch/x86/64/kernel/elf.o
$(OBJECT_DIR)/arch/x86/64/model/statedata.o: src/arch/x86/64/model/statedata.c
	$(CC) $(CFLAGS) $(INCLUDE) src/arch/x86/64/model/statedata.c -c -o $(OBJECT_DIR)/arch/x86/64/model/statedata.o
$(OBJECT_DIR)/arch/x86/64/model/smp.o: src/arch/x86/64/model/smp.c
	$(CC) $(CFLAGS) $(INCLUDE) src/arch/x86/64/model/smp.c -c -o $(OBJECT_DIR)/arch/x86/64/model/smp.o
$(OBJECT_DIR)/arch/x86/64/machine/registerset.o: src/arch/x86/64/machine/registerset.c
	$(CC) $(CFLAGS) $(INCLUDE) src/arch/x86/64/machine/registerset.c -c -o $(OBJECT_DIR)/arch/x86/64/machine/registerset.o
$(OBJECT_DIR)/arch/x86/64/smp/ipi.o: src/arch/x86/64/smp/ipi.c
	$(CC) $(CFLAGS) $(INCLUDE) src/arch/x86/64/smp/ipi.c -c -o $(OBJECT_DIR)/arch/x86/64/smp/ipi.o

$(OBJECT_DIR)/arch/x86/multiboot.o: src/arch/x86/multiboot.S
	$(CC) $(CFLAGS) $(INCLUDE) src/arch/x86/multiboot.S -c -o $(OBJECT_DIR)/arch/x86/multiboot.o
$(OBJECT_DIR)/arch/x86/64/machine_asm.o: src/arch/x86/64/machine_asm.S
	$(CC) $(CFLAGS) $(INCLUDE) src/arch/x86/64/machine_asm.S -c -o $(OBJECT_DIR)/arch/x86/64/machine_asm.o
$(OBJECT_DIR)/arch/x86/64/traps.o: src/arch/x86/64/traps.S
	$(CC) $(CFLAGS) $(INCLUDE) src/arch/x86/64/traps.S -c -o $(OBJECT_DIR)/arch/x86/64/traps.o
$(OBJECT_DIR)/arch/x86/64/head.o: src/arch/x86/64/head.S
	$(CC) $(CFLAGS) $(INCLUDE) src/arch/x86/64/head.S -c -o $(OBJECT_DIR)/arch/x86/64/head.o

$(OBJECT_DIR)/plat/pc99/machine/acpi.o: src/plat/pc99/machine/acpi.c
	$(CC) $(CFLAGS) $(INCLUDE) src/plat/pc99/machine/acpi.c -c -o $(OBJECT_DIR)/plat/pc99/machine/acpi.o
$(OBJECT_DIR)/plat/pc99/machine/hardware.o: src/plat/pc99/machine/hardware.c
	$(CC) $(CFLAGS) $(INCLUDE) src/plat/pc99/machine/hardware.c -c -o $(OBJECT_DIR)/plat/pc99/machine/hardware.o
$(OBJECT_DIR)/plat/pc99/machine/pic.o: src/plat/pc99/machine/pic.c
	$(CC) $(CFLAGS) $(INCLUDE) src/plat/pc99/machine/pic.c -c -o $(OBJECT_DIR)/plat/pc99/machine/pic.o
$(OBJECT_DIR)/plat/pc99/machine/ioapic.o: src/plat/pc99/machine/ioapic.c
	$(CC) $(CFLAGS) $(INCLUDE) src/plat/pc99/machine/ioapic.c -c -o $(OBJECT_DIR)/plat/pc99/machine/ioapic.o
$(OBJECT_DIR)/plat/pc99/machine/pit.o: src/plat/pc99/machine/pit.c
	$(CC) $(CFLAGS) $(INCLUDE) src/plat/pc99/machine/pit.c -c -o $(OBJECT_DIR)/plat/pc99/machine/pit.o
$(OBJECT_DIR)/plat/pc99/machine/io.o: src/plat/pc99/machine/io.c
	$(CC) $(CFLAGS) $(INCLUDE) src/plat/pc99/machine/io.c -c -o $(OBJECT_DIR)/plat/pc99/machine/io.o
$(OBJECT_DIR)/plat/pc99/machine/intel-vtd.o: src/plat/pc99/machine/intel-vtd.c
	$(CC) $(CFLAGS) $(INCLUDE) src/plat/pc99/machine/intel-vtd.c -c -o $(OBJECT_DIR)/plat/pc99/machine/intel-vtd.o
$(OBJECT_DIR)/config/default_domain.o: src/config/default_domain.c
	$(CC) $(CFLAGS) $(INCLUDE) src/config/default_domain.c -c -o $(OBJECT_DIR)/config/default_domain.o

kernel.elf: $(OBJECTS)
	$(CPP) $(INCLUDE) src/plat/pc99/linker.lds -o lds.tmp -P -DCONFIG_ARCH_X86_64
	$(LD) $(LDFLAGS) $(OBJECTS) -T lds.tmp -o kernel.elf

kernel: kernel.elf
	objcopy -O elf32-i386 kernel.elf kernel

clean:
	rm -f $(OBJECTS)
	rm -f lds.tmp
	rm -f kernel.elf
	rm -f kernel
	rm -rf $(OBJECT_DIR)