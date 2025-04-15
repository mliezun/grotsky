use std::env;
use std::{fs::{read, write}, process::exit, ptr};

use crate::interpreter;

// Add compile-time random byte generation functions
const fn simple_compile_time_lcg(seed: u64) -> u64 {
    // Parameters from Numerical Recipes (via Wikipedia)
    const A: u64 = 1664525;
    const C: u64 = 1013904223;
    // M is 2^64 implicitly due to wrapping arithmetic
    A.wrapping_mul(seed).wrapping_add(C)
}

const fn compile_time_random_bytes<const N: usize>() -> [u8; N] {
    // Simple FNV-1a hash for compile-time seeding based on file path
    const fn str_fnv1a_hash(s: &str) -> u64 {
        let mut hash: u64 = 0xcbf29ce484222325; // FNV offset basis
        let prime: u64 = 0x100000001b3; // FNV prime
        let bytes = s.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            hash ^= bytes[i] as u64;
            hash = hash.wrapping_mul(prime);
            i += 1;
        }
        hash
    }

    // Seed the LCG with the hash of the current file's path
    const SEED: u64 = str_fnv1a_hash(file!());

    let mut bytes = [0u8; N];
    let mut state = SEED;
    let mut i = 0;
    // Generate N bytes using the LCG
    while i < N {
        state = simple_compile_time_lcg(state);
        // Fill bytes using parts of the 64-bit state
        // This extracts 8 bytes per LCG iteration
        let state_bytes = state.to_le_bytes();
        let mut byte_idx = 0;
        while byte_idx < 8 && i < N {
             bytes[i] = state_bytes[byte_idx];
             i += 1;
             byte_idx += 1;
        }
    }
    bytes
}

#[repr(C)]
struct Marker {
    magic_pattern: [u8; 512],
    is_embedded: u8,
}

const fn new_marker() -> Marker {
    Marker{
        // Use the new function instead of const_random! macro
        magic_pattern: compile_time_random_bytes::<512>(),
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