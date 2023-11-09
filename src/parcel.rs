use std::{
    ffi::{c_char, c_int, c_void},
    mem::MaybeUninit,
    os::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, OwnedFd},
    ptr::{null, null_mut},
};

use crate::{
    status::Status,
    sys::{
        binder_status_t, AIBinder, AParcel, AParcel_delete, AParcel_getDataPosition, AParcel_readBool, AParcel_readBoolArray,
        AParcel_readByte, AParcel_readByteArray, AParcel_readChar, AParcel_readCharArray, AParcel_readDouble,
        AParcel_readDoubleArray, AParcel_readFloat, AParcel_readFloatArray, AParcel_readInt32, AParcel_readInt32Array,
        AParcel_readInt64, AParcel_readInt64Array, AParcel_readParcelFileDescriptor, AParcel_readParcelableArray,
        AParcel_readStatusHeader, AParcel_readString, AParcel_readStringArray, AParcel_readStrongBinder, AParcel_readUint32,
        AParcel_readUint32Array, AParcel_readUint64, AParcel_readUint64Array, AParcel_setDataPosition, AParcel_writeBool,
        AParcel_writeBoolArray, AParcel_writeByte, AParcel_writeByteArray, AParcel_writeChar, AParcel_writeCharArray,
        AParcel_writeDouble, AParcel_writeDoubleArray, AParcel_writeFloat, AParcel_writeFloatArray, AParcel_writeInt32,
        AParcel_writeInt32Array, AParcel_writeInt64, AParcel_writeInt64Array, AParcel_writeParcelFileDescriptor,
        AParcel_writeParcelableArray, AParcel_writeStatusHeader, AParcel_writeString, AParcel_writeStringArray,
        AParcel_writeStrongBinder, AParcel_writeUint32, AParcel_writeUint32Array, AParcel_writeUint64, AParcel_writeUint64Array,
        AStatus,
    },
    Code, IBinder,
};

pub trait Read: Sized {
    fn read(parcel: &Parcel) -> Result<Self, Status>;
}

pub trait Write {
    fn write(&self, parcel: &mut Parcel) -> Result<(), Status>;
}

enum RawParcel {
    Owned(*mut AParcel),
    Borrowed(*mut AParcel),
}

pub struct Parcel(RawParcel);

impl Drop for Parcel {
    fn drop(&mut self) {
        if let RawParcel::Owned(ptr) = self.0 {
            unsafe { AParcel_delete(ptr) }
        }
    }
}

impl Parcel {
    pub(crate) fn from_raw(raw: *mut AParcel) -> Self {
        Parcel(RawParcel::Owned(raw))
    }

    pub(crate) fn from_borrow_raw(raw: *mut AParcel) -> Self {
        Parcel(RawParcel::Borrowed(raw))
    }

    pub(crate) fn as_raw(&self) -> *mut AParcel {
        match self.0 {
            RawParcel::Owned(ptr) => ptr,
            RawParcel::Borrowed(ptr) => ptr,
        }
    }

    #[cfg(all(feature = "api-30", feature = "jni"))]
    pub unsafe fn from_java(env: *mut jni_sys::JNIEnv, obj: jni_sys::jobject) -> Option<Parcel> {
        unsafe {
            extern "C" {
                fn AParcel_fromJavaParcel(env: *mut jni_sys::JNIEnv, obj: jni_sys::jobject) -> *mut AParcel;
            }

            if obj.is_null() {
                None
            } else {
                Some(Parcel::from_raw(AParcel_fromJavaParcel(env, obj)))
            }
        }
    }

    #[cfg(feature = "api-31")]
    pub fn new() -> Parcel {
        unsafe { Parcel::from_raw(crate::sys::AParcel_create()) }
    }

    #[cfg(feature = "api-31")]
    pub fn reset(&mut self) {
        unsafe {
            crate::sys::AParcel_reset(self.as_raw());
        }
    }

    #[cfg(feature = "api-31")]
    pub fn get_data_size(&self) -> u32 {
        unsafe { crate::sys::AParcel_getDataSize(self.as_raw()) as u32 }
    }

    pub fn get_data_position(&self) -> u32 {
        unsafe { AParcel_getDataPosition(self.as_raw()) as u32 }
    }

