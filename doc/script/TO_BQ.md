# \>Q

Pronounced "to the back of the queue"

{% method -%}

Pushes a value to the back of the queue. Used in conjunction with [Q>](FROM_BQ.md), mostly
to pass values into and out of closures that are evaluated on separate stacks (although can be
sometimes used for other purposes as well)

Input stack: `a`

Output stack: -

{% common -%}

```
PumpkinDB> [1 >Q 2 >Q] EVAL Q<
1
```

{% endmethod %}

## Allocation

None

## Errors

## Tests

```test
works : 1 >Q 2 Q> 1 EQUAL?.
```
