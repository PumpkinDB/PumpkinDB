# \>R

Pronounced "to R"

{% method -%}

Pushes a value onto the return stack. Used in conjunction with [R>](FROM_R.md), mostly
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

## Tests

```test
works : 1 >R 2 R> 1 EQUAL?.
```
