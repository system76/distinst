use distinst::{self, Disks, UpgradeError, UpgradeEvent};
use distinst::auto::RecoveryOption;
use super::{DistinstDisks, DistinstRecoveryOption};
use libc;
use std::ptr;

#[repr(C)]
pub struct DistinstUpgradeEvent {
    tag: DISTINST_UPGRADE_TAG,
    percent: libc::uint8_t,
    str1: *const libc::uint8_t,
    str1_length1: libc::size_t,
    str2: *const libc::uint8_t,
    str2_length1: libc::size_t,
    str3: *const libc::uint8_t,
    str3_length1: libc::size_t,
}

#[repr(C)]
pub enum DISTINST_UPGRADE_TAG {
    ATTEMPTING_REPAIR,
    ATTEMPTING_UPGRADE,
    DPKG_INFO,
    DPKG_ERR,
    UPGRADE_INFO,
    UPGRADE_ERR,
    PACKAGE_PROCESSING,
    PACKAGE_PROGRESS,
    PACKAGE_SETTING_UP,
    PACKAGE_UNPACKING,
    RESUMING_UPGRADE
}

impl From<UpgradeEvent<'_>> for DistinstUpgradeEvent {
    fn from(event: UpgradeEvent) -> Self {
        let mut c_event = DistinstUpgradeEvent {
            tag: DISTINST_UPGRADE_TAG::ATTEMPTING_REPAIR,
            percent: 0,
            str1: ptr::null(),
            str1_length1: 0,
            str2: ptr::null(),
            str2_length1: 0,
            str3: ptr::null(),
            str3_length1: 0,
        };

        fn set_str(data: &mut *const libc::uint8_t, len: &mut libc::size_t, message: &str) {
            let message = message.as_bytes();
            *data = message.as_ptr();
            *len = message.len();
        }

        fn set_str1(event: &mut DistinstUpgradeEvent, message: &str) {
            set_str(&mut event.str1, &mut event.str1_length1, message);
        }

        fn set_str2(event: &mut DistinstUpgradeEvent, message: &str) {
            set_str(&mut event.str2, &mut event.str2_length1, message);
        }

        fn set_str3(event: &mut DistinstUpgradeEvent, message: &str) {
            set_str(&mut event.str3, &mut event.str3_length1, message);
        }

        match event {
            UpgradeEvent::AttemptingRepair => (),
            UpgradeEvent::AttemptingUpgrade => {
                c_event.tag = DISTINST_UPGRADE_TAG::ATTEMPTING_UPGRADE;
            }
            UpgradeEvent::DpkgInfo(info) => {
                c_event.tag = DISTINST_UPGRADE_TAG::DPKG_INFO;
                set_str1(&mut c_event, info);
            }
            UpgradeEvent::DpkgErr(info) => {
                c_event.tag = DISTINST_UPGRADE_TAG::DPKG_ERR;
                set_str1(&mut c_event, info);
            }
            UpgradeEvent::UpgradeInfo(info) => {
                c_event.tag = DISTINST_UPGRADE_TAG::UPGRADE_INFO;
                set_str1(&mut c_event, info);
            }
            UpgradeEvent::UpgradeErr(info) => {
                c_event.tag = DISTINST_UPGRADE_TAG::UPGRADE_ERR;
                set_str1(&mut c_event, info);
            }
            UpgradeEvent::PackageProcessing(package) => {
                c_event.tag = DISTINST_UPGRADE_TAG::PACKAGE_PROCESSING;
                set_str1(&mut c_event, package);
            }
            UpgradeEvent::PackageProgress(percent) => {
                c_event.tag = DISTINST_UPGRADE_TAG::PACKAGE_PROGRESS;
                c_event.percent = percent;
            }
            UpgradeEvent::PackageSettingUp(package) => {
                c_event.tag = DISTINST_UPGRADE_TAG::PACKAGE_SETTING_UP;
                set_str1(&mut c_event, package);
            }
            UpgradeEvent::PackageUnpacking { package, version, over } => {
                c_event.tag = DISTINST_UPGRADE_TAG::PACKAGE_UNPACKING;
                set_str1(&mut c_event, package);
                set_str2(&mut c_event, version);
                set_str3(&mut c_event, over);
            }
            UpgradeEvent::ResumingUpgrade => {
                c_event.tag = DISTINST_UPGRADE_TAG::RESUMING_UPGRADE;
            }
        }

        c_event
    }
}

pub type DistinstUpgradeEventCallback =
    extern "C" fn(event: DistinstUpgradeEvent, user_data: *mut libc::c_void);

pub type DistinstUpgradeRepairCallback =
    extern "C" fn(target: *const libc::uint8_t, user_data: *mut libc::c_void);

#[no_mangle]
pub unsafe extern "C" fn distinst_upgrade(
    disks: *mut DistinstDisks,
    option: *const DistinstRecoveryOption,
    event_cb: DistinstUpgradeEventCallback,
    cb_data: *mut libc::c_void,
) -> libc::c_int {
    let result = distinst::upgrade(
        &mut *(disks as *mut Disks),
        &*(option as *const RecoveryOption),
        move |event| event_cb(DistinstUpgradeEvent::from(event), cb_data),
    );

    match result {
        Ok(()) => 0,
        Err(why) => {
            error!("upgrade failed: {}", why);
            -1
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn distinst_resume_upgrade(
    disks: *const DistinstDisks,
    event_cb: DistinstUpgradeEventCallback,
    cb_data1: *mut libc::c_void,
    repair_cb: DistinstUpgradeRepairCallback,
    cb_data2: *mut libc::c_void,
) -> libc::c_int {
    assert!(!disks.is_null(), "disks parameter should not be null");

    let result = distinst::resume_upgrade(
        &*(disks as *const Disks),
        move |event| event_cb(DistinstUpgradeEvent::from(event), cb_data1),
        move |target| repair_cb(target.as_bytes().as_ptr(), cb_data2)
    );

    match result {
        Ok(()) => 0,
        Err(why) => {
            error!("resumed upgrade failed: {}", why);
            -1
        }
    }
}
