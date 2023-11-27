# One instruction CPU

Special properties:
 * No opcode, only args
 * Args:  addrA addrB JMPaddr(relative)
 * 32 bit wide instruction, 2 x memory address (u8) and JMPaddr (i16).
 * arithmetic: i16
 * half of addressed memory is RAM, second half is preloaded "ROM" area. 
 * some memory address has a special function, e.g. addr-0 is INPUT/OUTPUT.

How it works?
 * mem[addrA] -= mem[addrB];  // on SUBLEQ -=, on ADDLEQ +=
 * if mem[addrA] <= 0 { PC += JMPaddr }

All other function can create as a memory mapped function.