    pub fn set_data_position(&mut self, pos: u32) -> Result<(), Status> {
        unsafe { Status::from_raw_status_code(AParcel_setDataPosition(self.as_raw(), pos as i32)).err(|| ()) }
    }

    #[cfg(feature = "api-31")]
    pub fn append_from(&mut self, other: &Parcel, offset: u32, length: u32) -> Result<(), Status> {
        unsafe {
            Status::from_raw_status_code(crate::sys::AParcel_appendFrom(
                other.as_raw(),
                self.as_raw(),
                offset as i32,
                length as i32,
            ))
            .err(|| ())
        }
    }

    #[cfg(feature = "api-33")]
    pub fn marshal(&self, buffer: &mut [u8], offset: usize) -> Result<(), Status> {
        unsafe {
            Status::from_raw_status_code(crate::sys::AParcel_marshal(
                self.as_raw(),
                buffer.as_mut_ptr(),
                offset,
                buffer.len(),
            ))
            .err(|| ())
        }
    }

    #[cfg(feature = "api-33")]
    pub fn unmarshal(&self, buffer: &[u8]) -> Result<(), Status> {
        unsafe {
            Status::from_raw_status_code(crate::sys::AParcel_unmarshal(self.as_raw(), buffer.as_ptr(), buffer.len())).err(|| ())
        }
    }

    pub fn read<T: Read>(&self) -> Result<T, Status> {
        T::read(self)
    }

    pub fn write<T: Write>(&mut self, value: &T) -> Result<(), Status> {
        value.write(self)
    }

    pub fn read_array<T: Read>(&self) -> Result<Option<Vec<T>>, Status> {
        unsafe {
            let mut ret: Option<Vec<T>> = None;

            unsafe extern "C" fn reader<T: Read>(parcel: *const AParcel, data: *mut c_void, index: usize) -> binder_status_t {
                let data = &mut *data.cast::<Option<Vec<T>>>();

                let parcel = Parcel::from_borrow_raw(parcel.cast_mut());

                match parcel.read::<T>() {
                    Ok(v) => {
                        std::mem::forget(std::mem::replace(&mut data.as_mut().unwrap()[index], v));

                        Code::Ok.as_raw()
                    }
                    Err(err) => err.get_code().as_raw(),
                }
            }

            Status::from_raw_status_code(AParcel_readParcelableArray(
                self.as_raw(),
                (&mut ret as *mut Option<Vec<T>>).cast(),
                Some(typed_nullable_array_allocator::<T>),
                Some(reader::<T>),
            ))
            .err(|| ret)
        }
    }

    pub fn write_array<T: Write>(&mut self, value: Option<&[T]>) -> Result<(), Status> {
        unsafe {
            unsafe extern "C" fn setter<T: Write>(parcel: *mut AParcel, data: *const c_void, index: usize) -> binder_status_t {
                let mut parcel = Parcel::from_borrow_raw(parcel);

                match (*(data.cast::<T>().offset(index as isize))).write(&mut parcel) {
                    Ok(_) => Code::Ok.as_raw(),
                    Err(err) => err.get_code().as_raw(),
                }
            }

            let (ptr, len) = match value {
                None => (null(), -1),
                Some(v) => (v.as_ptr(), v.len() as i32),
            };

            Status::from_raw_status_code(AParcel_writeParcelableArray(
                self.as_raw(),
                ptr.cast(),
                len,
                Some(setter::<T>),
            ))
            .err(|| ())
        }
    }
}

fn read_basic_type<T: Sized>(
    ptr: *const AParcel,
    read: unsafe extern "C" fn(*const AParcel, *mut T) -> binder_status_t,
) -> Result<T, Status> {
    unsafe {
        let mut value = MaybeUninit::uninit();

        Status::from_raw_status_code(read(ptr, value.as_mut_ptr())).err(|| value.assume_init())
    }
}

fn write_basic_type<T: Copy>(
    ptr: *mut AParcel,
    value: T,
    write: unsafe extern "C" fn(*mut AParcel, T) -> binder_status_t,
) -> Result<(), Status> {
    unsafe { Status::from_raw_status_code(write(ptr, value)).err(|| ()) }
}

