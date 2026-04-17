#![cfg(windows)]

use std::ffi::CString;
use std::io;

use windows::Win32::Foundation::{HWND, INVALID_HANDLE_VALUE, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::DataExchange::COPYDATASTRUCT;
use windows::Win32::System::Memory::{
    CreateFileMappingA, FILE_MAP_ALL_ACCESS, MapViewOfFile, PAGE_READWRITE, UnmapViewOfFile,
};
use windows::Win32::UI::WindowsAndMessaging::{FindWindowA, SendMessageA, WM_COPYDATA};
use windows::core::PCSTR;

use crate::agent::AgentBackend;

const AGENT_COPYDATA_ID: usize = 0x804e50ba;
const AGENT_MAX_MESSAGE_LENGTH: usize = 8192;

pub struct PageantBackend {
    hwnd: HWND,
}

impl PageantBackend {
    pub fn connect() -> io::Result<Self> {
        let class_name = c"Pageant";
        let hwnd = unsafe {
            FindWindowA(
                PCSTR(class_name.as_ptr() as *const u8),
                PCSTR(class_name.as_ptr() as *const u8),
            )
        }
        .map_err(to_io_error)?;
        Ok(Self { hwnd })
    }
}

impl AgentBackend for PageantBackend {
    fn roundtrip(&mut self, request: &[u8]) -> io::Result<Vec<u8>> {
        if request.len() > AGENT_MAX_MESSAGE_LENGTH {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Pageant request too large: {} bytes", request.len()),
            ));
        }

        let map_name = format!("WSLPageantRequest{:x}", std::process::id());
        let map_name = CString::new(map_name)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid map name"))?;

        let file_map = unsafe {
            CreateFileMappingA(
                INVALID_HANDLE_VALUE,
                None,
                PAGE_READWRITE,
                0,
                AGENT_MAX_MESSAGE_LENGTH as u32,
                PCSTR(map_name.as_ptr() as *const u8),
            )
        }
        .map_err(to_io_error)?;

        let shared_memory =
            unsafe { MapViewOfFile(file_map, FILE_MAP_ALL_ACCESS, 0, 0, AGENT_MAX_MESSAGE_LENGTH) };
        if shared_memory.Value.is_null() {
            return Err(io::Error::last_os_error());
        }

        let result = (|| {
            let shared = unsafe {
                std::slice::from_raw_parts_mut(
                    shared_memory.Value.cast::<u8>(),
                    AGENT_MAX_MESSAGE_LENGTH,
                )
            };
            shared[..request.len()].copy_from_slice(request);

            let mut map_name_with_nul = map_name.as_bytes_with_nul().to_vec();
            let copy_data = COPYDATASTRUCT {
                dwData: AGENT_COPYDATA_ID,
                cbData: map_name_with_nul.len() as u32,
                lpData: map_name_with_nul.as_mut_ptr() as *mut _,
            };

            let response = unsafe {
                SendMessageA(
                    self.hwnd,
                    WM_COPYDATA,
                    WPARAM(0),
                    LPARAM((&copy_data as *const COPYDATASTRUCT).cast::<core::ffi::c_void>() as isize),
                )
            };
            if response == LRESULT(0) {
                return Err(io::Error::other("Pageant WM_COPYDATA failed"));
            }

            let response_len = u32::from_be_bytes(shared[..4].try_into().unwrap()) as usize + 4;
            if response_len > AGENT_MAX_MESSAGE_LENGTH {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Pageant response too large: {response_len} bytes"),
                ));
            }

            Ok(shared[..response_len].to_vec())
        })();

        unsafe {
            UnmapViewOfFile(shared_memory).map_err(to_io_error)?;
        }

        result
    }
}

fn to_io_error(err: windows::core::Error) -> io::Error {
    io::Error::new(io::ErrorKind::Other, err)
}
