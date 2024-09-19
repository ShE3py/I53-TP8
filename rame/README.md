# RAME

A [random-access machine](https://en.wikipedia.org/wiki/Random-access_machine) emulator and optimizer.

```
Run, test or optimize a RAM program

Usage: rame [OPTIONS] <FILE> [ARGS]...

Arguments:
<FILE>     The program to execute
[ARGS]...  The program's arguments

Options:
-b, --bits <BITS>         The integer's type bits [default: 16] [possible values: 8, 16, 32, 64, 128]
-t, --test [<OUTPUT>...]  Test the program's output
-o, --optimize [<FILE>]   Optimize the program into the specified file (experimental)
-h, --help                Print help
-V, --version             Print version
```

The binary is built for integers only, but the library should work with rational or complex numbers, matrices,
or any other type. The optimizer only requires the numbers to have additive and multiplicative
identities (i.e. `0` and `1`).

## Runner

A hyphen can be used as the `<FILE>` to read from stdin:
```
$ rame - 1,2
0 | READ      ; R0 = 1
1 | STORE 1   ; R1 = ACC
2 | READ      ; R0 = 2
3 | ADD 1     ; R0 = R0 + R1
4 | WRITE     ; 3
5 | STOP
6 |           ; <eof>
Output = [3]
```

Integer range can be selected with `--bits`:
```
$ rame fibo.out 30
error: anon:141: "ADD @2": integer overflow
error: anon:141: help: ACC = 17711
error: anon:141: help: R26 = 28657
error: anon:141: help: using `--bits=16`; only values from -32768 to 32767 are accepted.

$ rame fibo.out 30 -b32
Output = [832040]
```

Unit tests can be run with `--test`:
```
$ rame - -t 2
0 | LOAD #1   ; R0 = 1
1 | WRITE     ; 1
2 | WRITE     ; 1
3 | STOP
4 |           ; <eof>
error: output mismatch
 computed: [1, 1]
 expected: [2]
```

Reading unitinialized memoty is undefined behavior:
```
rame -
0 | LOAD #30  ; R0 = 30
1 | LOAD @0   ; R0 = R30
2 |           ; <eof>
error: anon:2: "LOAD @0": reading uninitialized memory R30
```

## Optimizer

Left is original, right is optimized.

### `remove_nops`

```
0 | NOP          0 | JUMP 1
1 | JUMP 3       1 | WRITE
2 | NOP          2 | JUMP 1
3 | WRITE          |
4 | NOP            |
5 | NOP            |
6 | JUMP 3         |
```

### `combine_consts`

```
0 | ADD #0       0 | ADD #3
1 | SUB #-1      1 | DIV #6
2 | ADD #2         |
3 | MUL #1         |
4 | DIV #2         |
5 | DIV #3         |
```

```
0 | ADD #1       0 | ADD #6
1 | ADD #2       1 | ADD #4
2 | ADD #3       2 | JUMP 1
3 | ADD #4         |
4 | JUMP 3         |
```

### `follow_jumps`

```
0 | JUMZ 1       0 | JUMZ 3
1 | JUMP 2       1 | JUMP 3
2 | JUMP 3       2 | JUMP 3
3 | JUML 4       3 | JUML 4
```

### `remove_dead_code`

```
0 | LOAD #0      0 | LOAD #0
1 | JUMP 5       1 | JUMP 2
2 | ADD #1       2 | WRITE
3 | ADD 0        3 | JUMP 2
4 | DIV 2          |
5 | WRITE          |
6 | JUMP 5         |
```

## Model

Based on Jean-Pierre Zanotti's course, University of Toulon;  
<https://zanotti.univ-tln.fr/ALGO/II/MachineRAM.html>

### Operand set

|    Target    |        Choice         |                       Description                       |        Value         |
|:------------:|:---------------------:|:-------------------------------------------------------:|:--------------------:|
|  `<value>`   | `#n`<br/>`<register>` |              Constant<br/>Memory location               | `n`<br/>`<register>` |
| `<register>` |     `n`<br/>`@n`      | Direct memory addressing<br/>Indirect memory addressing | `R[n]`<br/>`R[R[n]]` |
| `<address>`  |     `n`<br/>`@n`      | Direct code addressing<br/>Indirect code addressing[^0] |    `n`<br/>`R[n]`    |

[^0]: Indirect jumps are not supported by the optimizer.

### Instruction set

|    Group    |                                                           Instruction                                                           | Action                                                                                                                                                                                                          |
|:-----------:|:-------------------------------------------------------------------------------------------------------------------------------:|:----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
|     I/O     |                                                       `READ`<br />`WRITE`                                                       | `ACC` ← `E[i++]`<br />`S[i++]` ← `ACC`                                                                                                                                                                          |
| Assignments |                                              `LOAD <value>`<br/>`STORE <register>`                                              | `ACC` ← `<value>`<br/>`<register>` ← `ACC`                                                                                                                                                                      |
| Arithmetics | `INC <register>`<br/>`DEC <register>`<br/>`ADD <value>`<br/>`SUB <value>`<br/>`MUL <value>`<br/>`DIV <value>`<br/>`MOD <value>` | `<register>` ← `<register> + 1`<br/>`<register>` ← `<register> - 1`<br/>`ACC` ← `ACC + <value>`<br/>`ACC` ← `ACC - <value>`<br/>`ACC` ← `ACC * <value>`<br/>`ACC` ← `ACC / <value>`<br/>`ACC` ← `ACC % <value>` |
|    Jumps    |                         `JUMP <address>`<br/>`JUMZ <address>`<br/>`JUML <address>`<br/>`JUMG <address>`                         | `IP` ← `<address>`<br/>`IF(ACC = 0)` `IP` ← `<address>`<br/>`IF(ACC < 0)` `IP` ← `<address>`<br/>`IF(ACC > 0)` `IP` ← `<address>`                                                                               |
|    Misc.    |                                                        `STOP`<br/>`NOP`                                                         | Terminates the process.<br/>Does nothing.                                                                                                                                                                       |