fn read_basic_type_array<T>(
    ptr: *const AParcel,
    read: unsafe extern "C" fn(
        *const AParcel,
        *mut c_void,
        Option<unsafe extern "C" fn(*mut c_void, i32, *mut *mut T) -> bool>,
    ) -> binder_status_t,
) -> Result<Option<Vec<T>>, Status> {
    unsafe {
        let mut ret: Option<Vec<T>> = None;

        unsafe extern "C" fn allocator<T>(array_data: *mut c_void, length: i32, out_buffer: *mut *mut T) -> bool {
            let data: &mut Option<Vec<T>> = &mut *array_data.cast::<Option<Vec<T>>>();

            if length < 0 {
                *data = None;
            } else {
                let mut array = Vec::with_capacity(length as usize);
                array.set_len(length as usize);
                *out_buffer = array.as_mut_ptr();
                *data = Some(array);
            }

            true
        }

        Status::from_raw_status_code(read(ptr, ((&mut ret) as *mut Option<Vec<T>>).cast(), Some(allocator::<T>))).err(|| ret)
    }
}

fn write_basic_type_array<T>(
    ptr: *mut AParcel,
    data: Option<&[T]>,
    write: unsafe extern "C" fn(*mut AParcel, *const T, i32) -> binder_status_t,
) -> Result<(), Status> {
    unsafe {
        Status::from_raw_status_code(match data {
            None => write(ptr, null(), -1),
            Some(array) => write(ptr, array.as_ptr(), array.len() as i32),
        })
        .err(|| ())
    }
}

macro_rules! impls_for_basic_type {
    ($typ:ty, $write:ident, $read:ident) => {
        impl Read for $typ {
            fn read(parcel: &Parcel) -> Result<Self, Status> {
                read_basic_type(parcel.as_raw(), $read)
            }
        }

        impl Write for $typ {
            fn write(&self, parcel: &mut Parcel) -> Result<(), Status> {
                write_basic_type(parcel.as_raw(), *self, $write)
            }
        }
    };
}

impls_for_basic_type!(bool, AParcel_writeBool, AParcel_readBool);
impls_for_basic_type!(i8, AParcel_writeByte, AParcel_readByte);
impls_for_basic_type!(u16, AParcel_writeChar, AParcel_readChar);
impls_for_basic_type!(f64, AParcel_writeDouble, AParcel_readDouble);
impls_for_basic_type!(f32, AParcel_writeFloat, AParcel_readFloat);
impls_for_basic_type!(i32, AParcel_writeInt32, AParcel_readInt32);
impls_for_basic_type!(i64, AParcel_writeInt64, AParcel_readInt64);
impls_for_basic_type!(u32, AParcel_writeUint32, AParcel_readUint32);
impls_for_basic_type!(u64, AParcel_writeUint64, AParcel_readUint64);

macro_rules! impls_for_basic_type_array {
    ($typ:ty, $write_array:ident, $read_array:ident) => {
        impl Read for Option<Vec<$typ>> {
            fn read(parcel: &Parcel) -> Result<Self, Status> {
                read_basic_type_array(parcel.as_raw(), $read_array)
            }
        }

        impl Write for Option<Vec<$typ>> {
            fn write(&self, parcel: &mut Parcel) -> Result<(), Status> {
                write_basic_type_array(parcel.as_raw(), self.as_deref(), $write_array)
            }
        }

        impl Write for Option<&[$typ]> {
            fn write(&self, parcel: &mut Parcel) -> Result<(), Status> {
                write_basic_type_array(parcel.as_raw(), self.as_deref(), $write_array)
            }
        }
    };
}

impls_for_basic_type_array!(i8, AParcel_writeByteArray, AParcel_readByteArray);
impls_for_basic_type_array!(u16, AParcel_writeCharArray, AParcel_readCharArray);
impls_for_basic_type_array!(f64, AParcel_writeDoubleArray, AParcel_readDoubleArray);
impls_for_basic_type_array!(f32, AParcel_writeFloatArray, AParcel_readFloatArray);
impls_for_basic_type_array!(i32, AParcel_writeInt32Array, AParcel_readInt32Array);
impls_for_basic_type_array!(i64, AParcel_writeInt64Array, AParcel_readInt64Array);
impls_for_basic_type_array!(u32, AParcel_writeUint32Array, AParcel_readUint32Array);
impls_for_basic_type_array!(u64, AParcel_writeUint64Array, AParcel_readUint64Array);

