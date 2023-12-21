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

; macro with zero parameter. Parameters here: %1 %2 ... %n
; by usepoint: macro_name param1, param2 (labels)

%macro half_adder 0
    # See picture: adder_nand_half.jpg
    tmp = nand(a, b)      ; U1 -> U3
    carry = nand(tmp tmp) ; U2 -> carry
    a = nand(tmp, a)      ; U3 -> U5
    b = nand(tmp, b)      ; U4 -> U5
    a = nand(a, b)        ; U5 -> a_out
%endmacro

%macro full_adder 0
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
%endmacro

; Input: A as input and B bits (as carry)
; Output: A as result and B as carry
%macro half_adder_for_counter 0
    tmp = nand(A, B)   ; U1 -> U2, U3, U4
    a = nand(tmp, a)   ; U3 -> U5
    b = nand(tmp, B)   ; U4 -> U5
    a = nand(a, b)     ; U5 --> pin A
    b = nand(tmp, tmp) ; U2 --> pin B (here is the carry)
%endmacro
