//! Permission-free capture activity queries for macOS meeting detection.
//! Cameras use `CoreMediaIO` device activity. Microphones use `CoreAudio` process
//! activity to exclude known dictation apps, with device activity as a fallback
//! when process attribution is unavailable. Nothing is captured.

#![allow(unsafe_code)]

use std::ffi::c_void;
use std::ptr::NonNull;

use objc2_core_audio::{
    kAudioDevicePropertyDeviceIsRunningSomewhere,
    kAudioDevicePropertyStreamConfiguration, kAudioHardwarePropertyDevices,
    kAudioHardwarePropertyProcessObjectList, kAudioObjectPropertyElementMain,
    kAudioObjectPropertyScopeGlobal, kAudioObjectPropertyScopeInput,
    kAudioObjectSystemObject, kAudioProcessPropertyBundleID,
    kAudioProcessPropertyIsRunningInput, kAudioProcessPropertyPID,
    AudioObjectGetPropertyData, AudioObjectGetPropertyDataSize,
    AudioObjectPropertyAddress,
};
use objc2_core_foundation::{CFRetained, CFString};
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
        println!("camera_mic: camera enumeration failed status={status}");
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
        println!("camera_mic: camera enumeration failed status={status}");
        return Err(());
    }

    devices.truncate((used / U32_SIZE) as usize);
    println!("camera_mic: enumerated {} cameras", devices.len());
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
        println!(
            "camera_mic: camera device={dev} running={running} status={status}"
        );
        if status == 0 && running != 0 {
            return Ok(true);
        }
    }
    Ok(false)
}

// Exact bundle IDs only: ignoring a browser would also ignore its meetings.
const IGNORED_MICROPHONE_APPS: &[&str] =
    &["com.electron.wispr-flow", "com.electron.wispr-flow.helper"];

/// Is a microphone in use by a process outside the built-in ignore list?
pub fn mic_running() -> Result<bool, ()> {
    match microphone_processes() {
        Ok(processes) => {
            let blocks =
                processes_block_break(processes.iter().map(Option::as_deref));
            println!("camera_mic: microphone path=per-process active_clients={} blocks_break={blocks}", processes.len());
            Ok(blocks)
        }
        Err(()) => {
            println!("camera_mic: microphone path=device-fallback reason=process-query-failed; app exclusions unavailable");
            let result = mic_device_running();
            println!(
                "camera_mic: microphone device-fallback result={result:?}"
            );
            result
        }
    }
}

fn processes_block_break<'a>(
    processes: impl IntoIterator<Item = Option<&'a str>>,
) -> bool {
    processes.into_iter().any(|bundle_id| match bundle_id {
        Some(id) => !IGNORED_MICROPHONE_APPS.contains(&id),
        None => true,
    })
}

const fn audio_address(selector: u32) -> AudioObjectPropertyAddress {
    AudioObjectPropertyAddress {
        mSelector: selector,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMain,
    }
}

// The caller must provide the exact ABI type for this fixed-size property.
unsafe fn read_audio_property<T>(
    object: u32,
    selector: u32,
    value: &mut T,
) -> Result<(), ()> {
    let address = audio_address(selector);
    let mut size = u32::try_from(std::mem::size_of::<T>()).map_err(|_| ())?;
    let status = unsafe {
        AudioObjectGetPropertyData(
            object,
            NonNull::from(&address),
            0,
            std::ptr::null(),
            NonNull::from(&mut size),
            NonNull::from(value).cast::<c_void>(),
        )
    };
    if status != 0 || size as usize != std::mem::size_of::<T>() {
        println!("camera_mic: CoreAudio property failed object={object} selector={selector:#010x} status={status} returned_bytes={size} expected_bytes={}", std::mem::size_of::<T>());
        return Err(());
    }
    Ok(())
}

