# GT?

{% method -%}

Compares two topmost items lexicographically.

Input stack: `a b`

Output stack: `a`

`GT?` will push `1` if `a` is strictly greater than `b`, `0` otherwise.

{% common -%}

```
PumpkinDB> 0x10 0x20 GT?
0
PumpkinDB> 0x20 0x10 GT?
1
```

{% endmethod %}

## Allocation

None

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there are less than two items on the stack

## Tests

```test
less : 0x10 0x20 GT? NOT.
greater : 0x20 0x10 GT?.
equal : 0x10 0x10 GT? NOT.
requires_two_items_0 : [GT?] TRY UNWRAP 0x04 EQUAL?.
requires_two_items_1 : [1 GT?] TRY UNWRAP 0x04 EQUAL?.

more_different_sign_usized : +1 -1 GT?.
more_same_sign_unsized : -1 -2 GT?.

more_different_sign_i8 : +1i8 -1i8 GT?.
more_same_sign_i8 : -1i8 -2i8 GT?.

more_different_sign_i16 : +1i16 -1i16 GT?.
more_same_sign_i16 : -1i16 -2i16 GT?.

more_different_sign_i32 : +1i32 -1i32 GT?.
more_same_sign_i32 : -1i32 -2i32 GT?.

more_different_sign_i64 : +1i64 -1i64 GT?.
more_same_sign_i64 : -1i64 -2i64 GT?.
```
