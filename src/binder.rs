use std::{
    ffi::{c_void, CString},
    fmt::{Debug, Formatter},
    os::fd::RawFd,
    ptr::null_mut,
};

use crate::{
    sys::{
        AIBinder, AIBinder_DeathRecipient, AIBinder_DeathRecipient_delete, AIBinder_DeathRecipient_new, AIBinder_Weak,
        AIBinder_Weak_delete, AIBinder_Weak_new, AIBinder_Weak_promote, AIBinder_decStrong, AIBinder_dump,
        AIBinder_getCallingPid, AIBinder_getCallingUid, AIBinder_incStrong, AIBinder_isAlive, AIBinder_isRemote,
        AIBinder_linkToDeath, AIBinder_ping, AIBinder_prepareTransaction, AIBinder_transact, AIBinder_unlinkToDeath, AParcel,
        FLAG_ONEWAY,
    },
    Parcel, Status,
};

pub enum Flags {
    Oneway,
}

pub struct IBinder {
    ptr: *mut AIBinder,
}

unsafe impl Send for IBinder {}

unsafe impl Sync for IBinder {}

impl Debug for IBinder {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self.ptr))
    }
}

impl Clone for IBinder {
    fn clone(&self) -> Self {
        unsafe { AIBinder_incStrong(self.ptr) }

        IBinder { ptr: self.ptr }
    }
}

impl Drop for IBinder {
    fn drop(&mut self) {
        unsafe { AIBinder_decStrong(self.ptr) }
    }
}

#[cfg(feature = "api-31")]
const _: () = {
    impl PartialEq<Self> for IBinder {
        fn eq(&self, other: &Self) -> bool {
            <Self as Ord>::cmp(self, other).is_eq()
        }
    }

    impl Eq for IBinder {}

    impl PartialOrd for IBinder {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(<Self as Ord>::cmp(self, other))
        }
    }

    impl Ord for IBinder {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            if unsafe { crate::sys::AIBinder_lt(self.ptr, other.ptr) } {
                std::cmp::Ordering::Less
            } else if unsafe { crate::sys::AIBinder_lt(other.ptr, self.ptr) } {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        }
    }
};

impl IBinder {
    pub(crate) unsafe fn from_raw(ptr: *mut AIBinder) -> Self {
        if ptr.is_null() {
            panic!("unexpected null AIBinder");
        }

        IBinder { ptr }
    }

    pub(crate) fn as_raw(&self) -> *mut AIBinder {
        self.ptr
    }

    pub fn get_calling_pid() -> u32 {
        unsafe { AIBinder_getCallingPid() as u32 }
    }

    pub fn get_calling_uid() -> u32 {
        unsafe { AIBinder_getCallingUid() }
    }

    #[cfg(feature = "api-33")]
    pub fn is_handling_transaction() -> bool {
        unsafe { crate::sys::AIBinder_isHandlingTransaction() }
    }

    pub fn dump(&self, fd: RawFd, args: &[&str]) -> Result<(), Status> {
        let mut c_args = Vec::with_capacity(args.len());
        for &s in args {
            c_args.push(CString::new(s).map_err(|_| Status::bad_value())?);
        }
        let mut c_args = c_args.iter().map(|s| s.as_ptr()).collect::<Vec<_>>();

        unsafe { Status::from_raw_status_code(AIBinder_dump(self.ptr, fd, c_args.as_mut_ptr(), c_args.len() as u32)).err(|| ()) }
    }

    #[cfg(feature = "api-30")]
    pub fn get_extension(&self) -> Result<IBinder, Status> {
        unsafe {
            let mut ptr: *mut AIBinder = null_mut();

            Status::from_raw_status_code(crate::sys::AIBinder_getExtension(self.ptr, &mut ptr)).err(|| Self::from_raw(ptr))
        }
    }

    #[cfg(feature = "api-30")]
    pub fn set_extension(&self, extension: &IBinder) -> Result<(), Status> {
        unsafe { Status::from_raw_status_code(crate::sys::AIBinder_setExtension(self.ptr, extension.ptr)).err(|| ()) }
    }

    pub fn is_alive(&self) -> bool {
        unsafe { AIBinder_isAlive(self.ptr) }
    }

    pub fn is_remote(&self) -> bool {
        unsafe { AIBinder_isRemote(self.ptr) }
    }

    pub fn ping(&self) -> Result<(), Status> {
        unsafe { Status::from_raw_status_code(AIBinder_ping(self.ptr)).err(|| ()) }
    }

    pub fn weak_ref(&self) -> WeakIBinder {
        WeakIBinder {
            ptr: unsafe { AIBinder_Weak_new(self.ptr) },
        }
    }

