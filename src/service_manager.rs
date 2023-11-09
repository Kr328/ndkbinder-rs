use std::{
    error::Error,
    ffi::{c_char, CString, NulError},
    fmt::{Debug, Display, Formatter},
    sync::OnceLock,
};

use crate::{
    sys::{binder_exception_t, AIBinder},
    Exception, IBinder,
};

pub enum ServiceManagerError {
    SymbolNotFound(&'static str),
    InvalidString(NulError),
    RemoteException(Exception),
}

impl Debug for ServiceManagerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceManagerError::SymbolNotFound(sym) => f.write_fmt(format_args!("symbol {:?} not found", sym)),
            ServiceManagerError::InvalidString(err) => f.write_fmt(format_args!("invalid string: {:?}", err)),
            ServiceManagerError::RemoteException(ex) => f.write_fmt(format_args!("remote exception: {:?}", ex)),
        }
    }
}

impl Display for ServiceManagerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceManagerError::SymbolNotFound(sym) => f.write_fmt(format_args!("symbol {} not found", sym)),
            ServiceManagerError::InvalidString(err) => f.write_fmt(format_args!("invalid string: {}", err)),
            ServiceManagerError::RemoteException(ex) => f.write_fmt(format_args!("remote exception: {:?}", ex)),
        }
    }
}

impl Error for ServiceManagerError {}

pub struct ServiceManager;

impl ServiceManager {
    fn get_or_check_service(
        func: &OnceLock<unsafe extern "C" fn(*const c_char) -> *mut AIBinder>,
        func_name: &[u8],
        instance: &str,
    ) -> Result<Option<IBinder>, ServiceManagerError> {
        unsafe {
            let func = func.get_or_init(|| std::mem::transmute(libc::dlsym(libc::RTLD_DEFAULT, func_name.as_ptr().cast())));

            let instance = CString::new(instance).map_err(|err| ServiceManagerError::InvalidString(err))?;

            let ptr = func(instance.as_ptr());
            if ptr.is_null() {
                Ok(None)
            } else {
                Ok(Some(IBinder::from_raw(ptr)))
            }
        }
    }

    pub fn get_service(instance: &str) -> Result<Option<IBinder>, ServiceManagerError> {
        static FUNC: OnceLock<unsafe extern "C" fn(*const c_char) -> *mut AIBinder> = OnceLock::new();

        Self::get_or_check_service(&FUNC, "AServiceManager_getService\0".as_bytes(), instance)
    }

    pub fn check_service(instance: &str) -> Result<Option<IBinder>, ServiceManagerError> {
        static FUNC: OnceLock<unsafe extern "C" fn(*const c_char) -> *mut AIBinder> = OnceLock::new();

        Self::get_or_check_service(&FUNC, "AServiceManager_checkService\0".as_bytes(), instance)
    }

    pub fn add_service(instance: &str, binder: &IBinder) -> Result<(), ServiceManagerError> {
        unsafe {
            static FUNC: OnceLock<unsafe extern "C" fn(*mut AIBinder, *const c_char) -> binder_exception_t> = OnceLock::new();
            let func = FUNC.get_or_init(|| {
                std::mem::transmute(libc::dlsym(
                    libc::RTLD_DEFAULT,
                    "AServiceManager_addService\0".as_ptr().cast(),
                ))
            });

            let instance = CString::new(instance).map_err(|err| ServiceManagerError::InvalidString(err))?;

            match Exception::from_raw_exception(func(binder.as_raw(), instance.as_ptr())) {
                Exception::None => Ok(()),
                ex => Err(ServiceManagerError::RemoteException(ex)),
            }
        }
    }
}
