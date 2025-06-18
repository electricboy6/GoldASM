# Gold assembly language specification
## Basic Syntax
### Includes
To include another file, use a ``#include filename``. This will simply append the file to the end of the file where it
was included. Circular includes will cause a stack overflow. Includes inside of files that have been included are also
supported.  
To use a subroutine/label in another file, put the filename (except for the .gasm part), then a dot, then the label.  
For example, ``jsr ~example.INIT``
### Comments
To make a comment, simply put ``//`` before the comment. Everything after the semicolon up to the newline will be  a comment.  
For example,  
```
lda #9B // this is a comment
```
### Flow Control
#### Labels
The syntax for a label is the name of the label in all uppercase followed by a ``:``. The label should be the only thing
on that line.  
For example,
```
LOOP:
    the rest of the code here
```
Please note that labels do not prevent execution of the code inside of them by advancement of the program counter.
#### Subroutines
The syntax for a subroutine is the same as a label, it just has ``sr`` in front of the name. Subroutines do, however,
prevent the program counter from advancing into the subroutine with jumps around the subroutine code.  
Please note that there **CANNOT** be a subroutine and a label with the same name, which also means that files that have
been included **CANNOT** have subroutines or labels with the same name as another subroutine or label.
#### Jumping to a Subroutine or Label
To jump to a subroutine or label, put a ``~`` in front of the name.
### Numbers and Memory
### Numbers
Numbers should be padded to the size expected by that instruction. By default, all numbers are memory addresses.  
If a 16 bit number is required, it should not be
separated.  
For example, ``F2C3`` instead of ``F2, C3``
#### Hexadecimal
By default, numbers are interpreted as hexadecimal.
#### Binary
To make the assembler interpret a number in binary, put a ``^`` in front of it.  
For example, ``^1010001``
#### Decimal
There is currently no support for decimal.
#### Immediates
To make a number just an immediate value, use a ``#`` in front of it.  
For example, ``lda #9B``
### Memory
#### Absolute
Putting a ``%`` in front of an address signifies that it is absolute  
For example, ``lda %F2C3``
#### Indexed
Putting a ``$`` in front of an address indicates that it is indexed  
For example, ``lda $B78E, FE``
#### Zero Page
The assembler will automatically infer that the addressing mode is zero paged if the syntax is correct.  
For example, ``lda A2``
#### Zero Page Indexed
The assembler will infer that the addressing mode is zero page indexed if the syntax is correct.  
For example, ``lda C3, $03``
### Instructions
Instructions are not case sensitive.
#### Parameters
Parameters are separated with commas.
#### Instruction List
The parameters of the function are in parenthesis. If there are multiple versions, they are nested.  
``a`` is absolute  
``i`` is indexed  
``zp`` is zero paged  
``zpi`` is zero page indexed  
``r`` is register  
``im`` is immediate
```
- noop
- add   ((r), (r, r))
- sub   ((r), (r, r))
- sc
- clc
- xor   ((r), (r, r))
- xnor  ((r), (r, r))
- or    ((r), (r, r))
- nor   ((r), (r, r))
- and   ((r), (r, r))
- nand  ((r), (r, r))
- not
- ror
- rol
- shr
- shl
- phr   (r)
- plr   (r)
- lda   ((a), (i), (zp), (zpi), (im))
- sta   ((a), (i), (zp), (zpi))
- cpa   (r)
- cpr   (r)
- bcs   ((a), (i))
- bcc   ((a), (i))
- bn    ((a), (i))
- bp    ((a), (i))
- beq   ((r, a), (r, i))
- bne   ((r, a), (r, i))
- bze   ((a), (i))
- bnz   ((a), (i))
- jmp   ((a), (i))
- jsr   ((a), (i))
- rts
```