# \<Q

Pronounced "to the front of the queue"

{% method -%}

Pushes a value to the front of the queue. Used in conjunction with [Q<](FROM_FQ.md), mostly
to pass values into and out of closures that are evaluated on separate stacks (although can be
sometimes used for other purposes as well)

Input stack: `a`

Output stack: -

{% common -%}

```
PumpkinDB> [2 <Q 1 <Q] EVAL Q<
1
```

{% endmethod %}

## Allocation

None

## Errors

## Tests

```test
works : 1 <Q 2 <Q Q< 2 EQUAL?.
```
