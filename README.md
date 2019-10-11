# Dora

[![Join the chat at https://gitter.im/dora-lang/dora](https://badges.gitter.im/dora-lang/Lobby.svg)](https://gitter.im/dora-lang/Lobby?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge) [![Build Status](https://travis-ci.org/dinfuehr/dora.svg?branch=master)](https://travis-ci.org/dinfuehr/dora)

Dora is both the name of the custom programming language and of the JIT-compiler. 

Architecture
The architecture is pretty simple: dora hello.dora parses the given input file into an Abstract Syntax Tree (AST). After parsing, the whole AST is semantically checked, if this succeeds execution starts with the main function.

To execute main, machine-code is generated for that function by the baseline compiler by traversing the AST nodes of the function. The function is traversed twice, first to generate information (mostly about the stack frame), the second traversal then generates the machine code.

The baseline compiler is a method-based compiler and not a tracing JIT like for example LuaJIT. The purpose of the baseline compiler in Dora is to generate code as fast as possible, not to generate the most efficient code. The sooner it finishes code generation, the sooner execution can start.

Many VMs like the OpenJDK or V8 pair the baseline compiler (and/or interpreter) with one or more optimizing compilers that compile functions to more efficient machine-code if it detects a function to be hot. The optimizing compiler needs longer to compile a given function, but generates more optimized machine-code. This is acceptable since not all code gets compiled by the optimizing compiler but only hot code/functions. Dora doesn’t have an optimizing compiler at the moment, but I have plans to implement one.

Compilation
The baseline compiler uses a MacroAssembler to generate machine code. All differences between different Instruction Set Architectures (ISAs) are handled by the MacroAssembler. Dora can generate machine-code for x86_64 and AArch64. Adding other ISAs should be possible without touching the baseline compiler.

JIT-compiler for the programming language Dora implemented in Rust.
Works on Linux (x86\_64, aarch64) and macOS (x86\_64).
Build with:

## Dependencies
You need to install these dependencies:

```
# on Fedora
$ sudo dnf install capstone-devel ruby

# on Ubuntu/Debian
$ sudo apt install libcapstone-dev ruby

# on MacOS capstone can be installed via homebrew
$ brew install capstone
```

[Ruby](https://www.ruby-lang.org/) is used for running tests, while [capstone](https://github.com/aquynh/capstone) is used for instruction decoding/disassembling machine code.


## Compilation & Testing
Install current Rust Nightly via [rustup.rs](http://rustup.rs). The nightly version of
Rust is needed because Dora uses some unstable features of Rust (e.g. inline assembly).

Dora uses [cargo](http://crates.io) for building, which is bundled with Rust:

```
# install last nightly and use it for this project
rustup update nightly
rustup override set nightly

# run all tests in debug and release mode
tools/test
tools/test-release
```
