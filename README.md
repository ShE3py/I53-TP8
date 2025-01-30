# ARC

A pseudolanguage to abstract machine compiler;
```
FUNCTION fibo(i)
BEGIN
    IF i <= 1 THEN
        RETURN i;
    ELSE
        RETURN fibo(i - 2) + fibo(i - 1);
    FI
END

FUNCTION main()
BEGIN
    READ n;
    RETURN fibo(n);
END
```

Arc stands for Algo-Ram-Compiler.
See [compiler/LISEZMOI.md](compiler/LISEZMOI.md) (french) for the language reference.

## Installation

Requires GCC, GNU Make, Flex and Bison.
Just run `make` in [compiler/](./compiler).

## Usage

```
./arc infile [-o outfile]
```

# RAME

A RA-machine emulator and optimizer.
See [rame/README.md](rame/README.md) for more info.

## Installation

Requires Cargo.
Feature list:

|      Feature      | Description                                                                      |
|:-----------------:|:---------------------------------------------------------------------------------|
|    `compiler`     | Compiles `arc` into the binaries.                                                |
|    `optimizer`    | Enables the optimizer.                                                           |
| `indirect_jumps`  | Enables indirect jumps in the abstract machine; incompatible with the optimizer. |


The full suit:
```
cargo install --path driver
```

The Rust suit:
```
cargo install --path driver --no-default-features --features indirect_jumps
```

## Usage

```
Run an algorithmic or RAM program

Usage: rame-run [OPTIONS] <infile> [args]...

Arguments:
  <infile>   The program to run
  [args]...  The program's arguments

Options:
  -b, --bits <BITS>  The integers' width [default: 16] [possible values: 8, 16, 32, 64, 128]
  -O                 Optimize the RAM program before running it
  -c                 Compile the algorithmic program as a first step
  -h, --help         Print help
```

```
Compiles an algorithmic program into a RAM one

Usage: rame-cc [OPTIONS] <infile>

Arguments:
  <infile>  The program to compile

Options:
  -o <outfile>      Where to place the compiled program [default: a.out]
  -O                Turn on all optimizations
  -h, --help        Print help
```

```
Optimize a RAM program

Usage: rame-opt [OPTIONS] <infile>

Arguments:
  <infile>  The program to optimize

Options:
  -o <outfile>      Where to place the optimized program [default: a.out]
  -h, --help        Print help
```

```
Test an algorithmic program

Usage: rame-test [OPTIONS] [infile]...

Arguments:
  [infile]...  The files to test [default: tests]

Options:
  -c, --cc <compiler>  The path of the compiler to use
  -b, --bits <BITS>    The integers' width [default: 16] [possible values: 8, 16, 32, 64, 128]
  -h, --help           Print help
```

## Example

Integer range can be selected with `--bits`:
```
$ rame-run -c ./tests/fibo.algo 30
error: anon:135: "ADD @2": integer overflow
error: anon:135: help: ACC = 17711
error: anon:135: help: R26 = 28657
error: anon:135: help: using `--bits=16`; only values from -32768 to 32767 are accepted.

$ rame-run -cb32 ./tests/fibo.algo 30
Output = [832040]
```
