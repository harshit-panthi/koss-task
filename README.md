# Threadpool architecture
Partly based on go runtime's M:N scheduler and implements work stealing

# How the borrow checker and rust's type system are advantageous for safety

The ownership system and the borrow check together prevent use after frees and dangling pointers with the concept of lifetimes and
prevent data races by ensuring that no more that one mutable reference can exist at a given time.

Rust's rich type system allows it to better model data. For example, while most other languages have a concept of null pointers, rust avoids them with its sum types (enums).
There are many cases where the return value of funtions better model their behaviour with Results and Options, whereas other languages throw exceptions with no indication the the function can throw in its signature, or by setting some global error variable, checking which is optional, or by returning a null in case of pointers.
