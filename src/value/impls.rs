use crate::{
    std::fmt,
    stream,
    value::{
        self,
        Value,
    },
};

impl Value for () {
    fn stream<'s, 'v>(&'v self, mut stream: value::Stream<'s, 'v>) -> value::Result {
        stream.none()
    }
}

impl<T> Value for Option<T>
where
    T: Value,
{
    fn stream<'s, 'v>(&'v self, mut stream: value::Stream<'s, 'v>) -> value::Result {
        match self {
            Some(v) => v.stream(stream),
            None => stream.none(),
        }
    }
}

impl<T> Value for [T]
where
    T: Value,
{
    fn stream<'s, 'v>(&'v self, mut stream: value::Stream<'s, 'v>) -> value::Result {
        stream.seq_begin(Some(self.len()))?;

        for v in self {
            stream.seq_elem(v)?;
        }

        stream.seq_end()
    }
}

impl<T, U> Value for (T, U)
where
    T: Value,
    U: Value,
{
    fn stream<'s, 'v>(&'v self, mut stream: value::Stream<'s, 'v>) -> value::Result {
        stream.seq_begin(Some(2))?;

        stream.seq_elem(&self.0)?;
        stream.seq_elem(&self.1)?;

        stream.seq_end()
    }
}

impl Value for u8 {
    fn stream<'s, 'v>(&'v self, mut stream: value::Stream<'s, 'v>) -> value::Result {
        stream.u64(u64::from(*self))
    }
}

impl Value for u16 {
    fn stream<'s, 'v>(&'v self, mut stream: value::Stream<'s, 'v>) -> value::Result {
        stream.u64(u64::from(*self))
    }
}

impl Value for u32 {
    fn stream<'s, 'v>(&'v self, mut stream: value::Stream<'s, 'v>) -> value::Result {
        stream.u64(u64::from(*self))
    }
}

impl Value for u64 {
    fn stream<'s, 'v>(&'v self, mut stream: value::Stream<'s, 'v>) -> value::Result {
        stream.u64(*self)
    }
}

impl Value for i8 {
    fn stream<'s, 'v>(&'v self, mut stream: value::Stream<'s, 'v>) -> value::Result {
        stream.i64(i64::from(*self))
    }
}

impl Value for i16 {
    fn stream<'s, 'v>(&'v self, mut stream: value::Stream<'s, 'v>) -> value::Result {
        stream.i64(i64::from(*self))
    }
}

impl Value for i32 {
    fn stream<'s, 'v>(&'v self, mut stream: value::Stream<'s, 'v>) -> value::Result {
        stream.i64(i64::from(*self))
    }
}

impl Value for i64 {
    fn stream<'s, 'v>(&'v self, mut stream: value::Stream<'s, 'v>) -> value::Result {
        stream.i64(*self)
    }
}

impl Value for u128 {
    fn stream<'s, 'v>(&'v self, mut stream: value::Stream<'s, 'v>) -> value::Result {
        stream.u128(*self)
    }
}

impl Value for i128 {
    fn stream<'s, 'v>(&'v self, mut stream: value::Stream<'s, 'v>) -> value::Result {
        stream.i128(*self)
    }
}

impl Value for f32 {
    fn stream<'s, 'v>(&'v self, mut stream: value::Stream<'s, 'v>) -> value::Result {
        stream.f64(f64::from(*self))
    }
}

impl Value for f64 {
    fn stream<'s, 'v>(&'v self, mut stream: value::Stream<'s, 'v>) -> value::Result {
        stream.f64(*self)
    }
}

impl Value for bool {
    fn stream<'s, 'v>(&'v self, mut stream: value::Stream<'s, 'v>) -> value::Result {
        stream.bool(*self)
    }
}

impl Value for char {
    fn stream<'s, 'v>(&'v self, mut stream: value::Stream<'s, 'v>) -> value::Result {
        stream.char(*self)
    }
}

impl Value for str {
    fn stream<'s, 'v>(&'v self, mut stream: value::Stream<'s, 'v>) -> value::Result {
        stream.str(self)
    }
}

impl<'a> Value for fmt::Arguments<'a> {
    fn stream<'s, 'v>(&'v self, mut stream: value::Stream<'s, 'v>) -> value::Result {
        stream.debug(self)
    }
}

impl<'a> Value for stream::Arguments<'a> {
    fn stream<'s, 'v>(&'v self, mut stream: value::Stream<'s, 'v>) -> value::Result {
        stream.debug(self)
    }
}

