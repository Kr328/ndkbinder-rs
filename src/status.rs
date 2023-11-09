use std::{
    error::Error,
    ffi::{CStr, CString, NulError},
    fmt::{Debug, Display, Formatter},
    str::Utf8Error,
};

use crate::sys::{
    binder_exception_t, binder_status_t, AStatus, AStatus_delete, AStatus_fromExceptionCode,
    AStatus_fromExceptionCodeWithMessage, AStatus_fromServiceSpecificError, AStatus_fromServiceSpecificErrorWithMessage,
    AStatus_fromStatus, AStatus_getExceptionCode, AStatus_getMessage, AStatus_getServiceSpecificError, AStatus_getStatus,
    AStatus_isOk, EX_BAD_PARCELABLE, EX_ILLEGAL_ARGUMENT, EX_ILLEGAL_STATE, EX_NETWORK_MAIN_THREAD, EX_NONE, EX_NULL_POINTER,
    EX_PARCELABLE, EX_SECURITY, EX_SERVICE_SPECIFIC, EX_TRANSACTION_FAILED, EX_UNSUPPORTED_OPERATION,
};

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum Exception {
    None,
    Security,
    BadParcelable,
    IllegalArgument,
    NullPointer,
    IllegalState,
    NetworkMainThread,
    UnsupportedOperation,
    ServiceSpecific,
    Parcelable,
    TransactionFailed,
}

impl Exception {
    pub(crate) const fn from_raw_exception(raw: binder_exception_t) -> Exception {
        match raw {
            EX_NONE => Exception::None,
            EX_SECURITY => Exception::Security,
            EX_BAD_PARCELABLE => Exception::BadParcelable,
            EX_ILLEGAL_ARGUMENT => Exception::IllegalArgument,
            EX_NULL_POINTER => Exception::NullPointer,
            EX_ILLEGAL_STATE => Exception::IllegalState,
            EX_NETWORK_MAIN_THREAD => Exception::NetworkMainThread,
            EX_UNSUPPORTED_OPERATION => Exception::UnsupportedOperation,
            EX_SERVICE_SPECIFIC => Exception::ServiceSpecific,
            EX_PARCELABLE => Exception::Parcelable,
            EX_TRANSACTION_FAILED => Exception::TransactionFailed,
            _ => Exception::IllegalArgument,
        }
    }

    pub(crate) const fn as_raw_exception(self) -> binder_exception_t {
        match self {
            Exception::None => EX_NONE,
            Exception::Security => EX_SECURITY,
            Exception::BadParcelable => EX_BAD_PARCELABLE,
            Exception::IllegalArgument => EX_ILLEGAL_ARGUMENT,
            Exception::NullPointer => EX_NULL_POINTER,
            Exception::IllegalState => EX_ILLEGAL_STATE,
            Exception::NetworkMainThread => EX_NETWORK_MAIN_THREAD,
            Exception::UnsupportedOperation => EX_UNSUPPORTED_OPERATION,
            Exception::ServiceSpecific => EX_SERVICE_SPECIFIC,
            Exception::Parcelable => EX_PARCELABLE,
            Exception::TransactionFailed => EX_TRANSACTION_FAILED,
        }
    }
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum Code {
    Ok,
    UnknownError,
    NoMemory,
    InvalidOperation,
    BadValue,
    BadType,
    NameNotFound,
    PermissionDenied,
    NoInit,
    AlreadyExists,
    DeadObject,
    FailedTransaction,
    BadIndex,
    NotEnoughData,
    WouldBlock,
    TimedOut,
    UnknownTransaction,
    FdsNotAllowed,
    UnexpectedNull,
}

impl Code {
    pub(crate) const fn as_raw(&self) -> binder_status_t {
        match self {
            Code::Ok => crate::sys::STATUS_OK,
            Code::UnknownError => crate::sys::STATUS_UNKNOWN_ERROR,
            Code::NoMemory => crate::sys::STATUS_NO_MEMORY,
            Code::InvalidOperation => crate::sys::STATUS_INVALID_OPERATION,
            Code::BadValue => crate::sys::STATUS_BAD_VALUE,
            Code::BadType => crate::sys::STATUS_BAD_TYPE,
            Code::NameNotFound => crate::sys::STATUS_NAME_NOT_FOUND,
            Code::PermissionDenied => crate::sys::STATUS_PERMISSION_DENIED,
            Code::NoInit => crate::sys::STATUS_NO_INIT,
            Code::AlreadyExists => crate::sys::STATUS_ALREADY_EXISTS,
            Code::DeadObject => crate::sys::STATUS_DEAD_OBJECT,
            Code::FailedTransaction => crate::sys::STATUS_FAILED_TRANSACTION,
            Code::BadIndex => crate::sys::STATUS_BAD_INDEX,
            Code::NotEnoughData => crate::sys::STATUS_NOT_ENOUGH_DATA,
            Code::WouldBlock => crate::sys::STATUS_WOULD_BLOCK,
            Code::TimedOut => crate::sys::STATUS_TIMED_OUT,
            Code::UnknownTransaction => crate::sys::STATUS_UNKNOWN_TRANSACTION,
            Code::FdsNotAllowed => crate::sys::STATUS_FDS_NOT_ALLOWED,
            Code::UnexpectedNull => crate::sys::STATUS_UNEXPECTED_NULL,
        }
    }

