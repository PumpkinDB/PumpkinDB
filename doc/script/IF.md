# IF

{% method -%}

Provides conditional flow control depending on a boolean value.

Input stack: `a [b]`

Output stack: maybe `b`

`IF` will push the result `[b]` to the stack if `a` is `1`.

`IF` evaluates the branch closure on the current stack.
{% common -%}

```
PumpkinDB> 0x01 [0x20] IF
0x20
PumpkinDB> 0x00 [0x20] IF
```

{% endmethod %}

## Allocation

None

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there are less than two items on the stack

[InvalidValue](./errors/InvalidValue.md) error if the value being checked for truth is not a boolean.

[Decoding error](./errors/DECODING.md) error if the code is undecodable.

## Tests

```test
works : 1 [2] IF 2 EQUAL?.
invalid_code : [1 1 IF] TRY UNWRAP 0x05 EQUAL?.
invalid_value : [5 [1] IF] TRY UNWRAP 0x03 EQUAL?.
requires_two_items_0 : [IF] TRY UNWRAP 0x04 EQUAL?.
requires_two_items_1 : [[] IF] TRY UNWRAP 0x04 EQUAL?.
```
