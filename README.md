loop-escape-rs
==========

This repository contains a rust program to attach to a running process & modify
execution to escape from an infinite loop.

### Example usage

* Compile one of the example programs in `examples/`, eg `rustc example1.rs`.
* Start the example running, eg: `./example1`
* Run the escaper (you probably need to use sudo), eg: `sudo cargo run`
* Follow the directions to enter the process name and choose the escape strategy

### Supported platforms

Only tested on macOS running on apple silicon (M*).

Will not work on other architectures (x86/x64) or any platforms without the
`mach2` crate available.

### References

Draws heavily from this [example
program](https://github.com/JohnTitor/mach2/blob/main/examples/dump_process_registers.rs)
in the `mach2` crate

Useful blog post: [Replacing
ptrace()](http://uninformed.org/index.cgi?v=4&a=3&p=14)

Related previous project: [Detecting and Escaping Infinite Loops with
Bolt](https://groups.csail.mit.edu/pac/bolt/)

