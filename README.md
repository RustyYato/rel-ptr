# rel-ptr

`rel-ptr` a library for relative pointers, which can be used to create
moveable self-referential types. This library was inspired by
Johnathan Blow's work on Jai, where he added relative pointers
as a primitive into Jai.

A relative pointer is a pointer that uses an offset and it's current location to
calculate where it points to.

## Features

### `no_std`

This crate is `no-std` compatible, simply add the feature `no_std` to move into `no_std` mode.

### nightly

with nightly you get the ability to use trait objects with relative pointers

## Example

take the memory segment below

`[.., 0x3a, 0x10, 0x02, 0xe4, 0x2b ..]`

where `0x3a` has the address `0xff304050` (32-bit system)
then `0x2b` has the address `0xff304054`.

if we have a 1-byte relative pointer (`RelPtr<_, i8>`)
at the address `0xff304052`, then that relative pointer points to
`0x2b` as well, this is because its address `0xff304052`, plus its
offset, `0x02` points to `0x2b`.

There are three interesting things
about this
1) it only took 1 byte to point to another value,
2) a relative pointer cannot access all memory, only memory near it
3) if both the relative pointer and the pointee move together,
   then the relative pointer will not be invalidated

The third point is what makes moveable self-referential structs possible

The type `RelPtr<T, I>` is a relative pointer. `T` is what it points to,
and `I` is what it uses to store its offset. In practice you can ignore `I`,
which is defaulted to `isize`, because that will cover all of your cases for using
relative pointers. But if you want to optimize the size of the pointer, you can use
any type that implements `Delta`. Some types from std that do so are:
`i8`, `i16`, `i32`, `i64`, `i128`, and `isize`. Note that the trade off is that as you
decrease the size of the offset, you decrease the range to which you can point to.
`isize` will cover at least half of addressable memory, so it should work unless you do
something really crazy. For self-referential structs use a type whose max value is atleast
as big as your struct. i.e. `std::mem::size_of::<T>() <= I::max_value()`.

Note on usized types: these are harder to get working 

## Self Referential Type Example

```rust
 struct SelfRef {
     value: (String, u32),
     ptr: RelPtr<String, i8>
 }

 impl SelfRef {
     pub fn new(s: String, i: u32) -> Self {
         let mut this = Self {
             value: (s, i),
             ptr: RelPtr::null()
         };
         
         this.ptr.set(&this.value.0).unwrap();
         
         this
     }

     pub fn fst(&self) -> &str {
         unsafe { self.ptr.as_ref_unchecked() }
     }

     pub fn snd(&self) -> u32 {
         self.value.1
     }
 }

 let s = SelfRef::new("Hello World".into(), 10);
 
 assert_eq!(s.fst(), "Hello World");
 assert_eq!(s.snd(), 10);
 
 let s = Box::new(s); // force a move, note: relative pointers even work on the heap
 
 assert_eq!(s.fst(), "Hello World");
 assert_eq!(s.snd(), 10);
```

This example is contrived, and only useful as an example.
In this example, we can see a few important parts to safe moveable self-referential types,
lets walk through them.

First, the definition of `SelfRef`, it contains a value and a relative pointer, the relative pointer that will point into the tuple inside of `SelfRef.value` to the `String`. There are no lifetimes involved because they would either make `SelfRef` immovable, or they could not be resolved correctly.

We see a pattern inside of `SelfRef::new`, first create the object, and use the sentinel `RelPtr::null()` and immediately afterwards assigning it a value using `RelPtr::set` and unwraping the result. This unwrapping is get quick feedback on whether or not the pointer was set, if it wasn't set then we can increase the size of the offset and resolve that.

Once the pointer is set, moving the struct is still safe because it is using a *relative* pointer, so it doesn't matter where it is, only it's offset from its pointee.
In `SelfRef::fst` we use `RelPtr::as_ref_unchecked` because it is impossible to invalidate the pointer. It is impossible because we cannot
set the relative pointer directly, and we cannot change the offsets of the fields of `SelfRef` after the relative pointer is set.