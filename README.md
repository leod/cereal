Cereal
======
A simple *cereal*isation library.

## How it works
- Primitives are serialisable (excluding slices/pointers/references/. This library is for handling raw data)
- Things solely made out of primitives are serialisable.
  - Use `#[derive(CerealData)]` with nightly
  - Or `impl_cereal_data!` with stable.
- Things that can be represented as primitive(s) or other serialisable(s) are also serialisable
  - `String`
  - `Vec<CerealData>`, `HashMap<CerealData,CerealData>`, and other collections as I think of them.
- Empty things are serialisable.
  - `()`
  - `PhantomData<T> where T: CerealData`

## How you use it

1\. Add to Cargo.toml
```toml
[dependencies]
# ...Other dependencies...
cereal = "*"
```
Or if you're on nightly, you may want to include `cereal_macros`
```toml
[dependencies]
# ...Other dependencies...
cereal = "*"
cereal_macros = "*"
```

2\. Implement `CerealData` for your types.  
Nightly:
```rust
#[derive(CerealData)]
struct MyAwesomeStruct {
    field1: u32,
    field2: String,
    field3: AnotherAwesomeStruct,
}
```
Stable:
```rust
struct MyAwesomeStruct {
    field1: u32,
    field2: String,
    field3: AnotherAwesomeStruct,
}
impl_cereal_data!(MyAwesomeStruct, field1, field2, field3);
```
Yes, you have to write out each field. It's annoying, but at least you have the comfort of knowing that your code will continue to work even if I forget to update the syntax extension for a compiler change.

3\. Create your type
```rust
let my_awesome_instance = MyAwesomeStruct {
    // ...
};
```

4\. Get a writer  
```rust
let a_writer: &mut Write = /* ... */;
```

5\. Write
```rust
my_awesome_instance.write(a_writer);
```

6\. Stop compiler screaming about an unused result
```rust
-- my_awesome_instance.write(a_writer);
++ my_awesome_instance.write(a_writer).unwrap();
```

7\. Get a reader
```rust
let a_reader: &mut Read = /* ... */;
```

8\. Read
```rust
let my_awesome_instance_2 = MyAwesomeStruct::read(a_reader).unwrap();
```

## What if it eats my laundry?
This library is by nature a binarvore (binary data eater), but feeding it unhealthy data could result in it turning into a laundrovore.
#### You have been warned
If you encounter any problems or missing socks, open up an issue on the issue tracker, or ping me on `#rust-gamedev` if I'm there (nick is `HeroesGrave`).
