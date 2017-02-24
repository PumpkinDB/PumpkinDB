# DEPTH

{% method -%}

Puts the depth (length) of the stack on the top of the stack

Input stack:

Output stack: `a`

{% common -%}

```
PunkinDB> "Hello, " "world!" DEPTH
2
```

{% endmethod %}

## Allocation

Allocates for the result of stack size calculation

## Errors

None

## Tests

```test
depth : "Hello, " "world!" DEPTH 2 EQUAL?.
```