    pub fn transact<O, D, R>(&self, code: u32, data: D, reply: R, flags: &[Flags]) -> Result<O, Status>
    where
        D: FnOnce(&mut Parcel) -> Result<(), Status>,
        R: FnOnce(Option<&Parcel>) -> Result<O, Status>,
    {
        unsafe {
            let mut data_parcel: *mut AParcel = null_mut();

            Status::from_raw_status_code(AIBinder_prepareTransaction(self.ptr, &mut data_parcel)).err(|| ())?;

            data(&mut Parcel::from_borrow_raw(data_parcel))?;

            let mut reply_parcel: *mut AParcel = null_mut();

            let flags = flags.iter().fold(0, |v, f| match f {
                Flags::Oneway => v | FLAG_ONEWAY,
            });

            Status::from_raw_status_code(AIBinder_transact(self.ptr, code, &mut data_parcel, &mut reply_parcel, flags))
                .err(|| ())?;

            let reply_parcel = if reply_parcel.is_null() {
                None
            } else {
                Some(Parcel::from_raw(reply_parcel))
            };

            reply(reply_parcel.as_ref())
        }
    }
}

#[cfg(feature = "jni")]
impl IBinder {
    pub unsafe fn from_java(env: *mut jni_sys::JNIEnv, obj: jni_sys::jobject) -> Option<Self> {
        extern "C" {
            fn AIBinder_fromJavaBinder(env: *mut jni_sys::JNIEnv, obj: jni_sys::jobject) -> *mut AIBinder;
        }

        let ptr = AIBinder_fromJavaBinder(env, obj);
        if ptr.is_null() {
            None
        } else {
            Some(IBinder::from_raw(ptr))
        }
    }

    pub fn as_java(&self, env: *mut jni_sys::JNIEnv) -> jni_sys::jobject {
        extern "C" {
            fn AIBinder_toJavaBinder(env: *mut jni_sys::JNIEnv, obj: *mut AIBinder) -> jni_sys::jobject;
        }

        unsafe { AIBinder_toJavaBinder(env, self.ptr) }
    }
}

pub struct WeakIBinder {
    ptr: *mut AIBinder_Weak,
}

unsafe impl Send for WeakIBinder {}

unsafe impl Sync for WeakIBinder {}

impl Drop for WeakIBinder {
    fn drop(&mut self) {
        unsafe { AIBinder_Weak_delete(self.ptr) }
    }
}

#[cfg(feature = "api-31")]
impl Clone for WeakIBinder {
    fn clone(&self) -> Self {
        WeakIBinder {
            ptr: unsafe { crate::sys::AIBinder_Weak_clone(self.ptr) },
        }
    }
}

#[cfg(feature = "api-31")]
const _: () = {
    impl PartialEq<Self> for WeakIBinder {
        fn eq(&self, other: &Self) -> bool {
            <Self as Ord>::cmp(self, other).is_eq()
        }
    }

    impl Eq for WeakIBinder {}

    impl PartialOrd for WeakIBinder {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(<Self as Ord>::cmp(self, other))
        }
    }

    impl Ord for WeakIBinder {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            if unsafe { crate::sys::AIBinder_Weak_lt(self.ptr, other.ptr) } {
                std::cmp::Ordering::Less
            } else if unsafe { crate::sys::AIBinder_Weak_lt(other.ptr, self.ptr) } {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        }
    }
};

impl WeakIBinder {
    pub fn upgrade(&self) -> Option<IBinder> {
        let ptr = unsafe { AIBinder_Weak_promote(self.ptr) };
        if ptr.is_null() {
            None
        } else {
            Some(IBinder { ptr })
        }
    }
}

pub trait DeathRecipient: Send {
    fn on_dead(self);
}

pub struct LinkedDeathRecipient<B: AsRef<IBinder>, R: DeathRecipient> {
    binder: B,
    recipient: *mut Option<R>,
    recipient_spec: *mut AIBinder_DeathRecipient,
}

impl<B: AsRef<IBinder>, R: DeathRecipient> Drop for LinkedDeathRecipient<B, R> {
    fn drop(&mut self) {
        unsafe {
            AIBinder_unlinkToDeath(self.binder.as_ref().as_raw(), self.recipient_spec, self.recipient.cast());

            AIBinder_DeathRecipient_delete(self.recipient_spec);

            drop(Box::from_raw(self.recipient));
        }
    }
}

impl IBinder {
    pub fn link_to_death<B: AsRef<IBinder>, R: DeathRecipient>(
        binder: B,
        recipient: R,
    ) -> Result<LinkedDeathRecipient<B, R>, Status> {
        unsafe {
            unsafe extern "C" fn on_dead<R: DeathRecipient>(cookies: *mut c_void) {
                if let Some(r) = (*cookies.cast::<Option<R>>()).take() {
                    r.on_dead();
                }
            }

            let recipient_spec = AIBinder_DeathRecipient_new(Some(on_dead::<R>));
            let recipient: *mut Option<R> = Box::into_raw(Box::new(Some(recipient)));
            let linked = LinkedDeathRecipient {
                binder,
                recipient,
                recipient_spec,
            };

            Status::from_raw_status_code(AIBinder_linkToDeath(
                linked.binder.as_ref().as_raw(),
                recipient_spec,
                linked.recipient.cast(),
            ))
            .err(|| linked)
        }
    }
}
