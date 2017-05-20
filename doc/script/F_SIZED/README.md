# Fixed-sized floats

Signed floats, the size can be f32 or f64.

## Tests

```test
no_sign_f32 : 1.0f32 +1.0f32 EQUAL?.
no_sign_f64 : 1.0f64 +1.0f64 EQUAL?.
zero_f32  : -0.0f32 +0.0f32 EQUAL?.
zero_f64  : -0.0f32 +0.0f32 EQUAL?.
more_different_sign_f32 : +1.0f32 -1.0f32 GT?.
more_same_sign_f32 : -1.0f32 -2.0f32 GT?.
more_different_sign_f64 : +1.0f64 -1.0f64 GT?.
more_same_sign_f64 : -1.0f64 -2.0f64 GT?.
less_different_sign_f32 : -1.0f32 1.0f32 LT?.
less_same_sign_f32 : -2.0f32 -1.0f32 LT?.
less_different_sign_f64 : -1.0f64 1.0f64 LT?.
less_same_sign_f64 : -2.0f64 -1.0f64 LT?.
```