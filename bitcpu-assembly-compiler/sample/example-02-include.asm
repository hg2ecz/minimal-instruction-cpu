NAND_CPU       # first line: TYPE of VCPU

%include "example-02-include-macro.inc"

start:
    0x18 = nand(LOW, LOW) ; inv zero, counter
    0x19 = nand(LOW, LOW) ; inv zero, counter

loop:
    # bit input: 0x10..0x17
    0x10 = nand(HIGH, stdin)
    0x11 = nand(HIGH, stdin)
    0x12 = nand(HIGH, stdin)
    0x13 = nand(HIGH, stdin)

    0x14 = nand(HIGH, stdin)
    0x15 = nand(HIGH, stdin)
    0x16 = nand(HIGH, stdin)
    0x17 = nand(HIGH, stdin)

    ### 0x00ffff # lowest carry 0 --> half adder
    a = nand(HIGH, 0x13)
    b = nand(HIGH, 0x17)
    half_adder                  ; macro
    0x0f = nand(HIGH, a)

    a = nand(HIGH, 0x12)
    b = nand(HIGH, 0x16)
    full_adder                  ; macro
    0x0e = nand(HIGH, a)

    a = nand(HIGH, 0x11)
    b = nand(HIGH, 0x15)
    full_adder                  ; macro
    0x0d = nand(HIGH, a)

    a = nand(HIGH, 0x10)
    b = nand(HIGH, 0x14)
    full_adder                  ; macro
    0x0c = nand(HIGH, 0x01)

    carry = nand(carry, carry) ; invert carry

    stdout = nand(HIGH, HIGH) ; put 0
    stdout = nand(HIGH, HIGH) ; put 0
    stdout = nand(HIGH, HIGH) ; put 0
    stdout = nand(HIGH, carry) ; carry
    stdout = nand(HIGH, 0x0c) ; put bit
    stdout = nand(HIGH, 0x0d) ; put bit
    stdout = nand(HIGH, 0x0e) ; put bit
    stdout = nand(HIGH, 0x0f) ; put bit

    # for counter icrement
    # 0. bit
    a = nand(HIGH, 0x18)
    b = nand(LOW, LOW)        ; b as carry (high)
    half_adder_for_counter    ; macro
    0x18 = nand(HIGH, a)

    # 1. bit
    a = nand(HIGH, 0x19)
    half_adder_for_counter    ; macro
    0x19 = nand(HIGH, a)

    b = nand(HIGH, b)  ; invert b (as carry)
    skip = nand(HIGH, b)
    jmp loop

    # For test only
    stdout = nand(HIGH, b) ; put b as negated carry bit
