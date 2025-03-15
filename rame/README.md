# RAME

A [random-access machine](https://en.wikipedia.org/wiki/Random-access_machine) emulator and optimizer.
The library is tested on integers but should work with rational or complex numbers, matrices,
or any other type. The optimizer only requires the numbers to have additive and multiplicative
identities (i.e. `0` and `1`).

## Model

Based on Jean-Pierre Zanotti's course, Université de Toulon;  
<https://zanotti.univ-tln.fr/ALGO/II/MachineRAM.html>

### Operand set

|    Target    |        Choice         |                     Description                      |        Value         |
|:------------:|:---------------------:|:----------------------------------------------------:|:--------------------:|
|  `<value>`   | `#n`<br/>`<register>` |             Constant<br/>Memory location             | `n`<br/>`<register>` |
| `<register>` |     `n`<br/>`@n`      |          Direct memory<br/>Indirect memory           | `R[n]`<br/>`R[R[n]]` |
| `<address>`  |     `n`<br/>`@n`      |          Direct code<br/>Indirect code[^0]           |    `n`<br/>`R[n]`    |

[^0]: Indirect jumps are not supported by the optimizer.

### Instruction set

|    Group    |                                                           Instruction                                                           | Action                                                                                                                                                                                                          |
|:-----------:|:-------------------------------------------------------------------------------------------------------------------------------:|:----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
|     I/O     |                                                       `READ`<br />`WRITE`                                                       | `ACC` ← `E[i++]`<br />`S[i++]` ← `ACC`                                                                                                                                                                          |
| Assignments |                                              `LOAD <value>`<br/>`STORE <register>`                                              | `ACC` ← `<value>`<br/>`<register>` ← `ACC`                                                                                                                                                                      |
| Arithmetics | `INC <register>`<br/>`DEC <register>`<br/>`ADD <value>`<br/>`SUB <value>`<br/>`MUL <value>`<br/>`DIV <value>`<br/>`MOD <value>` | `<register>` ← `<register> + 1`<br/>`<register>` ← `<register> - 1`<br/>`ACC` ← `ACC + <value>`<br/>`ACC` ← `ACC - <value>`<br/>`ACC` ← `ACC * <value>`<br/>`ACC` ← `ACC / <value>`<br/>`ACC` ← `ACC % <value>` |
|    Jumps    |                         `JUMP <address>`<br/>`JUMZ <address>`<br/>`JUML <address>`<br/>`JUMG <address>`                         | `IP` ← `<address>`<br/>`IF(ACC = 0)` `IP` ← `<address>`<br/>`IF(ACC < 0)` `IP` ← `<address>`<br/>`IF(ACC > 0)` `IP` ← `<address>`                                                                               |
|    Misc.    |                                                        `STOP`<br/>`NOP`                                                         | Terminates the process.<br/>Does nothing.                                                                                                                                                                       |

## Runner

Reading unitinialized memory is undefined behavior:
```
$ rame-run -
0 | LOAD 1  
1 | 
error: anon:1: "LOAD 1": reading uninitialized memory R1
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
