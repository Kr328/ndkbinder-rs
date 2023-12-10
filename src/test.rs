use std::{ffi::CString, os::fd::*};

use fake::*;

use crate::*;

#[test]
#[cfg(feature = "api-31")]
fn parcel_basic() {
    #[derive(Debug, Dummy)]
    struct Parcelables {
        bool: bool,
        bool_array: Option<Vec<bool>>,
        byte: i8,
        byte_array: Option<Vec<i8>>,
        char: u16,
        char_array: Option<Vec<u16>>,
        double: f64,
        double_array: Option<Vec<f64>>,
        int_32: i32,
        int32_array: Option<Vec<i32>>,
        int64: i64,
        int64_array: Option<Vec<i64>>,
        string: Option<String>,
        string_array: Option<Vec<Option<String>>>,
        uint32: u32,
        uint32_array: Option<Vec<u32>>,
        uint64: u64,
        uint64_array: Option<Vec<u64>>,
    }

    for _ in 0..100 {
        let parcelables: Parcelables = Faker.fake();

        let mut parcel = Parcel::new();
        parcel.write(&parcelables.bool).unwrap();
        parcel.write(&parcelables.bool_array).unwrap();
        parcel.write(&parcelables.bool).unwrap();
        parcel.write(&parcelables.bool_array).unwrap();
        parcel.write(&parcelables.byte).unwrap();
        parcel.write(&parcelables.byte_array).unwrap();
        parcel.write(&parcelables.char).unwrap();
        parcel.write(&parcelables.char_array).unwrap();
        parcel.write(&parcelables.double).unwrap();
        parcel.write(&parcelables.double_array).unwrap();
        parcel.write(&parcelables.int_32).unwrap();
        parcel.write(&parcelables.int32_array).unwrap();
        parcel.write(&parcelables.int64).unwrap();
        parcel.write(&parcelables.int64_array).unwrap();
        parcel.write(&parcelables.string).unwrap();
        parcel.write(&parcelables.string_array).unwrap();
        parcel.write(&parcelables.uint32).unwrap();
        parcel.write(&parcelables.uint32_array).unwrap();
        parcel.write(&parcelables.uint64).unwrap();
        parcel.write(&parcelables.uint64_array).unwrap();

        parcel.set_data_position(0).unwrap();

        assert_eq!(&parcelables.bool, &parcel.read().unwrap());
        assert_eq!(&parcelables.bool_array, &parcel.read().unwrap());
        assert_eq!(&parcelables.bool, &parcel.read().unwrap());
        assert_eq!(&parcelables.bool_array, &parcel.read().unwrap());
        assert_eq!(&parcelables.byte, &parcel.read().unwrap());
        assert_eq!(&parcelables.byte_array, &parcel.read().unwrap());
        assert_eq!(&parcelables.char, &parcel.read().unwrap());
        assert_eq!(&parcelables.char_array, &parcel.read().unwrap());
        assert_eq!(&parcelables.double, &parcel.read().unwrap());
        assert_eq!(&parcelables.double_array, &parcel.read().unwrap());
        assert_eq!(&parcelables.int_32, &parcel.read().unwrap());
        assert_eq!(&parcelables.int32_array, &parcel.read().unwrap());
        assert_eq!(&parcelables.int64, &parcel.read().unwrap());
        assert_eq!(&parcelables.int64_array, &parcel.read().unwrap());
        assert_eq!(&parcelables.string, &parcel.read().unwrap());
        assert_eq!(&parcelables.string_array, &parcel.read().unwrap());
        assert_eq!(&parcelables.uint32, &parcel.read().unwrap());
        assert_eq!(&parcelables.uint32_array, &parcel.read().unwrap());
        assert_eq!(&parcelables.uint64, &parcel.read().unwrap());
        assert_eq!(&parcelables.uint64_array, &parcel.read().unwrap());
    }
}

#[test]
#[cfg(feature = "api-31")]
fn parcel_alive_obj() {
    let pipe = unsafe {
        let mut pipe: [RawFd; 2] = [0; 2];
        assert!(libc::pipe(pipe.as_mut_ptr()) >= 0);
        pipe.map(|fd| OwnedFd::from_raw_fd(fd))
    };

    struct FakeService;

    impl Class for FakeService {
        const INTERFACE_NAME: &'static str = "com.github.kr328.NdkBinder";

        fn on_transact(&self, _code: u32, _data: &Parcel, _reply: Option<&mut Parcel>) -> Result<(), Status> {
            Ok(())
        }
    }

    define_class!(FakeService);

    fn fd_id(fd: RawFd) -> String {
        std::fs::read_link(format!("/proc/self/fd/{fd}"))
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    }

    let binder: IBinder = FakeService.into();

    let mut parcel = Parcel::new();
    parcel.write(&Some(pipe[0].as_fd())).unwrap();
    parcel.write(&Some(&binder)).unwrap();
    parcel.write(&Some(pipe[1].as_fd())).unwrap();
    parcel.write::<Option<BorrowedFd>>(&None).unwrap();
    parcel.write::<Option<IBinder>>(&None).unwrap();

    parcel.set_data_position(0).unwrap();

    let pipe1: Option<OwnedFd> = parcel.read().unwrap();
    let rbinder: Option<IBinder> = parcel.read().unwrap();
    let pipe2: Option<OwnedFd> = parcel.read().unwrap();
    let none_fd: Option<OwnedFd> = parcel.read().unwrap();
    let none_binder: Option<IBinder> = parcel.read().unwrap();

    assert_eq!(fd_id(pipe[0].as_raw_fd()), fd_id(pipe1.unwrap().as_raw_fd()));
    assert_eq!(binder, rbinder.unwrap());
    assert_eq!(fd_id(pipe[1].as_raw_fd()), fd_id(pipe2.unwrap().as_raw_fd()));
    assert!(none_fd.is_none());
    assert!(none_binder.is_none());
}

