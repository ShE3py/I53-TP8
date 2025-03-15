# Driver

Binaries linking the compiler and the abstract machine emulator.
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
cargo install --path .
```

The Rust suit:
```
cargo install --path . --no-default-features --features indirect_jumps
```

## Usage

### Running

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
  -V, --version      Print version
```

Stdin can be read with `-`.

#### Example

```
$ rame-run -c ../tests/fibo.algo 30
error: anon:135: "ADD @2": integer overflow
error: anon:135: help: ACC = 17711
error: anon:135: help: R26 = 28657
error: anon:135: help: using `--bits=16`; only values from -32768 to 32767 are accepted.

$ rame-run -cb32 ./tests/fibo.algo 30
Output = [832040]
```

### Unit Testing

```
Test an algorithmic program

Usage: rame-test [OPTIONS] [infile]...

Arguments:
  [infile]...  The files to test [default: tests]

Options:
  -c, --cc <compiler>  The path of the compiler to use
  -b, --bits <BITS>    The integers' width [default: 16] [possible values: 8, 16, 32, 64, 128]
  -h, --help           Print help
  -V, --version        Print version

```

The files should start with a test header:
```
# TEST: [1, 2] => [3]
# TEST: [2, 1] => [3]
READ x;
READ y;
PRINT x + y;
```

By default, all the files in `tests/` are tested.

### Misc.

```
rame-cc [OPTIONS] <infile>
rame-opt [OPTIONS] <infile>
```