unsafe extern "C" fn typed_nullable_array_allocator<T>(data: *mut c_void, length: i32) -> bool {
    let data = &mut *data.cast::<Option<Vec<T>>>();

    if length < 0 {
        *data = None;
    } else {
        *data = Some(Vec::with_capacity(length as usize));
        data.as_mut().unwrap().set_len(length as usize);
    }

    true
}

impl Read for Option<Vec<bool>> {
    fn read(parcel: &Parcel) -> Result<Self, Status> {
        let mut data: Option<Vec<bool>> = None;

        unsafe {
            unsafe extern "C" fn setter(data: *mut c_void, index: usize, value: bool) {
                let data = &mut *data.cast::<Option<Vec<bool>>>();

                data.as_mut().unwrap()[index] = value;
            }

            Status::from_raw_status_code(AParcel_readBoolArray(
                parcel.as_raw(),
                (&mut data as *mut Option<Vec<bool>>).cast(),
                Some(typed_nullable_array_allocator::<bool>),
                Some(setter),
            ))
            .err(|| data)
        }
    }
}

impl Write for Option<&[bool]> {
    fn write(&self, parcel: &mut Parcel) -> Result<(), Status> {
        unsafe {
            unsafe extern "C" fn getter(data: *const c_void, index: usize) -> bool {
                let data = &*data.cast::<Option<&[bool]>>();

                data.as_ref().unwrap()[index]
            }

            Status::from_raw_status_code(AParcel_writeBoolArray(
                parcel.as_raw(),
                (self as *const Self).cast(),
                match self {
                    None => -1,
                    Some(a) => a.len() as i32,
                },
                Some(getter),
            ))
            .err(|| ())
        }
    }
}

impl Write for Option<Vec<bool>> {
    fn write(&self, parcel: &mut Parcel) -> Result<(), Status> {
        <Option<&[bool]> as Write>::write(&self.as_deref(), parcel)
    }
}

impl Read for Status {
    fn read(parcel: &Parcel) -> Result<Self, Status> {
        unsafe {
            let mut ptr: *mut AStatus = null_mut();
            Status::from_raw_status_code(AParcel_readStatusHeader(parcel.as_raw(), &mut ptr)).err(|| Status::from_raw_status(ptr))
        }
    }
}

impl Write for Status {
    fn write(&self, parcel: &mut Parcel) -> Result<(), Status> {
        unsafe { Status::from_raw_status_code(AParcel_writeStatusHeader(parcel.as_raw(), self.as_raw_status())).err(|| ()) }
    }
}

impl Read for Option<OwnedFd> {
    fn read(parcel: &Parcel) -> Result<Self, Status> {
        unsafe {
            let mut fd: c_int = 0;
            Status::from_raw_status_code(AParcel_readParcelFileDescriptor(parcel.as_raw(), &mut fd)).err(|| {
                if fd >= 0 {
                    Some(OwnedFd::from_raw_fd(fd))
                } else {
                    None
                }
            })
        }
    }
}

impl<'a> Write for Option<BorrowedFd<'a>> {
    fn write(&self, parcel: &mut Parcel) -> Result<(), Status> {
        unsafe {
            let fd = match self {
                None => -1,
                Some(fd) => fd.as_raw_fd(),
            };
            Status::from_raw_status_code(AParcel_writeParcelFileDescriptor(parcel.as_raw(), fd)).err(|| ())
        }
    }
}

impl Write for Option<OwnedFd> {
    fn write(&self, parcel: &mut Parcel) -> Result<(), Status> {
        self.as_ref().map(|fd| fd.as_fd()).write(parcel)
    }
}

impl Read for Option<String> {
    fn read(parcel: &Parcel) -> Result<Self, Status> {
        unsafe {
            let mut ret: Option<Vec<u8>> = None;

            unsafe extern "C" fn allocator(data: *mut c_void, length: i32, buffer: *mut *mut c_char) -> bool {
                let data = &mut *data.cast::<Option<Vec<u8>>>();

                if length < 0 {
                    *data = None;
                } else {
                    let mut array: Vec<u8> = Vec::with_capacity(length as usize);
                    array.set_len((length - 1) as usize);
                    *buffer = array.as_mut_ptr().cast();
                    *data = Some(array);
                }

                true
            }

            Status::from_raw_status_code(AParcel_readString(
                parcel.as_raw(),
                (&mut ret as *mut Option<Vec<u8>>).cast(),
                Some(allocator),
            ))
            .err(|| ())?;

            match ret {
                None => Ok(None),
                Some(ret) => Ok(Some(String::from_utf8(ret).map_err(|_| Status::bad_value())?)),
            }
        }
    }
}

