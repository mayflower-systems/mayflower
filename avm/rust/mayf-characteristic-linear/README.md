# mayf-characteristic-linear

`LinearParams<N>` stores the positive slope of the neutral characteristic
`f(x) = slope * x`. Pass a `Pos<N>` to `new`, or use `try_new` to validate a
raw fixed-point value. `mayf-characteristic-linear-driver` evaluates and
integrates the line from these parameters.