#[test]
#[cfg(feature = "api-31")]
fn parcel_parcelable_array() {
    for _ in 0..100 {
        let data: Option<Vec<Option<String>>> = Faker.fake();

        let mut parcel = Parcel::new();
        parcel.write_array(data.as_deref()).unwrap();

        parcel.set_data_position(0).unwrap();

        let r_data: Option<Vec<Option<String>>> = parcel.read_array().unwrap();

        assert_eq!(data, r_data);
    }
}

#[test]
#[cfg(feature = "api-31")]
fn parcel_status() {
    let mut parcel = Parcel::new();

    assert!(matches!(parcel.write(&Status::bad_value()), Err(s) if s.get_code() == Code::BadValue));

    let msg: String = Faker.fake();

    parcel
        .write(&Status::with_exception_and_message(Exception::IllegalArgument, &msg).unwrap())
        .unwrap();

    parcel.set_data_position(0).unwrap();

    let r_status: Status = parcel.read().unwrap();

    assert_eq!(r_status.get_exception(), Exception::IllegalArgument);
    assert_eq!(r_status.get_message().unwrap().unwrap(), msg);

    let mut parcel = Parcel::new();

    let error: i32 = Faker.fake();
    let msg: String = Faker.fake();

    parcel
        .write(&Status::with_service_specific_error_and_message(error, &msg).unwrap())
        .unwrap();

    parcel.set_data_position(0).unwrap();

    let r_status: Status = parcel.read().unwrap();

    assert_eq!(r_status.get_service_specific_error(), error);
    assert_eq!(r_status.get_message().unwrap().unwrap(), msg);
}

#[test]
#[cfg(feature = "api-31")]
fn binder_local_transact() {
    struct LocalService;

    impl Class for LocalService {
        const INTERFACE_NAME: &'static str = "com.github.kr328.NdkBinder";

        fn on_transact(&self, code: u32, data: &Parcel, reply: Option<&mut Parcel>) -> Result<(), Status> {
            match code {
                1 => {
                    let s: Option<String> = data.read()?;

                    reply.unwrap().write(&s.map(|s| s + "114514"))?;

                    Ok(())
                }
                2 => {
                    let v: u64 = data.read()?;

                    reply.unwrap().write(&(v + 114514))?;

                    Ok(())
                }
                _ => Err(Status::unknown_transaction()),
            }
        }
    }

    define_class!(LocalService);

    let binder: IBinder = LocalService.into();

    let s: Option<String> = Faker.fake();

    let rs = binder
        .transact(1, |data| data.write(&s), |reply| reply.unwrap().read::<Option<String>>(), &[])
        .unwrap();

    assert_eq!(s.map(|s| s + "114514"), rs);

    let v: u64 = Faker.fake();

    let rv = binder
        .transact(2, |data| data.write(&v), |reply| reply.unwrap().read::<u64>(), &[])
        .unwrap();

    assert_eq!(v + 114514, rv);

    let rn = binder.transact(3, |_| Ok(()), |_| Ok(()), &[]);

    assert!(matches!(rn, Err(st) if st.get_code() == Code::UnknownTransaction));
}

fn open_memfd() -> OwnedFd {
    unsafe {
        let func: unsafe extern "C" fn(name: *const std::ffi::c_char, flags: std::ffi::c_uint) -> RawFd =
            std::mem::transmute(libc::dlsym(libc::RTLD_DEFAULT, "memfd_create\0".as_ptr().cast()));

        OwnedFd::from_raw_fd(func("file\0".as_ptr().cast(), 0))
    }
}

#[test]
fn binder_dump() {
    struct DumpService;

    impl Class for DumpService {
        const INTERFACE_NAME: &'static str = "com.github.kr328.NdkBinder";

        fn on_transact(&self, _: u32, _: &Parcel, _: Option<&mut Parcel>) -> Result<(), Status> {
            Err(Status::unknown_transaction())
        }

        fn on_dump(&self, fd: RawFd, args: &[&str]) -> Result<(), Status> {
            let args = CString::new(args.join(",")).unwrap().into_bytes();

            unsafe {
                libc::write(fd, args.as_ptr().cast(), args.len());
            }

            Ok(())
        }
    }

    define_class!(DumpService);

    let binder: IBinder = DumpService.into();

    let fd = open_memfd();
    let args: Vec<String> = Faker.fake();
    let args = args.iter().map(|s| s.as_ref()).collect::<Vec<_>>();

    binder.dump(fd.as_raw_fd(), &args).unwrap();

    let content = std::fs::read_to_string(format!("/proc/self/fd/{}", fd.as_raw_fd())).unwrap();

    assert_eq!(content, args.join(","));
}

#[cfg(feature = "service_manager")]
#[test]
fn binder_dump_sys_service() {
    use std::io::Read;

    let fd = open_memfd();

    let binder = crate::service_manager::ServiceManager::get_service("activity")
        .unwrap()
        .unwrap();

    binder.dump(fd.as_raw_fd(), &[]).unwrap();

    let mut data: String = String::new();
    std::process::Command::new("dumpsys")
        .arg("activity")
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap()
        .stdout
        .unwrap()
        .read_to_string(&mut data)
        .unwrap();

    let r_data = std::fs::read_to_string(format!("/proc/self/fd/{}", fd.as_raw_fd())).unwrap();

    assert_eq!(&data[..5], &r_data[..5]);
    assert_eq!(&data[data.len() - 5..], &r_data[r_data.len() - 5..]);
}
