# PAD

{% method -%}

Pads a binary with a number of bytes on left.

Input stack: `a size byte`

Output stack: `b`

`PAD` takes a and pads it with up to `size` bytes of `byte` on the left, up to 1024
bytes. This is an extremely important tool in building comparable collections over variable-length
values (such as bigintegers, for example)

{% common -%}

```
PumpkinDB> 0x01 4 0 PAD
0x00000001
```

{% endmethod %}

## Allocation

Allocates for a result of padding

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there are less than three items on the stack

[InvalidValue](./errors/InvalidValue.md) error if `byte` is larger or smaller than one byte

[InvalidValue](./errors/InvalidValue.md) error if `size` is larger than 1024.

[InvalidValue](./errors/InvalidValue.md) error if `size` is lesser than the length of `a`.

## Tests

```test
pad : 0x01 4 0 PAD 0x00000001 EQUAL?.
pad_1 : 0x01 4 0xff PAD 0xffffff01 EQUAL?.
requires_three_items_0 : [PAD] TRY UNWRAP 0x04 EQUAL?.
requires_three_items_1 : [1 PAD] TRY UNWRAP 0x04 EQUAL?.
requires_three_items_2 : [1 1 PAD] TRY UNWRAP 0x04 EQUAL?.
invalid_value : [0x01 4 "test" PAD] TRY UNWRAP 0x03 EQUAL?.
too_big : [0x01 1025 0 PAD] TRY UNWRAP 0x03 EQUAL?.
too_small : [0x0102 1 0 PAD] TRY UNWRAP 0x03 EQUAL?.
```
