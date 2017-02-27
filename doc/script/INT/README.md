# INT

By convention, INTs are signed big integers. They are serialized as a sign
prefix (`0` for negative, `1` for positive and zero) followed by a variable
sized bigint serialization.

```test
zero : -0 +0 EQUAL?.
neg_zero : -1 +0 LT?.
neg_pos : -1 +1 LT?.
```