impl Write for Option<&str> {
    fn write(&self, parcel: &mut Parcel) -> Result<(), Status> {
        unsafe {
            let (ptr, len) = match self {
                None => (null(), -1),
                Some(s) => (s.as_bytes().as_ptr(), s.as_bytes().len() as i32),
            };

            Status::from_raw_status_code(AParcel_writeString(parcel.as_raw(), ptr.cast(), len)).err(|| ())
        }
    }
}

impl Write for Option<String> {
    fn write(&self, parcel: &mut Parcel) -> Result<(), Status> {
        <Option<&str>>::write(&self.as_deref(), parcel)
    }
}

impl Read for Option<Vec<Option<String>>> {
    fn read(parcel: &Parcel) -> Result<Self, Status> {
        unsafe {
            let mut data: Option<Vec<Option<Vec<u8>>>> = None;

            unsafe extern "C" fn element_allocator(
                data: *mut c_void,
                index: usize,
                length: i32,
                buffer: *mut *mut c_char,
            ) -> bool {
                let data = &mut *data.cast::<Option<Vec<Option<Vec<u8>>>>>();

                if length < 0 {
                    data.as_mut().unwrap()[index] = None;
                } else {
                    let mut array: Vec<u8> = Vec::with_capacity(length as usize);
                    array.set_len((length - 1) as usize);
                    *buffer = array.as_mut_ptr().cast();
                    std::mem::forget(std::mem::replace(&mut data.as_mut().unwrap()[index], Some(array)));
                }

                true
            }

            Status::from_raw_status_code(AParcel_readStringArray(
                parcel.as_raw(),
                (&mut data as *mut Option<Vec<Option<Vec<u8>>>>).cast(),
                Some(typed_nullable_array_allocator::<Option<Vec<u8>>>),
                Some(element_allocator),
            ))
            .err(|| ())?;

            Ok(match data {
                None => None,
                Some(data) => {
                    let mut ret = Vec::with_capacity(data.len());

                    for x in data {
                        let s = match x {
                            None => None,
                            Some(data) => Some(String::from_utf8(data).map_err(|_| Status::bad_value())?),
                        };

                        ret.push(s);
                    }

                    Some(ret)
                }
            })
        }
    }
}

impl Write for Option<Vec<Option<String>>> {
    fn write(&self, parcel: &mut Parcel) -> Result<(), Status> {
        unsafe {
            unsafe extern "C" fn getter(data: *const c_void, index: usize, length: *mut i32) -> *const c_char {
                let data = &*data.cast::<Option<Vec<Option<String>>>>();

                let s = data.as_ref().unwrap()[index].as_ref();
                match s {
                    None => {
                        *length = -1;

                        null()
                    }
                    Some(s) => {
                        *length = s.as_bytes().len() as i32;

                        s.as_bytes().as_ptr().cast()
                    }
                }
            }

            Status::from_raw_status_code(AParcel_writeStringArray(
                parcel.as_raw(),
                (self as *const Option<Vec<Option<String>>>).cast(),
                match self {
                    None => -1,
                    Some(v) => v.len() as i32,
                },
                Some(getter),
            ))
            .err(|| ())
        }
    }
}

impl Read for Option<IBinder> {
    fn read(parcel: &Parcel) -> Result<Self, Status> {
        unsafe {
            let mut ptr: *mut AIBinder = null_mut();

            Status::from_raw_status_code(AParcel_readStrongBinder(parcel.as_raw(), &mut ptr)).err(|| {
                if ptr.is_null() {
                    None
                } else {
                    Some(IBinder::from_raw(ptr))
                }
            })
        }
    }
}

impl Write for Option<&IBinder> {
    fn write(&self, parcel: &mut Parcel) -> Result<(), Status> {
        unsafe {
            Status::from_raw_status_code(AParcel_writeStrongBinder(
                parcel.as_raw(),
                match self {
                    None => null_mut(),
                    Some(b) => b.as_raw(),
                },
            ))
            .err(|| ())
        }
    }
}

impl Write for Option<IBinder> {
    fn write(&self, parcel: &mut Parcel) -> Result<(), Status> {
        <Option<&IBinder> as Write>::write(&self.as_ref(), parcel)
    }
}
