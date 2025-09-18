# GoldASM
Video demo for Hack Club voters: https://youtu.be/B5GBNrQVszc?si=sB_gBukX7l-n5j-C
(in case you don't want to have to run it yourself, this is a great 1 minute overview that I recommend watching).

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
For voters/reviewers - I HIGHLY recommend either watching the video or just using a demo program (download the examples zip and unzip it), because you'll otherwise need to learn the language. ``serialtest.gasm`` is what I would recommend using. When it gets to just looping then it's waiting for input (press ``s`` to add something to the transmit buffer).

Note - This is a command line tool, after downloading the right file for your OS (and probably the examples folder), run the correct command in a terminal to run the tool

Base command - ``.\GoldASM.exe`` on Windows or ``./GoldASM.linux64`` on linux  

Note - There is a ``--help`` subcommand, run it if you need clarification.

#### Assembling
``baseCommand assemble sourceFile``

#### Simulating
``baseCommand simulate binaryFile`` or ``baseCommand simulate binaryFile symbolTable``


### What is the Gold ISA?
- The Gold ISA is a lightweight ISA targeting small FPGAs, especially the Alchitry Au v2 (hence the name)
- It is designed for 8 bit processors
- It is *not* designed to be fast or efficient, just simple and easy to understand
- It is also *not* particularly stable, and makes **no** guarantees about compatibility

### What is the state of development?
GoldASM, the Gold ISA, and the Gold assembly language are still in active development.
There is still a lot of work to do for them to be ready, including a lot of tooling development and the actual hardware
design to be done on the FPGA. This FPGA development is probably going to be the next focus.

If you want to learn more, take a look at the documentation in the ``doc`` folder.
