# Q\>

Pronounced "from the back of the queue"

{% method -%}

Pops a value from the back of the queue.  Used in conjunction with [>Q](TO_BQ.md), mostly
to pass values into and out of closures that are evaluated on separate stacks (although can be
sometimes used for other purposes as well)

Input stack: -

Output stack: `a`

{% common -%}

```
PumpkinDB> [1 >Q 2 >Q] EVAL Q>
2
```

{% endmethod %}

## Allocation

None

## Errors

[NoValue](./errors/NoValue.md) error if the queue is empty

## Tests

```test
works : 1 >Q 2 Q> 1 EQUAL?.
empty_stack : [Q>] TRY UNWRAP 0x0a EQUAL?.
```
