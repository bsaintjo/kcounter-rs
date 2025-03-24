docker:
    docker run --rm -it -v "$(pwd)":/src -w /src linux-dev:latest /bin/bash

build kdir:
    make ARCH=riscv LLVM=1 KDIR={{kdir}}

rust-analyzer kdir:
    make LLVM=1 ARCH=riscv -C {{kdir}} M=$PWD rust-analyzer