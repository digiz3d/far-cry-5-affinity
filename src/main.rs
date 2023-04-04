extern crate winapi;

use std::mem;

use winapi::shared::minwindef::{DWORD, FALSE};
use winapi::um::handleapi::CloseHandle;
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::tlhelp32::{
    CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS,
};
use winapi::um::winbase::SetProcessAffinityMask;
use winapi::um::winnt::{PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SET_INFORMATION};

fn main() {
    let process_name = String::from("FarCry5.exe");
    let affinity_mask: u32 = sum_consecutive_nums_bitmask(16, 32);

    let process_id = get_process_id_by_name(process_name).unwrap();

    let process_handle = unsafe {
        OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SET_INFORMATION,
            FALSE,
            process_id,
        )
    };

    if !process_handle.is_null() {
        let result = unsafe { SetProcessAffinityMask(process_handle, affinity_mask) };

        if result == 0 {
            println!("Failed to set process affinity");
        }

        unsafe {
            CloseHandle(process_handle);
        }
    } else {
        println!("Failed to open process");
    }
}

fn sum_consecutive_nums_bitmask(start: u32, end: u32) -> u32 {
    let num_bits = (end - start + 1) as usize;
    let mask = (1 << num_bits) - 1;
    mask << start
}

fn get_process_id_by_name(input_process_name: String) -> Option<DWORD> {
    let mut process_id: DWORD = 0;

    let handle = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };

    if handle == winapi::um::handleapi::INVALID_HANDLE_VALUE {
        return None;
    }

    let mut entry: PROCESSENTRY32 = unsafe { mem::zeroed() };
    entry.dwSize = mem::size_of::<PROCESSENTRY32>() as u32;

    let mut result = unsafe { Process32First(handle, &mut entry) };
    while result != 0 {
        let process_name = {
            let name_i8 = entry.szExeFile.as_ref();

            let input_slice: &[i8] = unsafe {
                let l = name_i8.len();
                std::slice::from_raw_parts(name_i8.as_ptr() as *const i8, l)
            };

            let transmuted_input = unsafe { std::mem::transmute::<&[i8], &[u8]>(input_slice) };

            transmuted_input
        };

        let fixed_found_process_name = {
            let string_slice = std::str::from_utf8(process_name).unwrap();
            String::from(string_slice)
        };

        if fixed_found_process_name.starts_with(input_process_name.as_str()) {
            process_id = entry.th32ProcessID;
            break;
        }

        result = unsafe { Process32Next(handle, &mut entry) };
    }

    unsafe {
        CloseHandle(handle);
    }

    if process_id == 0 {
        None
    } else {
        Some(process_id)
    }
}