/// Bundle IDs of active input clients; an unidentified client must still veto.
fn microphone_processes() -> Result<Vec<Option<String>>, ()> {
    let address = audio_address(kAudioHardwarePropertyProcessObjectList);
    #[allow(clippy::cast_sign_loss)]
    let system = kAudioObjectSystemObject as u32;
    let mut size = 0;
    let status = unsafe {
        AudioObjectGetPropertyDataSize(
            system,
            NonNull::from(&address),
            0,
            std::ptr::null(),
            NonNull::from(&mut size),
        )
    };
    if status != 0 || size % U32_SIZE != 0 {
        println!("camera_mic: CoreAudio process-list-size failed status={status} bytes={size}");
        return Err(());
    }
    if size == 0 {
        println!("camera_mic: CoreAudio process list is empty");
        return Ok(Vec::new());
    }
    let mut processes = vec![0u32; (size / U32_SIZE) as usize];
    let status = unsafe {
        AudioObjectGetPropertyData(
            system,
            NonNull::from(&address),
            0,
            std::ptr::null(),
            NonNull::from(&mut size),
            NonNull::new(processes.as_mut_ptr().cast::<c_void>()).ok_or(())?,
        )
    };
    if status != 0 || size % U32_SIZE != 0 {
        println!("camera_mic: CoreAudio process-list-read failed status={status} bytes={size}");
        return Err(());
    }
    processes.truncate((size / U32_SIZE) as usize);
    println!(
        "camera_mic: CoreAudio enumerated {} audio clients",
        processes.len()
    );
    let mut active = Vec::new();
    for process in processes {
        let mut running = 0u32;
        unsafe {
            read_audio_property(
                process,
                kAudioProcessPropertyIsRunningInput,
                &mut running,
            )?;
        }
        if running == 0 {
            continue;
        }
        let mut bundle_id: *mut CFString = std::ptr::null_mut();
        let result = unsafe {
            read_audio_property(
                process,
                kAudioProcessPropertyBundleID,
                &mut bundle_id,
            )
        };
        // CoreAudio transfers ownership of the returned CFString to us.
        let bundle_id = NonNull::new(bundle_id)
            .map(|ptr| unsafe { CFRetained::from_raw(ptr) });
        let bundle_id = if result.is_ok() {
            bundle_id.map(|id| id.to_string())
        } else {
            None
        };
        let mut pid = 0i32;
        let pid = unsafe {
            read_audio_property(process, kAudioProcessPropertyPID, &mut pid)
        }
        .ok()
        .map(|()| pid);
        let blocks = processes_block_break([bundle_id.as_deref()]);
        println!("camera_mic: active microphone client object={process} pid={pid:?} bundle_id={bundle_id:?} ignored={} blocks_break={blocks}", !blocks);
        active.push(bundle_id);
    }
    Ok(active)
}

/// Is any *input* audio device (microphone) in use by some process
/// (`CoreAudio` `kAudioDevicePropertyDeviceIsRunningSomewhere`, input scope)?
fn mic_device_running() -> Result<bool, ()> {
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
        println!(
            "camera_mic: fallback device enumeration failed status={status}"
        );
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
        println!(
            "camera_mic: fallback device enumeration failed status={status}"
        );
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
        println!("camera_mic: fallback device={dev} input_config_bytes={config_size} status={status}");
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
        println!("camera_mic: fallback input device={dev} running={running} status={status}");
        if status == 0 && running != 0 {
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::processes_block_break;

    #[test]
    #[ignore = "manual read-only diagnostic of current camera and microphone activity"]
    fn inspect_capture_activity() {
        println!(
            "camera_mic: diagnostic camera result={:?}",
            super::camera_running()
        );
        println!(
            "camera_mic: diagnostic microphone result={:?}",
            super::mic_running()
        );
    }

    #[test]
    fn dictation_alone_does_not_block_breaks() {
        assert!(!processes_block_break([Some("com.electron.wispr-flow")]));
        assert!(!processes_block_break([]));
    }

    #[test]
    fn wispr_recording_helper_does_not_block_breaks() {
        assert!(!processes_block_break([Some(
            "com.electron.wispr-flow.helper"
        )]));
        assert!(processes_block_break([
            Some("com.electron.wispr-flow.helper"),
            Some("com.google.Chrome"),
        ]));
        assert!(processes_block_break([Some(
            "com.electron.wispr-flow.helper.other"
        )]));
    }

    #[test]
    fn meeting_alongside_dictation_still_blocks_breaks() {
        for meeting in ["com.google.Chrome", "us.zoom.xos"] {
            assert!(processes_block_break([
                Some("com.electron.wispr-flow"),
                Some(meeting),
            ]));
        }
    }

    #[test]
    fn unidentified_or_similarly_named_clients_still_block_breaks() {
        for client in [None, Some(""), Some("com.electron.wispr-flow.other")] {
            assert!(processes_block_break([
                Some("com.electron.wispr-flow"),
                client,
            ]));
        }
    }
}
