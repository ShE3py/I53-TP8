A pseudolanguage to abstract machine optimizing compiler;
```
$ cargo run --bin rame-run -- -cO - 20
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
    PRINT fibo(n);
END
^D
```
```
Output = [6765]
```

Further reading:
- [compiler/README.md](compiler/README.md) for the language reference;
- [rame/README.md](rame/README.md) for the abstract machine model and optimizer;
- [driver/README.md](driver/README.md) for installing and running this repository.
