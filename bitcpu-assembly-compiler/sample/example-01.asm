NAND_CPU       # first line: TYPE of VCPU

stdin  equ 0xfd     ; stdin, stdout
stdout equ 0xfd     ; stdin, stdout

skip equ 0xfe       ; write is skip next, equal with skip_nand(a, b);
low equ 0xfe        ; only read this
high equ 0xff       ; only read this

carry equ 0
a equ 1
b equ 2
tmp equ 3
tmp2 equ 4 ; full_adder

start:
    JMP realstart

half_adder:
    # See picture: adder_nand_half.jpg
    tmp = nand(a, b)      ; U1 -> U3
    carry = nand(tmp tmp) ; U2 -> carry
    a = nand(tmp, a)      ; U3 -> U5
    b = nand(tmp, b)      ; U4 -> U5
    a = nand(a, b)        ; U5 -> a_out
    RET

full_adder:
    # See picture: adder_nand_full.jpg
    tmp = nand(A, B)        ; U1 -> U2, U3, U9
    a = nand(tmp, A)        ; U2 -> U4
    b = nand(tmp, B)        ; U3 -> U4
    a = nand(a, b)          ; U4 -> U5, U6
    tmp2 = nand(a, Carry)   ; U5 -> U6, U7, U9
    a = nand(tmp2, a)       ; U6 -> U8
    b = nand(tmp2, Carry)   ; U7 -> U8
    a = nand(a, b)          ; U8 -> A_out
    carry = nand(tmp, tmp2) ; U9 -> Carry_out
    RET

; Input: A as input and B bits (as carry)
; Output: A as result and B as carry
half_adder_for_counter:
    tmp = nand(A, B)   ; U1 -> U2, U3, U4
    a = nand(tmp, a)   ; U3 -> U5
    b = nand(tmp, B)   ; U4 -> U5
    a = nand(a, b)     ; U5 --> pin A
    b = nand(tmp, tmp) ; U2 --> pin B (here is the carry)
    RET

realstart:
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
    CALL half_adder
    0x0f = nand(HIGH, a)

    a = nand(HIGH, 0x12)
    b = nand(HIGH, 0x16)
    CALL full_adder
    0x0e = nand(HIGH, a)

    a = nand(HIGH, 0x11)
    b = nand(HIGH, 0x15)
    CALL full_adder
    0x0d = nand(HIGH, a)

    a = nand(HIGH, 0x10)
    b = nand(HIGH, 0x14)
    CALL full_adder
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
    CALL half_adder_for_counter
    0x18 = nand(HIGH, a)

    # 1. bit
    a = nand(HIGH, 0x19)
    CALL half_adder_for_counter
    0x19 = nand(HIGH, a)

    b = nand(HIGH, b)  ; invert b (as carry)
    skip = nand(HIGH, b)
    jmp loop

    # For test only
    stdout = nand(HIGH, b) ; put b as negated carry bit
