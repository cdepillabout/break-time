//! Queries for whether capture devices are in use by *some* process: the
//! camera via `CoreMediaIO`, the microphone via `CoreAudio`. Both ask the
//! "is running somewhere" device property — reading it needs no TCC permission
//! (nothing is captured), which is why meeting detection on macOS is built on
//! these instead of window titles (whose enumeration requires Screen
//! Recording).
//!
//! `Err(())` means the device *list* could not be read (treated by the caller
//! as "do not break right now", like every other plugin failure); a failed
//! read on an individual device just skips that device.

#![allow(unsafe_code)]

use std::ffi::c_void;
use std::ptr::NonNull;

use objc2_core_audio::{
    kAudioDevicePropertyDeviceIsRunningSomewhere,
    kAudioDevicePropertyStreamConfiguration, kAudioHardwarePropertyDevices,
    kAudioObjectPropertyElementMain, kAudioObjectPropertyScopeGlobal,
    kAudioObjectPropertyScopeInput, kAudioObjectSystemObject,
    AudioObjectGetPropertyData, AudioObjectGetPropertyDataSize,
    AudioObjectPropertyAddress,
};
use objc2_core_media_io::{
    kCMIODevicePropertyDeviceIsRunningSomewhere, kCMIOHardwarePropertyDevices,
    kCMIOObjectPropertyElementMain, kCMIOObjectPropertyScopeGlobal,
    kCMIOObjectSystemObject, CMIOObjectGetPropertyData,
    CMIOObjectGetPropertyDataSize, CMIOObjectPropertyAddress,
};

// size_of::<u32>() is 4; the cast cannot truncate.
#[allow(clippy::cast_possible_truncation)]
const U32_SIZE: u32 = std::mem::size_of::<u32>() as u32;

/// Is any camera in use by some process (`CoreMediaIO`
/// `kCMIODevicePropertyDeviceIsRunningSomewhere`)?
pub fn camera_running() -> Result<bool, ()> {
    let devices_addr = CMIOObjectPropertyAddress {
        mSelector: kCMIOHardwarePropertyDevices,
        mScope: kCMIOObjectPropertyScopeGlobal,
        mElement: kCMIOObjectPropertyElementMain,
    };

    let mut size: u32 = 0;
    let status = unsafe {
        CMIOObjectGetPropertyDataSize(
            kCMIOObjectSystemObject,
            &raw const devices_addr,
            0,
            std::ptr::null(),
            &raw mut size,
        )
    };
    if status != 0 {
        return Err(());
    }

    let mut devices = vec![0u32; size as usize / U32_SIZE as usize];
    let mut used: u32 = 0;
    let status = unsafe {
        CMIOObjectGetPropertyData(
            kCMIOObjectSystemObject,
            &raw const devices_addr,
            0,
            std::ptr::null(),
            size,
            &raw mut used,
            devices.as_mut_ptr().cast::<c_void>(),
        )
    };
    if status != 0 {
        return Err(());
    }

    let running_addr = CMIOObjectPropertyAddress {
        mSelector: kCMIODevicePropertyDeviceIsRunningSomewhere,
        mScope: kCMIOObjectPropertyScopeGlobal,
        mElement: kCMIOObjectPropertyElementMain,
    };
    for dev in devices {
        let mut running: u32 = 0;
        let mut used: u32 = 0;
        let status = unsafe {
            CMIOObjectGetPropertyData(
                dev,
                &raw const running_addr,
                0,
                std::ptr::null(),
                U32_SIZE,
                &raw mut used,
                (&raw mut running).cast::<c_void>(),
            )
        };
        if status == 0 && running != 0 {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Is any *input* audio device (microphone) in use by some process
/// (`CoreAudio` `kAudioDevicePropertyDeviceIsRunningSomewhere`, input scope)?
pub fn mic_running() -> Result<bool, ()> {
    let devices_addr = AudioObjectPropertyAddress {
        mSelector: kAudioHardwarePropertyDevices,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMain,
    };

    // The generated binding types `kAudioObjectSystemObject` as `c_int` even
    // though object ids are `u32`.
    #[allow(clippy::cast_sign_loss)]
    let system_object = kAudioObjectSystemObject as u32;

    let mut size: u32 = 0;
    let status = unsafe {
        AudioObjectGetPropertyDataSize(
            system_object,
            NonNull::from(&devices_addr),
            0,
            std::ptr::null(),
            NonNull::from(&mut size),
        )
    };
    if status != 0 {
        return Err(());
    }

    let mut devices = vec![0u32; size as usize / U32_SIZE as usize];
    let status = unsafe {
        AudioObjectGetPropertyData(
            system_object,
            NonNull::from(&devices_addr),
            0,
            std::ptr::null(),
            NonNull::from(&mut size),
            NonNull::new(devices.as_mut_ptr().cast::<c_void>()).ok_or(())?,
        )
    };
    if status != 0 {
        return Err(());
    }

    for dev in devices {
        // Only devices with input streams count as microphones. An
        // AudioBufferList with no buffers is 4 bytes (mNumberBuffers alone);
        // more than that means the device has input streams.
        let config_addr = AudioObjectPropertyAddress {
            mSelector: kAudioDevicePropertyStreamConfiguration,
            mScope: kAudioObjectPropertyScopeInput,
            mElement: kAudioObjectPropertyElementMain,
        };
        let mut config_size: u32 = 0;
        let status = unsafe {
            AudioObjectGetPropertyDataSize(
                dev,
                NonNull::from(&config_addr),
                0,
                std::ptr::null(),
                NonNull::from(&mut config_size),
            )
        };
        if status != 0 || config_size <= 4 {
            continue;
        }

        let running_addr = AudioObjectPropertyAddress {
            mSelector: kAudioDevicePropertyDeviceIsRunningSomewhere,
            mScope: kAudioObjectPropertyScopeInput,
            mElement: kAudioObjectPropertyElementMain,
        };
        let mut running: u32 = 0;
        let mut running_size = U32_SIZE;
        let status = unsafe {
            AudioObjectGetPropertyData(
                dev,
                NonNull::from(&running_addr),
                0,
                std::ptr::null(),
                NonNull::from(&mut running_size),
                NonNull::from(&mut running).cast::<c_void>(),
            )
        };
        if status == 0 && running != 0 {
            return Ok(true);
        }
    }
    Ok(false)
}