impl<'a> Value for stream::Source<'a> {
    fn stream<'s, 'v>(&'v self, mut stream: value::Stream<'s, 'v>) -> value::Result {
        #[cfg(feature = "std")]
        {
            stream.error(self.get())
        }

        #[cfg(not(feature = "std"))]
        {
            stream.none()
        }
    }
}

#[cfg(feature = "alloc")]
mod alloc_support {
    use super::*;

    use crate::std::{
        boxed::Box,
        collections::BTreeMap,
        rc::Rc,
        string::String,
        vec::Vec,
    };

    impl<T: ?Sized> Value for Box<T>
    where
        T: Value,
    {
        fn stream<'s, 'v>(&'v self, stream: value::Stream<'s, 'v>) -> value::Result {
            (**self).stream(stream)
        }
    }

    impl<T: ?Sized> Value for Rc<T>
    where
        T: Value,
    {
        fn stream<'s, 'v>(&'v self, stream: value::Stream<'s, 'v>) -> value::Result {
            (**self).stream(stream)
        }
    }

    impl Value for String {
        fn stream<'s, 'v>(&'v self, mut stream: value::Stream<'s, 'v>) -> value::Result {
            stream.str(&*self)
        }
    }

    impl<T> Value for Vec<T>
    where
        T: Value,
    {
        fn stream<'s, 'v>(&'v self, stream: value::Stream<'s, 'v>) -> value::Result {
            self.as_slice().stream(stream)
        }
    }

    impl<K, V> Value for BTreeMap<K, V>
    where
        K: Eq + Value,
        V: Value,
    {
        fn stream<'s, 'v>(&'v self, mut stream: value::Stream<'s, 'v>) -> value::Result {
            stream.map_begin(Some(self.len()))?;

            for (k, v) in self {
                stream.map_key(k)?;
                stream.map_value(v)?;
            }

            stream.map_end()
        }
    }
}

#[cfg(feature = "std")]
mod std_support {
    use super::*;

    use crate::std::{
        collections::HashMap,
        error,
        hash::{
            BuildHasher,
            Hash,
        },
        sync::Arc,
    };

