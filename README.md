# GoldASM
Video demo for Hack Club voters: https://youtu.be/B5GBNrQVszc?si=sB_gBukX7l-n5j-C
(in case you don't want to have to run it yourself, this is a great short overview that I recommend watching).
### What is GoldASM?
- GoldASM is an assembler targeting the Gold ISA
- It is a command line tool written in Rust
### Features
#### Assembler
- Outputs the complete memory image
- Includes, defines, comments, and other language features
- Very fast
- Creates a symbol table for the simulator to use
#### Simulator 
- Optimized, able to do >1000 cycles/second
- Full dissembly with a symbol table
- Gives live readouts of the stack, registers, and other important information
- Full simulated serial port

### Basic usage
Just run the ``--help`` subcommand, everything else should be obvious after that.
For voters/reviewers - I HIGHLY recommend either watching the video or just using a demo program (in the examples folder, you'll probably also need the stuff in lib), because you'll otherwise need to learn the language.

### What is the Gold ISA?
- The Gold ISA is a lightweight ISA targeting small FPGAs, especially the Alchitry Au v2 (hence the name)
- It is designed to be easily used on 8, 16, or 32 bit processors
- It is *not* designed to be fast or efficient, just simple and easy to understand
- It is also *not* particularly stable, and makes **no** guarantees about compatibility
### What is the state of development?
GoldASM, the Gold ISA, and the Gold assembly language are still in active development.
There is still a lot of work to do for them to be ready, including a lot of tooling development and the actual hardware
design to be done on the FPGA. This FPGA development is probably going to be the next focus.

If you want to learn more, take a look at the documentation in the ``doc`` folder.
