#!/bin/sh
cargo rustc -- -C no-redzone=no -C code-model=kernel -C relocation-model=dynamic-no-pic -C link-arg="-no-pie" -C target-feature="-sse,-sse2,-sse3,-mmx,+soft-float" -C target-cpu=nehalem