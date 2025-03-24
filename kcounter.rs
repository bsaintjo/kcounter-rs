use core::{
    pin::Pin,
    sync::atomic::{AtomicU32, Ordering},
};
use kernel::{
    alloc::KBox,
    c_str,
    error::{Error, Result},
    fs::file::File,
    ioctl::{_IOC_SIZE, _IOR},
    miscdevice::{MiscDevice, MiscDeviceOptions, MiscDeviceRegistration},
    prelude::*,
    uaccess::{UserSlice, UserSliceWriter},
};

module! {
    type: RustKCounterModule,
    name: "kcounter",
    author: "Brandon Saint-John",
    description: "Count the number of times the device is opened",
    license: "GPL",
}

#[pin_data]
struct RustKCounterModule {
    #[pin]
    miscdev: MiscDeviceRegistration<KCounterDevice>,
}

impl kernel::InPlaceModule for RustKCounterModule {
    fn init(_module: &'static ThisModule) -> impl PinInit<Self, Error> {
        pr_info!("Rust kcounter (init)\n");

        let options = MiscDeviceOptions {
            name: c_str!("kcounter"),
        };

        try_pin_init!(Self {
            miscdev <- MiscDeviceRegistration::register(options),
        })
    }
}

#[pin_data(PinnedDrop)]
struct KCounterDevice {
    counter: AtomicU32,
    // dev: ARef<Device>,
}

impl KCounterDevice {
    fn new() -> impl PinInit<Self, Error> {
        try_pin_init!( Self {
            counter: AtomicU32::default()
        }? Error)
    }

    fn respond(self: Pin<&Self>, mut uslice: UserSliceWriter) -> Result<isize> {
        let n = self.counter.fetch_add(1, Ordering::Relaxed);
        uslice
            .write(&n)
            .inspect_err(|_| log::error!("User slice full"))?;
        Ok(0)
    }
}

const RESPOND: u32 = _IOR::<i32>('|' as u32, 0x81);

#[vtable]
impl MiscDevice for KCounterDevice {
    type Ptr = Pin<KBox<Self>>;

    fn open(_file: &File, _misc: &MiscDeviceRegistration<Self>) -> Result<Self::Ptr> {
        // let dev = ARef::from(misc.device());
        // let inner = try_pin_init!(KCounterDevice {
        //     counter: AtomicU32::default(),
        //     // dev: dev,
        // });
        KBox::try_pin_init(Self::new(), GFP_KERNEL)
    }

    fn ioctl(self: Pin<&KCounterDevice>, _file: &File, cmd: u32, arg: usize) -> Result<isize> {
        dev_info!(self.dev, "IOCTLing Rust KCounter\n");

        let size = _IOC_SIZE(cmd);

        match cmd {
            READ_VALUE => self.respond(UserSlice::new(arg, size).writer())?,
            // RUST_MISC_DEV_SET_VALUE => me.set_value(UserSlice::new(arg, size).reader())?,
            // RUST_MISC_DEV_HELLO => me.hello()?,
            _ => {
                dev_err!(me.dev, "-> IOCTL not recognised: {}\n", cmd);
                return Err(ENOTTY);
            }
        };

        Ok(0)
    }
}

#[pinned_drop]
impl PinnedDrop for KCounterDevice {
    fn drop(self: Pin<&mut Self>) {
        pr_info!("Exiting Rust KCounterDevice\n");
    }
}
