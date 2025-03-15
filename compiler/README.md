# ARC

A pseudolanguage to abstract machine compiler.
Arc stands for Algo-Ram-Compiler.

Flex is used for lexical analysis, and Bison for syntactic analysis.

Bison's semantic action routines construct the abstract syntax tree, and perform
semantic analysis while creating the nodes.

The tree is then converted into RAM instructions with a `codegen(...)`,
which also performs some semantic analysis, e.g. checking that all functions exist
(i.e. linking) and that the main function exists.

## Installation

Requires GCC, GNU Make, Flex and Bison.
Just run `make`.

## Usage

```
./arc infile [-o outfile]
```

## File list

- `src/asa.[ch]`: the AST nodes
- `src/ram/codegen.[ch]`: AST to RAM codegen backend
- `src/ts.[ch]`: symbol tables (variables only)
- `src/parser.y`: the parser
- `src/lexer.lex`: the lexer

## Language reference

### Variables

All variables must be declared before their first use, and cannot be
be redeclared.

#### Scalars

A scalar `x` is declared as follows:
```
VAR x;
```

A value may be assigned during or after the scalar declaration:
```
VAR x := 0;
x := 12;
```

The `<-` operator can be used instead of `:=`.

Reading a scalar implicitly declares the variable if has not yet
been declared:
```
READ x;
```

You can also print an expression:
```
PRINT x + 1;
```

The scope of a variable is the entire function:
```
IF 1 == 1 THEN
    VAR x := 12;
FSI

DISPLAY x; # 12
```

#### Arrays

Static arrays can be declared as follows:
```
VAR t[5];
```

The size must be an integer (no expressions, even constants).  
Due to the fact that the grammar is not typed, some syntaxes are modified
for arrays:
```
# Reads five integers and stores them in t
READ[t];

# Displays the five integers contained in t
PRINT [t];
```

Arrays may also be indexed manually:
```
DISPLAY t[0];
READ t[1];
```

It is possible to declare an array during a `READ` with this syntax:
```
READ[3] s;
```

You can retrieve the size of an array using the `len()` method:
```
DISPLAY s.len(); # 3
```

And assign the values of an array directly:
```
s := { 1, 2, 3 };
VAR u := { 1, 2 };
```

The two sizes must naturally match, and are checked during
semantic analysis.

#### NoOps

Arrays of size zero produce NoOps, i.e. no instructions.  
NoOps are contagious, i.e. `NoOp + expr` = `NoOp`.

### Conditional structures

```
IF expr THEN
    ...
FI
```
```
IF expr THEN
    ...
ELSE
    ...
FI
```

```
WHILE expr DO
    ...
DONE
```

### Logical operators

- `NOT expr`
- `expr AND expr`
- `expr OR expr`
- `expr XOR expr`

Logical `AND` and `OR` are short-circuiting, i.e. `FALSE AND expr` and
`TRUE or expr` will not evaluate `expr` but will immediately evaluate to
resp. `FALSE` and `TRUE`.

### Functions

Functions are defined as follows:
```
FUNCTION g()
BEGIN
    RETURN;
END

FUNCTION f(x, y)
BEGIN
    RETURN x + y;
END
```

Not doing `RETURN` causes the program to `QUIT` at the end of the function.  
All functions return a value, either the specified expression or `0`.

Functions are usually invoked:
```
PRINT h();
```

Using an undeclared function will result in an error during semantic
analysis.

The main function is called `main` and has no parameters. Functions
may be defined in any order.
