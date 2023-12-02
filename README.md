# One instruction CPU

## One bit logical instruction
Subtypes: nand, nor, xor, xnor

* bitcpu: simple version
* bitcpu-call: with '''jmp, call, ret''' special register extension

By the jmp and call instructions the next word is an address.


    $ echo '1111 1111' | target/release/bitcpu sample/add_4bit.nand
    $ echo '1111 1111' | target/release/bitcpu sample/add_4bit.nand trace # for debug

    $ echo '1111 1111' | target/release/bitcpu-call sample/add_4bit.nand
    $ echo '1111 1111' | target/release/bitcpu-call sample/add_4bit.nand trace # for debug

## One u32 instruction (u8, u8, i16)
Subtype: subleq and addleq

* subleq: simple version; (regA, regB, jmpaddr)
* addleq: it can't nullable the registers
