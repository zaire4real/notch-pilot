//! Audio output devices — CoreAudio C API via hand-written FFI.
//!
//! No third-party binary (e.g. SwitchAudioSource) is bundled and no new
//! crates are added: CoreAudio and CoreFoundation are plain C
//! frameworks, so the few functions used here are declared directly
//! with `extern "C"` and linked with `#[link(kind = "framework")]`.
//! The non-macOS build is an explicit stub.
//!
//! Device identity: `id` is the CoreAudio device id (not stable across
//! reconnects); `uid` is the persistent device UID and is what the
//! settings store (`media.audio_output`). The frontend resolves a UID
//! back to an id through the list returned by `get_audio_outputs`.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct AudioOutput {
    /// CoreAudio device id (not stable across reconnects — the UID is).
    pub id: u32,
    pub name: String,
    /// Persistent device UID, used as the settings value.
    pub uid: String,
    pub is_default: bool,
}

#[tauri::command]
pub fn get_audio_outputs() -> Result<Vec<AudioOutput>, String> {
    #[cfg(target_os = "macos")]
    {
        macos::get_audio_outputs_macos()
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err("Audio output devices are only supported on macOS".to_string())
    }
}

#[tauri::command]
pub fn set_audio_output(device_id: u32) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        macos::set_audio_output_macos(device_id)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = device_id;
        Err("Audio output switching is only supported on macOS".to_string())
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use super::AudioOutput;
    use std::ffi::c_void;
    use std::os::raw::{c_char, c_uint};

    type OSStatus = i32;
    type AudioObjectID = u32;
    type CFStringRef = *const c_void;
    type CFIndex = isize;
    type Boolean = u8;

    #[repr(C)]
    struct AudioObjectPropertyAddress {
        m_selector: u32,
        m_scope: u32,
        m_element: u32,
    }

    /// FourCharCode from four ASCII bytes (big-endian), e.g.
    /// `fourcc(b"dev#")` is `kAudioHardwarePropertyDevices`.
    const fn fourcc(bytes: &[u8; 4]) -> u32 {
        u32::from_be_bytes(*bytes)
    }

    const SYSTEM_OBJECT: AudioObjectID = 1; // kAudioObjectSystemObject
    const ELEMENT_MAIN: u32 = 0; // kAudioObjectPropertyElementMain
    const SCOPE_GLOBAL: u32 = fourcc(b"glob"); // kAudioObjectPropertyScopeGlobal
    const SCOPE_OUTPUT: u32 = fourcc(b"outp"); // kAudioObjectPropertyScopeOutput
    const PROP_DEVICES: u32 = fourcc(b"dev#"); // kAudioHardwarePropertyDevices
    const PROP_DEFAULT_OUTPUT: u32 = fourcc(b"dOut"); // kAudioHardwarePropertyDefaultOutputDevice
    const PROP_DEVICE_NAME: u32 = fourcc(b"name"); // kAudioDevicePropertyDeviceNameCFString
    const PROP_DEVICE_UID: u32 = fourcc(b"uid "); // kAudioDevicePropertyDeviceUID
    const PROP_STREAMS: u32 = fourcc(b"strm"); // kAudioDevicePropertyStreams
    const CF_STRING_ENCODING_UTF8: u32 = 0x0800_0100; // kCFStringEncodingUTF8

    #[link(name = "CoreAudio", kind = "framework")]
    extern "C" {
        fn AudioObjectGetPropertyDataSize(
            in_object_id: AudioObjectID,
            in_address: *const AudioObjectPropertyAddress,
            in_qualifier_data_size: c_uint,
            in_qualifier_data: *const c_void,
            out_data_size: *mut c_uint,
        ) -> OSStatus;
        fn AudioObjectGetPropertyData(
            in_object_id: AudioObjectID,
            in_address: *const AudioObjectPropertyAddress,
            in_qualifier_data_size: c_uint,
            in_qualifier_data: *const c_void,
            io_data_size: *mut c_uint,
            out_data: *mut c_void,
        ) -> OSStatus;
        fn AudioObjectSetPropertyData(
            in_object_id: AudioObjectID,
            in_address: *const AudioObjectPropertyAddress,
            in_qualifier_data_size: c_uint,
            in_qualifier_data: *const c_void,
            in_data_size: c_uint,
            in_data: *const c_void,
        ) -> OSStatus;
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFStringGetCString(
            the_string: CFStringRef,
            buffer: *mut c_char,
            buffer_size: CFIndex,
            encoding: u32,
        ) -> Boolean;
        fn CFRelease(cf: *const c_void);
    }

    fn property_data(
        object: AudioObjectID,
        selector: u32,
        scope: u32,
        out: *mut c_void,
        size: &mut c_uint,
    ) -> OSStatus {
        let address = AudioObjectPropertyAddress {
            m_selector: selector,
            m_scope: scope,
            m_element: ELEMENT_MAIN,
        };
        unsafe { AudioObjectGetPropertyData(object, &address, 0, std::ptr::null(), size, out) }
    }

    /// Read a CFString property; the caller-owned reference is
    /// released. Returns an empty string on any failure.
    fn cfstring_property(object: AudioObjectID, selector: u32) -> String {
        let mut string_ref: CFStringRef = std::ptr::null();
        let mut size = std::mem::size_of::<CFStringRef>() as c_uint;
        let status = property_data(
            object,
            selector,
            SCOPE_GLOBAL,
            &mut string_ref as *mut CFStringRef as *mut c_void,
            &mut size,
        );
        if status != 0 || string_ref.is_null() {
            return String::new();
        }
        unsafe {
            let mut buf = vec![0u8; 512];
            let ok = CFStringGetCString(
                string_ref,
                buf.as_mut_ptr() as *mut c_char,
                buf.len() as CFIndex,
                CF_STRING_ENCODING_UTF8,
            );
            CFRelease(string_ref);
            if ok == 0 {
                return String::new();
            }
            let len = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
            String::from_utf8_lossy(&buf[..len]).to_string()
        }
    }

    /// True when the device has at least one output stream. Input-only
    /// devices (microphones) report zero output streams and are
    /// filtered out of the selector.
    fn has_output_streams(device: AudioObjectID) -> bool {
        let address = AudioObjectPropertyAddress {
            m_selector: PROP_STREAMS,
            m_scope: SCOPE_OUTPUT,
            m_element: ELEMENT_MAIN,
        };
        let mut size: c_uint = 0;
        let status = unsafe {
            AudioObjectGetPropertyDataSize(device, &address, 0, std::ptr::null(), &mut size)
        };
        status == 0 && size > 0
    }

    fn default_output_device() -> Option<AudioObjectID> {
        let mut device: AudioObjectID = 0;
        let mut size = std::mem::size_of::<AudioObjectID>() as c_uint;
        let status = property_data(
            SYSTEM_OBJECT,
            PROP_DEFAULT_OUTPUT,
            SCOPE_GLOBAL,
            &mut device as *mut AudioObjectID as *mut c_void,
            &mut size,
        );
        if status != 0 {
            None
        } else {
            Some(device)
        }
    }

    pub fn get_audio_outputs_macos() -> Result<Vec<AudioOutput>, String> {
        // 1. List all device ids.
        let address = AudioObjectPropertyAddress {
            m_selector: PROP_DEVICES,
            m_scope: SCOPE_GLOBAL,
            m_element: ELEMENT_MAIN,
        };
        let mut size: c_uint = 0;
        let status = unsafe {
            AudioObjectGetPropertyDataSize(SYSTEM_OBJECT, &address, 0, std::ptr::null(), &mut size)
        };
        if status != 0 {
            return Err(format!("could not list audio devices (OSStatus {status})"));
        }
        let count = (size as usize) / std::mem::size_of::<AudioObjectID>();
        let mut ids = vec![0u32; count];
        let mut size2 = size;
        let status = unsafe {
            AudioObjectGetPropertyData(
                SYSTEM_OBJECT,
                &address,
                0,
                std::ptr::null(),
                &mut size2,
                ids.as_mut_ptr() as *mut c_void,
            )
        };
        if status != 0 {
            return Err(format!(
                "could not read audio devices (OSStatus {status})"
            ));
        }
        let default = default_output_device();
        // 2. Keep output devices only, with name + UID.
        let mut outputs = Vec::new();
        for id in ids {
            if !has_output_streams(id) {
                continue;
            }
            let uid = cfstring_property(id, PROP_DEVICE_UID);
            let name = cfstring_property(id, PROP_DEVICE_NAME);
            outputs.push(AudioOutput {
                id,
                name: if name.is_empty() { uid.clone() } else { name },
                uid,
                is_default: default == Some(id),
            });
        }
        Ok(outputs)
    }

    pub fn set_audio_output_macos(device_id: u32) -> Result<(), String> {
        let address = AudioObjectPropertyAddress {
            m_selector: PROP_DEFAULT_OUTPUT,
            m_scope: SCOPE_GLOBAL,
            m_element: ELEMENT_MAIN,
        };
        let status = unsafe {
            AudioObjectSetPropertyData(
                SYSTEM_OBJECT,
                &address,
                0,
                std::ptr::null(),
                std::mem::size_of::<AudioObjectID>() as c_uint,
                &device_id as *const AudioObjectID as *const c_void,
            )
        };
        if status != 0 {
            return Err(format!(
                "could not set audio output device (OSStatus {status})"
            ));
        }
        Ok(())
    }
}
