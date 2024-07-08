# expression evaluation

## numerical expression evaluation

input any mathematical expression

```
cargo run -- numerical "{formula here}"
```

example:

```
cargo run -- numerical "1 + 1"
```

### list of supported operators

Addition: +

Subtraction: -

Multiplication: \*

Division: /

Power: ^

## logical expression evaluation

input any predicate logic formula

```
cargo run -- logical "{formula here}"
```

example:

```
cargo run -- logical "T & F"
```

### list of supproted operators

And: &

Or: |

Implies: >

Converse: <

Equivalence: =
