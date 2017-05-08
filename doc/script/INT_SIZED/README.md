# INT[size]

INTs are signed sized integers, the size can be i8, i16, i32 or i64.


```test
no_sign_i8 : 1i8 +1i8 EQUAL?.
no_sign_i16 : 1i16 +1i16 EQUAL?.
no_sign_i32 : 1i32 +1i32 EQUAL?.
no_sign_i64 : 1i64 +1i64 EQUAL?.
zero_i8  : -0i8 +0i8 EQUAL?.
zero_i16 : -0i16 +0i16 EQUAL?.
zero_i32 : -0i32 +0i32 EQUAL?.
zero_i64 : -0i64 +0i64 EQUAL?.

more_different_sign_i8 : +1i8 -1i8 GT?.
more_same_sign_i8 : -1i8 -2i8 GT?.

more_different_sign_i16 : +1i16 -1i16 GT?.
more_same_sign_i16 : -1i16 -2i16 GT?.

more_different_sign_i32 : +1i32 -1i32 GT?.
more_same_sign_i32 : -1i32 -2i32 GT?.

more_different_sign_i64 : +1i64 -1i64 GT?.
more_same_sign_i64 : -1i64 -2i64 GT?.

less_different_sign_i8 : -1i8 1i8 LT?.
less_same_sign_i8 : -2i8 -1i8 LT?.

less_different_sign_i16 : -1i16 1i16 LT?.
less_same_sign_i16 : -2i16 -1i16 LT?.

less_different_sign_i32 : -1i32 1i32 LT?.
less_same_sign_i32 : -2i32 -1i32 LT?.

less_different_sign_i64 : -1i64 1i64 LT?.
less_same_sign_i64 : -2i64 -1i64 LT?.
```
