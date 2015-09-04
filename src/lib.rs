
use std::any::Any;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::io::{self, Read, Write};
use std::ptr;

pub type CerealResult<T> = ::std::result::Result<T, CerealError>;

// NOTE: Should be unsafe but syntax extensions can't handle that yet.
// Must be implemented symmetrically (ie: returns same type and same number of bytes written as read)
pub trait CerealData: 'static {
    fn write(&self, &mut Write) -> CerealResult<()>;
    fn read(&mut Read) -> CerealResult<Self>;
}

#[derive(Debug)]
pub enum CerealError {
    Io(io::Error),
    Msg(String),
    Any(Box<Any>),
}

#[macro_export]
macro_rules! __priv_read_ignore {
    {
        $read:expr, $e:expr
    } => {
        $crate::CerealData::read($read)
    }
}

#[macro_export]
macro_rules! impl_cereal_data {
    {
        $Struct:ident
    } => {
        __priv_data_empty!($Struct, $Struct);
    };
    {
        $Struct:ident(), $($field:ident),+
    } => {
        impl $crate::CerealData for $Struct {
            fn write(&self, write: &mut ::std::io::Write) -> $crate::CerealResult<()> {
                match *self {
                    $Struct($(ref $field),+) => {
                        $(try!($crate::CerealData::write($field, write)));+
                    },
                }
                Ok(())
            }

            fn read(read: &mut ::std::io::Read) -> $crate::CerealResult<$Struct> {
                Ok($Struct(
                    $(
                        try!(__priv_read_ignore!(read, $field))
                    ),+
                ))
            }
        }
    };
    {
        $Struct:ident, $($field:ident),+
    } => {
        impl $crate::CerealData for $Struct {
            fn write(&self, write: &mut ::std::io::Write) -> $crate::CerealResult<()> {
                $(
                    try!($crate::CerealData::write(&self.$field, write))
                );+;
                Ok(())
            }

            fn read(read: &mut ::std::io::Read) -> $crate::CerealResult<$Struct> {
                Ok($Struct {
                    $(
                        $field: try!($crate::CerealData::read(read))
                    ),+
                })
            }
        }
    };
}

macro_rules! data_primitive {
    {
        $Type:ty, $N:expr
    } => {
        impl $crate::CerealData for $Type {
            fn write(&self, write: &mut ::std::io::Write) -> $crate::CerealResult<()> {
                let bytes: &[u8; $N] = unsafe {
                    ::std::mem::transmute(self)
                };
                write.write_all(bytes).map_err(|e| CerealError::Io(e))
            }

            fn read(read: &mut ::std::io::Read) -> $crate::CerealResult<$Type> {
                let mut r = [0u8; $N];
                let mut vec = Vec::new();
                if try!(read.take($N).read_to_end(&mut vec).map_err(|err| $crate::CerealError::Io(err))) != $N {
                    Err($crate::CerealError::Msg("Unexpectedly reached end of stream".to_string()))
                } else {
                    assert!(vec.len() == $N);
                    unsafe {
                        ptr::copy_nonoverlapping(vec.as_ptr(), r.as_mut_ptr(), $N);
                        Ok(::std::mem::transmute::<_, $Type>(r))
                    }
                }
            }
        }
    }
}

#[macro_export]
macro_rules! __priv_data_empty {
    {
        $Type:ty, $Expr:expr
    } => {
        impl $crate::CerealData for $Type {
            fn write(&self, _: &mut ::std::io::Write) -> $crate::CerealResult<()> { Ok(()) }
            fn read(_: &mut ::std::io::Read) -> $crate::CerealResult<$Type> { Ok($Expr) }
        }
    }
}

data_primitive!(u8, 1);
data_primitive!(i8, 1);
data_primitive!(u16, 2);
data_primitive!(i16, 2);
data_primitive!(u32, 4);
data_primitive!(i32, 4);
data_primitive!(f32, 4);
data_primitive!(u64, 8);
data_primitive!(i64, 8);
data_primitive!(f64, 8);

impl CerealData for bool { // We can't really write 1 bit by itself
    fn write(&self, write: &mut Write) -> CerealResult<()> {
        (if *self { 1u8 } else { 0u8 }).write(write)
    }

