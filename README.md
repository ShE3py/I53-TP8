# Yet Another Compiler

A pseudolanguage compiler;
```
FUNCTION fibo(i)
BEGIN
    IF i <= 1 THEN
        RETURN i;
    ELSE
        RETURN fibo(i - 2) + fibo(i - 1);
    EIF
END

FUNCTION main()
BEGIN
    READ n;
    RETURN fibo(n);
END
```

See [compiler/LISEZMOI.md](compiler/LISEZMOI.md) (french) for the language reference.

## Backends

Two backends are available:
- Random-access machine (see [rame/README.md](rame/README.md))
- LLVM (work in progress)

## File structure
- `compiler/`: lexing, parsing, AST (Flex, Yacc, C)
- `compiler/ram/`: RA-machine backend (C)
- `compiler/llvm/`: LLVM backend (C++)
- `rame/`: RA-machine emulator and optimizer library (Rust)
- `driver/`: executables (Rust)

## Usage

Requires Clang & GNU Make.

### Without Rust

Just run `make` in `compiler/`.

```
./arc infile [-o outfile]
```

### With Rust

Available binaries: `rame-cc`, `rame-opt`, `rame-run`, `rame-test`.

```
Usage: rame-run [OPTIONS] <infile> [args]...

Arguments:
  <infile>   The program to run
  [args]...  The program's arguments

Options:
  -b, --bits <BITS>  The integer's type bits [default: 16] [possible values: 8, 16, 32, 64, 128]
  -O                 Optimize the RAM program before running it
  -c                 Compile the algorithmic program as a first step
  -h, --help         Print help
  -V, --version      Print version
```

## Example

’Tis but a reminder for meowself, as the LLVM backend is still a WIP.

```
FONCTION main()
DÉBUT
    AFFICHER 1 + 2;
FIN
```
```
$ cargo run --bin rame-cc -- ./add.algo -o add.out
$ clang add.out compiler/src/llvm/intrinsics.c
```