    pub(crate) const fn from_raw(raw: binder_status_t) -> Code {
        match raw {
            crate::sys::STATUS_OK => Code::Ok,
            crate::sys::STATUS_UNKNOWN_ERROR => Code::UnknownError,
            crate::sys::STATUS_NO_MEMORY => Code::NoMemory,
            crate::sys::STATUS_INVALID_OPERATION => Code::InvalidOperation,
            crate::sys::STATUS_BAD_VALUE => Code::BadValue,
            crate::sys::STATUS_BAD_TYPE => Code::BadType,
            crate::sys::STATUS_NAME_NOT_FOUND => Code::NameNotFound,
            crate::sys::STATUS_PERMISSION_DENIED => Code::PermissionDenied,
            crate::sys::STATUS_NO_INIT => Code::NoInit,
            crate::sys::STATUS_ALREADY_EXISTS => Code::AlreadyExists,
            crate::sys::STATUS_DEAD_OBJECT => Code::DeadObject,
            crate::sys::STATUS_FAILED_TRANSACTION => Code::FailedTransaction,
            crate::sys::STATUS_BAD_INDEX => Code::BadIndex,
            crate::sys::STATUS_NOT_ENOUGH_DATA => Code::NotEnoughData,
            crate::sys::STATUS_WOULD_BLOCK => Code::WouldBlock,
            crate::sys::STATUS_TIMED_OUT => Code::TimedOut,
            crate::sys::STATUS_UNKNOWN_TRANSACTION => Code::UnknownTransaction,
            crate::sys::STATUS_FDS_NOT_ALLOWED => Code::FdsNotAllowed,
            crate::sys::STATUS_UNEXPECTED_NULL => Code::UnexpectedNull,
            _ => Code::UnknownError,
        }
    }
}

pub struct Status {
    ptr: *mut AStatus,
}

impl Debug for Status {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        #[cfg(feature = "api-30")]
        return f.write_str(self.get_description().as_deref().unwrap_or("unknown"));

        #[cfg(not(feature = "api-30"))]
        return self.get_code().fmt(f);
    }
}

impl Error for Status {}

unsafe impl Send for Status {}

impl Status {
    pub(crate) fn as_raw_status(&self) -> *const AStatus {
        self.ptr
    }

    pub(crate) unsafe fn from_raw_status(status: *mut AStatus) -> Status {
        Status { ptr: status }
    }

    pub(crate) fn from_raw_status_code(status: binder_status_t) -> Status {
        unsafe {
            Status {
                ptr: AStatus_fromStatus(status),
            }
        }
    }
}

impl Status {
    pub fn bad_value() -> Status {
        Status::from_raw_status_code(crate::sys::STATUS_BAD_VALUE)
    }

    pub fn unknown_transaction() -> Status {
        Status::from_raw_status_code(crate::sys::STATUS_UNKNOWN_TRANSACTION)
    }

    pub fn with_code(code: Code) -> Status {
        Status::from_raw_status_code(code.as_raw())
    }

    pub fn with_service_specific_error(error: i32) -> Status {
        unsafe {
            Status {
                ptr: AStatus_fromServiceSpecificError(error),
            }
        }
    }

    pub fn with_service_specific_error_and_message(error: i32, msg: &str) -> Result<Status, NulError> {
        unsafe {
            let msg = CString::new(msg)?;

            Ok(Status {
                ptr: AStatus_fromServiceSpecificErrorWithMessage(error, msg.as_ptr()),
            })
        }
    }

    pub fn with_exception(ex: Exception) -> Status {
        unsafe {
            Status {
                ptr: AStatus_fromExceptionCode(ex.as_raw_exception()),
            }
        }
    }

    pub fn with_exception_and_message(ex: Exception, msg: &str) -> Result<Status, NulError> {
        unsafe {
            let msg = CString::new(msg)?;

            Ok(Status {
                ptr: AStatus_fromExceptionCodeWithMessage(ex.as_raw_exception(), msg.as_ptr()),
            })
        }
    }

    pub fn get_code(&self) -> Code {
        unsafe { Code::from_raw(AStatus_getStatus(self.ptr)) }
    }

    pub fn get_message(&self) -> Result<Option<String>, Utf8Error> {
        unsafe {
            let msg = AStatus_getMessage(self.ptr);
            if msg.is_null() {
                Ok(None)
            } else {
                Ok(Some(CStr::from_ptr(msg).to_str()?.to_string()))
            }
        }
    }

    #[cfg(feature = "api-30")]
    pub fn get_description(&self) -> Result<String, Utf8Error> {
        unsafe {
            let description = crate::sys::AStatus_getDescription(self.ptr);

            let ret = CStr::from_ptr(description).to_str()?.to_string();

            crate::sys::AStatus_deleteDescription(description);

            Ok(ret)
        }
    }

    pub fn get_service_specific_error(&self) -> i32 {
        unsafe { AStatus_getServiceSpecificError(self.ptr) }
    }

    pub fn get_exception(&self) -> Exception {
        unsafe { Exception::from_raw_exception(AStatus_getExceptionCode(self.ptr)) }
    }

    pub fn err<R>(self, or_value: impl FnOnce() -> R) -> Result<R, Status> {
        if unsafe { AStatus_isOk(self.ptr) } {
            Ok(or_value())
        } else {
            Err(self)
        }
    }
}

impl Drop for Status {
    fn drop(&mut self) {
        unsafe { AStatus_delete(self.ptr) }
    }
}
