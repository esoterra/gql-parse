# GQL Parse

This is an experimental and very very early WIP parser for the GQL query language specified by [ISO/IEC 39075:2024](https://www.iso.org/obp/ui/#iso:std:iso-iec:39075:ed-1:v1:en).

## Goals

* Verified ISO 39075 Conformance
  * This is a long way away.
  * It will require an extensive test suite.
* Memory Safety
  * The entire library will be written in safe Rust.
* Speed
  * The token stream and ast are stored in bump-allocated memory. This makes allocation fast, memory as contiguous as possible, and deallocation nearly instantaneous.  
* Ergonomics
  * There are no `Box` or `Rc` smart pointers. There is no index-indirection using arenas. Everything uses plain references (e.g. `&'q ast::Path<'q, 'src>`) which compare and debug easily.
  * There are lifetime parameters to keep track of but hopefully this isn't too difficult for users.