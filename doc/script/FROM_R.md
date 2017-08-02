# R\>

Pronounced "from R"

{% method -%}

Pops a value from the return stack. Used in conjunction with [>R](TO_R.md), mostly
to pass values into and out of evaluated closures (although can be sometimes used for other purposes
as well)

Input stack: `a`

Output stack: -

{% common -%}

```
PumpkinDB> [1 >R] EVAL R>
1
```

{% endmethod %}

## Allocation

None

## Errors

[EmptyStack](./errors/EmptyStack.md) error if the return stack is empty

## Tests

```test
works : 1 >R 2 R> 1 EQUAL?.
empty_stack : [R>] TRY UNWRAP 0x04 EQUAL?.
```
