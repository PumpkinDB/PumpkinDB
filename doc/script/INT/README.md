# INT

By convention, INTs are signed big integers. They are serialized as a byte-long
sign prefix (`0` for negative, `1` for positive and zero) followed by a variable
sized bigint serialization. They are lexicographically sorted.

```test
zero : -0 +0 EQUAL?.
neg_zero : -1 +0 LT?.
neg_pos : -1 +1 LT?.
more_different_sign_usized : +1 -1 GT?.
more_same_sign_unsized : -1 -2 GT?.
more_different_sign_usized : -1 +1 LT?.
more_same_sign_unsized : -2 -1 LT?.
```

