# GoldASM
### What is GoldASM?
- GoldASM is an assembler targeting the Gold ISA
- It is a command line tool written in Rust
### What is the Gold ISA?
- The Gold ISA is a lightweight ISA targeting small FPGAs, especially the Alchitry Au v2 (hence the name)
- It is designed to be easily used on 8, 16, or 32 bit processors
- It is *not* designed to be fast or efficient, just simple and easy to understand
- It is also *not* particularly stable, and makes **no** guarantees about compatibility
### What is the state of development?
GoldASM, the Gold ISA, and the Gold assembly language are still in active development.
There is still a lot of work to do for them to be ready, including a lot of tooling development and the actual hardware
design to be done on the FPGA. This FPGA development cannot happen for a while yet, since I do not currently have an FPGA
to develop for, and can only simulate my designs.