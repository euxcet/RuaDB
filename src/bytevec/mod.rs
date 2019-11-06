//! bytevec: A Rust serialization library that uses byte vectors
//! ============================================================
//! 
//! bytevec takes advantage of Rust's concise and stable type system to
//! serialize data objects to a byte vector (`Vec<u8>`) and back.
//! 
//! What does it do?
//! ----------------
//! Rust has a very powerful type system with predictable sizes for most
//! types, starting with the primitive types, so it's fairly easy to convert
//! any type to a collection of bytes and convert it back. This library intends
//! to give the user the means of converting a given type instance to a byte vector
//! and store it or send it through the wire to another device, with the possibility
//! of getting the value back from the byte vector anytime using the library traits.
//! 
//! Of course, Rust isn't magical enough to implement the traits to serialize
//! the functions automatically, as every type has its quirks. This library
//! uses two traits to give a type the functionality it needs to do that: 
//! `ByteEncodable` and `ByteDecodable`.
//! 
//! ###The `ByteEncodable` trait
//! A type that implements this trait is able to use the `encode` method that 
//! yields a `Vec<u8>` byte sequence. Seems prone to failure right? Of course it is,
//! internally it uses `unsafe` blocks to extract the bytes of a given type, so 
//! it can be pretty unsafe. That's why it always checks for any possible error and
//! returns the vector wrapped around a `BVEncodeResult` instance. If everything
//! goes `Ok`, we will be able to get a byte vector value that represents the 
//! original data structure.
//! 
//! bytevec doesn't actually do a 1:1 conversion of the bytes of the original
//! type instance, as not every Rust type is stored on the stack. For any type
//! that wraps a heap stored value, it will give a representation of the 
//! underlying value.
//! 
//! bytevec implements `ByteEncodable` out of the box for the following types:
//! 
//! - The integral types: `u8`, `u16`, `u32`, `u64`, `i8`, `i16`, `i32`, `i64`
//! 
//! - The floating point types: `f32` and `f64`
//! 
//! - `char`, `str` and `String`
//! 
//! - [`Vec`](http://doc.rust-lang.org/stable/std/vec/struct.Vec.html)
//! 
//! - [`&[T]`](http://doc.rust-lang.org/stable/std/primitive.slice.html)
//! 
//! - [`HashMap`](http://doc.rust-lang.org/stable/std/collections/struct.HashMap.html)
//! 
//! - [`HashSet`](http://doc.rust-lang.org/stable/std/collections/struct.HashSet.html)
//! 
//! - Tuples with up to 12 elements
//! 
//! - Custom `struct`s
//! 
//! For collections and other structures, automatic implementation of bytevec
//! requires that all of its underlying elements implement the `ByteEncodable`
//! trait.
//! 
//! ###The bytevec serialization format
//! bytevec doesn't follow any particular serialization format. It follows simple
//! rules when translating some type value to bytes:
//! 
//! - For a primitive type such as the integral types, floating points
//! or char that have fixed size, it will just grab the bytes and put them 
//! on a `u8` buffer of the same length as the size of the type through 
//! [`std::mem::transmute`][1]. These types are converted to and from little endian on
//! serialization and deserialization respectively.
//! 
//! - String and str don't store their byte count, it's up to their container (if any)
//! to store the size of the byte buffer of the string.
//! 
//! - Complex data structures such as `struct`s, tuples and collections need to store
//! the sizes of their underlying data fields. These sizes are stored as values of a generic
//! integral type parameter `Size` that should be provided in every call of the methods of the
//! `ByteEncodable` and `ByteDecodable` traits. This type parameter is propagated to the
//! serialization and deserialization operations of the contained data fields. The type parameter
//! `Size` is constrained by the `BVSize` trait. Currently the types that implement this trait
//! are `u8`, `u16`, `u32` and `u64`. Users should select the type for the `Size` type parameter
//! according to the expected size of the byte buffer. If the expected size exceeds the 
//! 2<sup>32</sup> byte length limit of `u32`, use `u64` instead.
//! 
//! - For structures with defined fields such as a custom `struct` or a tuple,
//! it will store the size of each field on a sequence of `Size` values at the start
//! of the slice segment for the structure, followed by the actual bytes of 
//! the values of the fields.
//! 
//! - For any collection with variable length, it will first store the length
//! (in elements, not byte count) on a `Size` value, followed by the byte count
//! (yes, of `Size`) of each element, and then the actual values of the elements.
//! All of this done in order, order is important, the same order of serialization
//! is the order of deserialization.
//! 
//! - All serializable values can be nested, so any structure that implements 
//! `ByteEncodable` containing a `Vec`, `String`, or another structure that also implements
//! `ByteEncodable` will be serialized along all its fields.
//! 
//! ###The `ByteDecodable` trait
//! Given a byte vector retrieved from memory, a file, or maybe a TCP connection,
//! the user will be able to pass the vector to the `decode` method of
//! a type that implements the `ByteDecodable` trait. `decode` will do a few checks 
//! on the byte vector and if the required sizes matches, it will yield a type instance wrapped 
//! in a `BVDecodeResult`. If the size doesn't match, or if some other conversion problem 
//! arises, it will yield a `ByteVecError` detailing the failure.
//! 
//! Almost all of the out of the box implementations of `ByteEncodable` also
//! implement `ByteDecodable`, but some of them, particularly the slices and 
//! the tuple references don't make sense when deserialized, as they can't
//! point to the original data they were referencing. This is usually a problem
//! that requires some tweaking, but bytevec allows data conversion from byte
//! buffers that were originally referenced data to a new instance of an owned data type,
//! as long as the size requirements are the same. This way, slice data can
//! be assigned to a `Vec` instance for example, as long as they share the same 
//! type of the underlying elements.
//! 
//! The `ByteDecodable` trait also provides the `decode_max` method, which like `decode`, it
//! accepts the byte buffer to deserialize, but additionally, this method also accepts
//! a `limit` argument. This parameter is compared to the length of the `u8` buffer and
//! if the buffer length is greater than it, it will return a `BadSizeDecodeError`,
//! otherwise it will return the result of `decode` on the byte buffer.
//! 
//! ###Example: Serialization and deserialization of a slice
//! 
//! ```rust
//! # #[macro_use]
//! # extern crate bytevec;
//! #
//! # use bytevec::{ByteEncodable, ByteDecodable};
//! # fn main() {
//! let slice = &["Rust", "Is", "Awesome!"];
//! let bytes = slice.encode::<u32>().unwrap();
//! let vec = <Vec<String>>::decode::<u32>(&bytes).unwrap();
//! assert_eq!(vec, slice);
//! # }
//! ```
//! [1]: http://doc.rust-lang.org/stable/std/mem/fn.transmute.html

#[macro_use]
pub mod macros;
pub mod traits;
pub mod errors;
pub mod impls;

pub use traits::{ByteEncodable, ByteDecodable};
pub type BVEncodeResult<T> = Result<T, errors::ByteVecError>;
pub type BVDecodeResult<T> = Result<T, errors::ByteVecError>;
pub use impls::BVSize;