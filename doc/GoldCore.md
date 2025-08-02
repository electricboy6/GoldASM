# Gold Core Reference
## Memory Layout
### Basic layout
All memory addresses are RAM and are executable, except for ``0000-01FF`` and ``FF00-FFFF``.  
``0000-01FF`` is the zero page and stack.
### The Last Page
``FF00-FFFF`` is the last page in memory, and it is special. ``FFFC-FFFF`` are reserved for jump vectors, and
``FFF8-FFFB`` should contain an infinite loop that is jumped to at the end of the program. ``FFF0-FFF7`` is reserved for
IO.
### IO
Any blocks of IO not defined are unused (so far)
#### Serial
``FF00-FF10`` is reserved for the serial interface.
``FF00-FF03`` are used for tx, while ``FF08-FF09`` are used for rx.
##### Tx
``FF00`` is the outgoing byte. ``FF01`` is the new data flag (tells the serial interface to start sending),
``FF02`` is the block flag (tells the serial interface to stop sending), and ``FF03`` is the busy flag. All inputs are
write-only except for the busy flag, which is read-only. The data on the outgoing bus is input to a FIFO on each clock
cycle where the new data flag is high (the new data flag is reset after the data is written to the FIFO), and the busy
flag will go high when the FIFO is full.
##### Rx
`FF08` is the incoming byte. ``FF09`` is the new data flag (must be manually reset). ``FF0A`` is the busy flag (tells
the other end to stop sending new characters)