    fn read(read: &mut Read) -> CerealResult<bool> {
        <u8>::read(read).map(|u| u != 0)
    }
}

impl CerealData for usize { // For cross-platform consistency we use the highest possible number of bits
    fn write(&self, write: &mut Write) -> CerealResult<()> {
        (*self as u64).write(write)
    }

    fn read(read: &mut Read) -> CerealResult<usize> {
        <u64>::read(read).map(|u| u as usize)
    }
}

impl CerealData for isize { // For cross-platform consistency we use the highest possible number of bits
    fn write(&self, write: &mut Write) -> CerealResult<()> {
        (*self as i64).write(write)
    }

    fn read(read: &mut Read) -> CerealResult<isize> {
        <i64>::read(read).map(|i| i as isize)
    }
}

impl<T:CerealData> CerealData for Vec<T> {
    fn write(&self, write: &mut Write) -> CerealResult<()> {
        try!(self.len().write(write));
        for data in self {
            try!(data.write(write));
        }
        Ok(())
    }

    fn read(read: &mut Read) -> CerealResult<Vec<T>> {
        let len = try!(usize::read(read));
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(try!(CerealData::read(read)));
        }
        Ok(vec)
    }
}

impl<T:CerealData> CerealData for Option<T> {
    fn write(&self, write: &mut Write) -> CerealResult<()> {
        match self {
            &Some(ref x) => {
                try!(true.write(write));
                try!(x.write(write));
                Ok(())
            },
            &None => {
                try!(false.write(write));
                Ok(())
            }
        }
    }

    fn read(read: &mut Read) -> CerealResult<Option<T>> {
        if try!(bool::read(read)) {
            Ok(Some(try!(T::read(read))))
        } else {
            Ok(None)
        }
    }
}

impl<T:CerealData, U:CerealData> CerealData for (T, U) {
    fn write(&self, write: &mut Write) -> CerealResult<()> {
        try!(self.0.write(write));
        try!(self.1.write(write));
        Ok(())
    }

    fn read(read: &mut Read) -> CerealResult<(T, U)> {
        let t = try!(T::read(read));
        let u = try!(U::read(read));
        Ok((t, u))
    }
}

impl<T:CerealData> CerealData for [T; 2] {
    fn write(&self, write: &mut Write) -> CerealResult<()> {
        try!(self[0].write(write));
        try!(self[1].write(write));
        Ok(())
    }

    fn read(read: &mut Read) -> CerealResult<[T; 2]> {
        let a = try!(T::read(read));
        let b = try!(T::read(read));
        Ok([a, b])
    }
}

impl<T:CerealData> CerealData for Box<T> {
    fn write(&self, write: &mut Write) -> CerealResult<()> {
        CerealData::write(&**self, write)
    }

    fn read(read: &mut Read) -> CerealResult<Box<T>> {
        CerealData::read(read).map(|o| Box::new(o))
    }
}

impl CerealData for String {
    fn write(&self, write: &mut Write) -> CerealResult<()> {
        try!(self.len().write(write));
        write.write_all(self.as_bytes()).map_err(|err| CerealError::Io(err))
    }

    fn read(read: &mut Read) -> CerealResult<String> {
        Vec::read(read).and_then(|vec| String::from_utf8(vec).map_err(|err| CerealError::Any(Box::new(err) as Box<Any>)))
    }
}

impl<K:Eq+::std::hash::Hash+CerealData, V:CerealData> CerealData for HashMap<K, V> {
    fn write(&self, write: &mut Write) -> CerealResult<()> {
        try!(self.len().write(write));
        for (k, v) in self {
            try!(k.write(write));
            try!(v.write(write));
        }
        Ok(())
    }

    fn read(read: &mut Read) -> CerealResult<HashMap<K, V>> {
        let len = try!(usize::read(read));
        let mut map = HashMap::with_capacity(len);
        for _ in 0..len {
            map.insert(try!(CerealData::read(read)), try!(CerealData::read(read)));
        }
        Ok(map)
    }
}

impl<T: CerealData> CerealData for PhantomData<T> {
    fn write(&self, _: &mut Write) -> CerealResult<()> { Ok(()) }
    fn read(_: &mut Read) -> CerealResult<PhantomData<T>> { Ok(PhantomData) }
}

__priv_data_empty!((), ());