    impl Value for dyn error::Error + 'static {
        fn stream<'s, 'v>(&'v self, mut stream: value::Stream<'s, 'v>) -> value::Result {
            stream.error(self)
        }
    }

    impl<T: ?Sized> Value for Arc<T>
    where
        T: Value,
    {
        fn stream<'s, 'v>(&'v self, stream: value::Stream<'s, 'v>) -> value::Result {
            (**self).stream(stream)
        }
    }

    impl<K, V, H> Value for HashMap<K, V, H>
    where
        K: Hash + Eq + Value,
        V: Value,
        H: BuildHasher,
    {
        fn stream<'s, 'v>(&'v self, mut stream: value::Stream<'s, 'v>) -> value::Result {
            stream.map_begin(Some(self.len()))?;

            for (k, v) in self {
                stream.map_key(k)?;
                stream.map_value(v)?;
            }

            stream.map_end()
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "alloc")]
    mod alloc_support {
        use crate::{
            std::{
                boxed::Box,
                collections::BTreeMap,
                rc::Rc,
                string::String,
                vec::Vec,
            },
            test::{
                self,
                Token,
            },
        };

        #[cfg(target_arch = "wasm32")]
        use wasm_bindgen_test::*;

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
        fn stream_unsigned() {
            assert_eq!(vec![Token::Unsigned(1)], test::tokens(&1u8));

            assert_eq!(vec![Token::Unsigned(1)], test::tokens(&1u16));

            assert_eq!(vec![Token::Unsigned(1)], test::tokens(&1u32));

            assert_eq!(vec![Token::Unsigned(1)], test::tokens(&1u64));

            assert_eq!(vec![Token::BigUnsigned(1)], test::tokens(&1u128));
        }

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
        fn stream_signed() {
            assert_eq!(vec![Token::Signed(1)], test::tokens(&1i8));

            assert_eq!(vec![Token::Signed(1)], test::tokens(&1i16));

            assert_eq!(vec![Token::Signed(1)], test::tokens(&1i32));

            assert_eq!(vec![Token::Signed(1)], test::tokens(&1i64));

            assert_eq!(vec![Token::BigSigned(1)], test::tokens(&1i128));
        }

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
        fn stream_float() {
            assert_eq!(vec![Token::Float(1.0)], test::tokens(&1f32));

            assert_eq!(vec![Token::Float(1.0)], test::tokens(&1f64));
        }

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
        fn stream_bool() {
            assert_eq!(vec![Token::Bool(false)], test::tokens(&false));
        }

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
        fn stream_str() {
            assert_eq!(
                vec![Token::Str("a string".into())],
                test::tokens(&"a string")
            );

            assert_eq!(
                vec![Token::Str("a string".into())],
                test::tokens(&String::from("a string"))
            );

            assert_eq!(
                vec![Token::Str("a format 1".into())],
                test::tokens(&format_args!("a format {}", 1))
            );

            assert_eq!(vec![Token::Char('a')], test::tokens(&'a'));
        }

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
        fn stream_option() {
            assert_eq!(vec![Token::None], test::tokens(&Option::None::<i32>));

            assert_eq!(vec![Token::Signed(1)], test::tokens(&Some(1)));
        }

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
        fn stream_vec() {
            let v = test::tokens(&[] as &[i32]);
            assert_eq!(vec![Token::SeqBegin(Some(0)), Token::SeqEnd], v);

            let v = test::tokens(&[1, 2, 3] as &[i32]);
            assert_eq!(
                vec![
                    Token::SeqBegin(Some(3)),
                    Token::Signed(1),
                    Token::Signed(2),
                    Token::Signed(3),
                    Token::SeqEnd,
                ],
                v
            );

            let v = test::tokens(&Vec::<i32>::new());
            assert_eq!(vec![Token::SeqBegin(Some(0)), Token::SeqEnd], v);

            let v = test::tokens(&vec![1, 2, 3]);
            assert_eq!(
                vec![
                    Token::SeqBegin(Some(3)),
                    Token::Signed(1),
                    Token::Signed(2),
                    Token::Signed(3),
                    Token::SeqEnd,
                ],
                v
            );
        }

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
        fn stream_map() {
            let v = test::tokens(&BTreeMap::<i32, i32>::new());
            assert_eq!(vec![Token::MapBegin(Some(0)), Token::MapEnd], v);

            let v = test::tokens(&{
                let mut map = BTreeMap::new();
                map.insert(1, 11);
                map.insert(2, 22);
                map
            });
            assert_eq!(
                vec![
                    Token::MapBegin(Some(2)),
                    Token::Signed(1),
                    Token::Signed(11),
                    Token::Signed(2),
                    Token::Signed(22),
                    Token::MapEnd,
                ],
                v
            );
        }

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
        fn stream_box() {
            assert_eq!(vec![Token::Signed(1)], test::tokens(&Box::new(1i8)));
        }

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
        fn stream_rc() {
            assert_eq!(vec![Token::Signed(1)], test::tokens(&Rc::new(1i8)));
        }
    }

    #[cfg(feature = "std")]
    mod std_support {
        use crate::{
            std::{
                collections::HashMap,
                error,
                io,
                sync::Arc,
            },
            stream::Source,
            test::{
                self,
                Token,
            },
            value::{
                self,
                Value,
            },
        };

        #[cfg(target_arch = "wasm32")]
        use wasm_bindgen_test::*;

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
        fn stream_error() {
            let err = io::Error::from(io::ErrorKind::Other);
            assert_eq!(
                vec![Token::Error(test::Source::new(&err))],
                test::tokens(&Source::new(&err))
            );
        }

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
        fn stream_inner_error() {
            struct MyError;

            impl Value for MyError {
                fn stream<'s, 'v>(&'v self, mut stream: value::Stream<'s, 'v>) -> value::Result {
                    stream.owned().error(&io::Error::from(io::ErrorKind::Other))
                }
            }

            assert_eq!(
                vec![Token::Error(test::Source::new(&io::Error::from(
                    io::ErrorKind::Other
                )))],
                test::tokens(&MyError)
            );
        }

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
        fn stream_dyn_error() {
            let err: &(dyn error::Error + 'static) = &io::Error::from(io::ErrorKind::Other);
            assert_eq!(
                vec![Token::Error(test::Source::new(err))],
                test::tokens(&err)
            );
        }

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
        fn stream_map() {
            let v = test::tokens(&HashMap::<i32, i32>::new());
            assert_eq!(vec![Token::MapBegin(Some(0)), Token::MapEnd], v);
        }

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
        fn stream_rc() {
            assert_eq!(vec![Token::Signed(1)], test::tokens(&Arc::new(1i8)));
        }
    }
}
