use std::{
    ffi::{CStr, CString},
    ops::Deref,
    os::{
        fd::RawFd,
        raw::{c_char, c_int, c_void},
    },
    sync::Arc,
};

use crate::{
    sys::{
        binder_status_t, transaction_code_t, AIBinder, AIBinder_Class, AIBinder_Class_define, AIBinder_Class_setOnDump,
        AIBinder_getUserData, AIBinder_new, AParcel,
    },
    Code, IBinder, Parcel, Status,
};

pub trait Class: Send + Sync {
    const INTERFACE_NAME: &'static str;

    #[cfg(feature = "api-33")]
    fn disable_interface_token_header() -> bool {
        false
    }

    fn on_transact(&self, code: u32, data: &Parcel, reply: Option<&mut Parcel>) -> Result<(), Status>;

    fn on_dump(&self, fd: RawFd, args: &[&str]) -> Result<(), Status> {
        let _ = fd;
        let _ = args;

        Err(Status::unknown_transaction())
    }
}

macro_rules! impl_delegated {
    ($typ:ty) => {
        impl<T: Class> Class for $typ {
            const INTERFACE_NAME: &'static str = T::INTERFACE_NAME;

            fn on_transact(&self, code: u32, data: &Parcel, reply: Option<&mut Parcel>) -> Result<(), Status> {
                <$typ as AsRef<T>>::as_ref(self).on_transact(code, data, reply)
            }

            fn on_dump(&self, fd: RawFd, args: &[&str]) -> Result<(), Status> {
                <$typ as AsRef<T>>::as_ref(self).on_dump(fd, args)
            }
        }
    };
}

impl_delegated!(Box<T>);
impl_delegated!(Arc<T>);

#[doc(hidden)]
pub fn _define_class_impl<T: Class>() -> usize {
    let interface_name = CString::new(T::INTERFACE_NAME).unwrap();

    unsafe extern "C" fn on_create(args: *mut c_void) -> *mut c_void {
        args
    }

    unsafe extern "C" fn on_destroy<T: Class>(data: *mut c_void) {
        drop(Box::from_raw(data.cast::<T>()))
    }

    unsafe extern "C" fn on_transact<T: Class>(
        binder: *mut AIBinder,
        code: transaction_code_t,
        data: *const AParcel,
        reply: *mut AParcel,
    ) -> binder_status_t {
        if data.is_null() {
            return Code::BadValue.as_raw();
        }

        let obj = AIBinder_getUserData(binder);
        let data = Parcel::from_borrow_raw(data.cast_mut());
        let mut reply = if reply.is_null() {
            None
        } else {
            Some(Parcel::from_borrow_raw(reply.cast()))
        };

        match (&*obj.cast::<T>()).on_transact(code, &data, reply.as_mut()) {
            Ok(_) => Code::Ok.as_raw(),
            Err(err) => err.get_code().as_raw(),
        }
    }

    unsafe extern "C" fn on_dump<T: Class>(
        binder: *mut AIBinder,
        fd: c_int,
        args: *mut *const c_char,
        args_len: u32,
    ) -> binder_status_t {
        let obj = AIBinder_getUserData(binder);
        let args = (0..args_len)
            .map(|idx| CStr::from_ptr(*args.offset(idx as isize)).to_string_lossy())
            .collect::<Vec<_>>();
        let args: Vec<&str> = args.iter().map(|s| s.deref()).collect::<Vec<_>>();

        match (&*obj.cast::<T>()).on_dump(fd, &args[..]) {
            Ok(_) => Code::Ok.as_raw(),
            Err(err) => err.get_code().as_raw(),
        }
    }

    unsafe {
        let ret = AIBinder_Class_define(
            interface_name.as_ptr(),
            Some(on_create),
            Some(on_destroy::<T>),
            Some(on_transact::<T>),
        );

        AIBinder_Class_setOnDump(ret, Some(on_dump::<T>));

        #[cfg(feature = "api-33")]
        if T::disable_interface_token_header() {
            crate::sys::AIBinder_Class_disableInterfaceTokenHeader(ret);
        }

        ret as usize
    }
}

#[doc(hidden)]
pub fn _new_ibinder_with_class<T: Class>(class: usize, binder: T) -> IBinder {
    unsafe {
        let class = class as *mut AIBinder_Class;

        IBinder::from_raw(AIBinder_new(class, Box::into_raw(Box::new(binder)).cast()))
    }
}

#[macro_export]
macro_rules! define_class {
    ($class:ty) => {
        impl ::std::convert::Into<$crate::IBinder> for $class {
            fn into(self) -> $crate::IBinder {
                static CLASS: ::std::sync::OnceLock<usize> = ::std::sync::OnceLock::new();

                let class = CLASS.get_or_init(|| $crate::_define_class_impl::<$class>());

                $crate::_new_ibinder_with_class(*class, self)
            }
        }
    };
}
