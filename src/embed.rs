use std::env;
use std::{fs::{read, write}, process::exit, ptr};

use const_random::const_random;

use crate::interpreter;

#[repr(C)]
struct Marker {
    magic_pattern: [u8; 512],
    is_embedded: u8,
}

const fn new_marker() -> Marker {
    Marker{
        magic_pattern: const_random!([u8; 512]),
        is_embedded: 0,
    }
}

static EMBEDDED_MARKER: Marker = new_marker();

pub fn is_embedded() -> bool {
    let embedded_indicator = &EMBEDDED_MARKER.is_embedded as *const u8;
    unsafe {
        // Need to perform this trick to read the actual memory location.
        // Otherwise during compilation Rust does static analysis and assumes
        // this function always returns the same value.
        return ptr::read_volatile(embedded_indicator) != 0;
    }
}

fn magic_pattern() -> &'static[u8; 512] {
    &EMBEDDED_MARKER.magic_pattern
}

fn find_position(haystack: &Vec<u8>, needle: &[u8; 512]) -> Option<usize> {
    if haystack.len() < needle.len() {
        return None;
    }
    for i in 0..=haystack.len() - needle.len() {
        if &haystack[i..i + needle.len()] == needle.as_ref() {
            return Some(i);
        }
    }
    None
}

pub fn embed_file(compiled_script: String, output_binary: String) {
    let exe_path = env::current_exe().unwrap();
    let mut exe_contents = read(exe_path).unwrap();
    let pattern = magic_pattern();
    if let Some(pos) = find_position(&exe_contents, pattern) {
        exe_contents[pos+512] = 1;
        let mut compiled_content = read(compiled_script).unwrap();
        for i in 0..512 {
            exe_contents.push(pattern[i]);
        }
        exe_contents.append(&mut compiled_content);
        write(output_binary, exe_contents).unwrap();
    }
}

pub fn execute_embedded() {
    let exe_path = env::current_exe().unwrap();
    interpreter::set_absolute_path(exe_path.clone().to_str().unwrap().to_string());

    let exe_contents = read(exe_path).unwrap();
    let pattern = magic_pattern();
    let offset: usize = 512;
    let first_match = find_position(&exe_contents, pattern).unwrap();
    let remaining = &exe_contents[first_match+offset..].to_vec();
    let pos = find_position(remaining, pattern).unwrap();
    let compiled_content = &remaining[pos+offset..];
    
    if !interpreter::run_interpreter_from_bytecode(&compiled_content) {
        println!("Could not read embedded script");
        exit(1);
    }
}