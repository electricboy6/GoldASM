# Gold ISA 8-bit Specification
## Outline
The Gold ISA is designed to be an 8 bit ISA with variable instruction length.
Up to 2 immediate values can be in an instruction, so instructions may be up to 3 bytes long. Inspiration was taken from
the w65c02's ISA. The ISA is big-endian.
### Basic Specifications
The ISA requires a 16 bit stack pointer register, a 16 bit program counter, an 8 bit status register, and
8 general-purpose 8 bit registers, as well as an accumulator. The address bus is 16 bits wide. Supported addressing modes
include absolute, absolute indexed, zero page, and zero page indexed.  
#### Reset Vector
Upon reset, the CPU should load the values in memory locations 0xFFFE and 0xFFFF to see which address to jump to for the
program to start. This is in absolute addressing mode.
#### The Zero Page
The zero page is the first 256 bytes of the address space, where the address looks like ``0x00zz``. Generally, working
with the zero page is faster and uses less memory per operation than working with the rest of memory.
#### The Stack
The stack is another 256 byte region of memory, located at ``0x01zz``. It is accessed by operations dealing directly
with the stack, and I do not recommend using other operations with this memory region. It is, however, still legal to do
so.
#### Absolute addressing
Absolute addressing is giving the entire 16 bit address (unless it's the zero page, where it's 8 bit)  
#### Indexed addressing
Indexed addressing is adding the value of a register with the immediate and using the value at that memory address
as the new address (Example: (register 7 is ``0x03``) ``LDR 0x0C06,0x07`` would result in ``LDR 0x0C0D``)
### Supported ALU Operations
The ISA supports add with carry, subtract with carry, xor, xnor, or, nor, and, nand, not, rotate right, rotate left, shift right,
shift left, set carry, clear carry, and potentially more instructions in the future if I think it's necessary.
### Supported Memory Operations
The ISA supports pushing a register to the stack, popping a register from the stack, loading a value from RAM into the
accumulator, storing a value from the accumulator to RAM, copying the accumulator to a register, copying a register to
the accumulator, loading an immediate to the accumulator, and potentially more instructions if I see fit.
### Supported Flow Control Operations
The ISA supports branching on carry set, branching on carry not set, branching on negative, branching on positive,
branching on equal, branching on not equal, branching on zero, branching on not zero, pushing the program counter to the
stack, popping the program counter from the stack, and jumping (subroutines are implemented with jumps, pushing to the
stack, and pulling from the stack).
## Instructions
### Instruction Format
Instructions are in the format of instruction, optional immediate, and optional immediate. The number of immediates
depends on the instruction. For the purpose of examples, a value of ``z`` in hexadecimal indicates a value to be filled
in by a parameter.  
An example of this is as follows. ``0xzz`` would be an instruction that takes no immediate, ``0xzz 0xzz`` would take one
immediate, and ``0xzz 0xzz 0xzz`` would use both immediates.  
Timings are not specified in the ISA, but the CPU implementing them should allow each instruction to finish before
fetching the next, unless it is designed to be pipelined.
### ALU
Opcodes ``0x00``-``0x20`` are reserved for the ALU. If an ALU instruction has no immediates, it operates on the
accumulator. If it has one immediate, it operates on the accumulator then the register specified by the immediate. If it
has 2 immediates, it operates on the register specified by the first immediate, then the register specified by the second
immediate.
#### Noop
``0x00``
#### Addition
``0x01 0xzz``, ``0x02 0xzz 0xzz``
#### Subtraction
``0x03 0xzz``, ``0x04 0xzz 0xzz``
#### Set Carry/Clear Carry
``0x05``, ``0x06``
#### XOR/XNOR
``0x07 0xzz``, ``0x08 0xzz 0xzz``, ``0x09 0xzz``, ``0x0A 0xzz 0xzz``
#### OR/NOR
``0x0B 0xzz``, ``0x0C 0xzz 0xzz``, ``0x0D 0xzz``, ``0x0E 0xzz 0xzz``
#### AND/NAND
``0x0F 0xzz``, ``0x10 0xzz 0xzz``, ``0x11 0xzz``, ``0x12 0xzz 0xzz``
#### NOT
``0x13``
#### ROR/ROL
``0x14``, ``0x15``
#### SHR/SHL
``0x16``, ``0x17``
### Memory
Opcodes ``0x21``-``0x41`` have been reserved for memory.
#### PHR/PLR
``0x21 0xzz`` - pushes the register specified by the immediate to the stack  
``0x22 0xzz`` - pulls the top value from the stack into the register specified by the immediate
#### LDA
``0x23 0xzz 0xzz`` - absolute  
``0x24 0xzz 0xzz 0xzz`` - indexed  
``0x25 0xzz`` - zero page  
``0x26 0xzz 0xzz`` - zero page indexed  
``0x27 0xzz`` - load immediate
#### STA
``0x28 0xzz 0xzz`` - absolute  
``0x29 0xzz 0xzz 0xzz`` - indexed  
``0x2A 0xzz`` - zero page  
``0x2B 0xzz 0xzz`` - zero page indexed
#### CPA/CPR
``0x2C 0xzz`` - Copy accumulator to register  
``0x2D 0xzz`` - Copy register to accumulator
### Flow Control
Opcodes ``0x42``-``0x62`` have been reserved for flow control.
#### BCS
``0x42 0xzz 0xzz`` - absolute  
``0x43 0xzz 0xzz 0xzz`` - indexed
#### BCC
``0x44 0xzz 0xzz`` - absolute  
``0x45 0xzz 0xzz 0xzz`` - indexed
#### BN
``0x46 0xzz 0xzz`` - absolute  
``0x47 0xzz 0xzz 0xzz`` - indexed
#### BP
``0x48 0xzz 0xzz`` - absolute  
``0x49 0xzz 0xzz 0xzz`` - indexed
#### BEQ
``0x4A 0xzz 0xzz 0xzz`` - register, absolute  
``0x4B 0xzz 0xzz 0xzz 0xzz`` - register, indexed
#### BNE
``0x4C 0xzz 0xzz 0xzz`` - register, absolute  
``0x4D 0xzz 0xzz 0xzz 0xzz`` - register, indexed
#### BZE
``0x4E 0xzz 0xzz`` - absolute  
``0x4F 0xzz 0xzz 0xzz`` - indexed
#### BNZ
``0x50 0xzz 0xzz`` - absolute  
``0x51 0xzz 0xzz 0xzz`` - indexed
#### JMP
``0x52 0xzz 0xzz`` - absolute  
``0x53 0xzz 0xzz 0xzz`` - indexed
#### PHPC/PLPC
``0x54``, ``0x55``
#### INCPC
``0x56``
### Other operations
Other operations besides the ones described should reset the CPU. The instruction ``FF`` is never valid and must reset
the CPU.