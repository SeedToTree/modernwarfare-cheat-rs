use memlib::memory::*;
use log::*;
use std::num::Wrapping;

use super::offsets;
use crate::sdk::structs::{refdef_t};

#[repr(C)]
#[derive(Debug, Default)]
pub struct refdef_key_struct {
    pub ref0: u32,
    pub ref1: u32,
    pub ref2: u32
}

/*
pub fn get_refdef_pointer(game_base_address: Address) -> Result<Pointer<refdef_t>> {
    let keys =

    trace!("Lower: {:X}, upper: {:X}", lower, upper);

    let result_address: u64 = upper << 32 | lower;

    trace!("Result_address: {:X}", result_address);

    // Ok(Pointer::new(result_address))
    Err("".into())
}
 */

pub fn get_refdef_pointer(game_base_address: Address) -> Result<Pointer<refdef_t>> {
    let crypt: refdef_key_struct = read_memory(game_base_address + offsets::REFDEF as u64);

    if crypt.ref0 == 0 && crypt.ref1 == 0 && crypt.ref2 == 0 {
        return Err("Read 0 for refdef key struct".into());
    }
    trace!("Found refdef_key_struct: {:?}", crypt);

    let lower: Wrapping<u64> = (Wrapping(crypt.ref0 as u64) ^ Wrapping(crypt.ref2 as u64 ^ (game_base_address + offsets::REFDEF as u64)) * Wrapping((crypt.ref2 as u64 ^ (game_base_address + offsets::REFDEF as u64)) + 2));
    let upper: Wrapping<u64> = (Wrapping(crypt.ref1 as u64) ^ Wrapping(crypt.ref2 as u64 ^ (game_base_address + offsets::REFDEF as u64 + 0x4)) * Wrapping((crypt.ref2 as u64 ^ (game_base_address + offsets::REFDEF as u64 + 0x4)) + 2));
    let ref_def_key = (upper.0 as u32 as u64) << 32 | (lower.0 as u32 as u64);

    let ref_def_pointer: Pointer<refdef_t> = Pointer::new(ref_def_key);

    trace!("ref_def_key: 0x{:X}", ref_def_key);
    trace!("ref_def_pointer.read() = {:?}", ref_def_pointer.read());
    if ref_def_pointer.read().height == 0 {
        return Err("Read an invalid refdef_t struct".into());
    }

    Ok(ref_def_pointer)
}

pub fn get_client_info_address(game_base_address: Address) -> Result<Address> {
    // Get the encrypted base address
    let encrypted_address: Address = read_memory(game_base_address + offsets::client_info::ENCRYPTED_PTR);
    if encrypted_address == 0 {
        return Err("Could not find encrypted base address for client_info".into());
    }
    trace!("Found encrypted client_info address: 0x{:X}", encrypted_address);

    // Get last_key
    let last_key = get_last_key_byteswap(game_base_address, offsets::client_info::REVERSED_ADDRESS, offsets::client_info::DISPLACEMENT)
        .ok_or_else(|| "Could not get last_key for decrypting the client_info base address")?;

    // Get not_peb
    let not_peb = get_not_peb();
    trace!("not_peb: 0x{:X}", not_peb);

    let mut encrypted_address = Wrapping(encrypted_address);
    let last_key = Wrapping(last_key);
    let not_peb = Wrapping(not_peb);
    let game_base_address = Wrapping(game_base_address);


    encrypted_address += Wrapping(0x449679754A46FEBA);
    //qword_140013FEAD98 + 0x449679754A46FEBA

    encrypted_address ^= Wrapping(0xE565D1B5A421081B);
    //(qword_140013FEAD98 + 0x449679754A46FEBA) ^ 0xE565D1B5A421081B

    let mut rcx = not_peb ^ encrypted_address;
    //(not_peb ^ encrypted_address)

    rcx *= last_key;
    //(rcx * reversed_address)

    let mut rdx = not_peb ^ rcx;
    //(not_peb ^ rcx)

    rdx *= Wrapping(0xE6F54D109BF19D87);
    //v7 = 0xE6F54D109BF19D87 * rdx;

    let mut rgx = (rdx >> 0x19) ^ rdx;
    encrypted_address = (rgx >> 0x32) ^ rgx;


    trace!("Found decrypted client_info address: 0x{:X}", encrypted_address.0);

    Ok(encrypted_address.0)
}

pub fn get_client_base_address(game_base_address: Address, client_info_address: Address) -> Result<Address> {
    let encrypted_address = read_memory(client_info_address + offsets::client_base::BASE_OFFSET);
    if encrypted_address == 0 {
        return Err("Could not find the encrypted client_info_base address".into());
    }
    trace!("Found encrypted client_info_base address: 0x{:X}", encrypted_address);

    let last_key = get_last_key_byteswap(game_base_address, offsets::client_base::BASE_REVERSED_ADDR, offsets::client_base::BASE_DISPLACEMENT)
        .ok_or_else(|| "Could not get last_key for decrypting client_info_base")?;

    let not_peb = get_not_peb();

    let mut encrypted_address = Wrapping(encrypted_address);
    let mut last_key = Wrapping(last_key);
    let not_peb = Wrapping(not_peb);
    let game_base_address = Wrapping(game_base_address);

    // Actual decryption

    encrypted_address += Wrapping(0x59E1B3C03F80C8D);
    encrypted_address ^= (encrypted_address >> 0xD);
    encrypted_address ^= (encrypted_address >> 0x1A);
    encrypted_address ^= (encrypted_address >> 0x34);
    encrypted_address += Wrapping(0x3DF642E2EC502CC4);
    encrypted_address *= (last_key * Wrapping(0x0B824469A3604169));

    trace!("Found decrypted client_info_base address: 0x{:X}", encrypted_address.0);

    Ok(encrypted_address.0)
}

pub fn get_bone_base_address(game_base_address: Address) -> Result<Address> {
    let encrypted_address = read_memory(game_base_address + offsets::bones::ENCRYPTED_PTR);
    if encrypted_address == 0 {
        return Err("Could not find the encrypted bone_base address".into());
    }
    trace!("Found encrypted bone_base address: 0x{:X}", encrypted_address);

    let last_key = get_last_key_byteswap(game_base_address, offsets::bones::REVERSED_ADDRESS, offsets::bones::DISPLACEMENT)
        .ok_or_else(|| "Could not get last_key for decrypting base_address")?;

    let not_peb = get_not_peb();

    let mut encrypted_address = Wrapping(encrypted_address);
    let last_key = Wrapping(last_key);
    let not_peb = Wrapping(not_peb);
    let game_base_address = Wrapping(game_base_address);


    encrypted_address ^= (encrypted_address >> 0x1E);
    encrypted_address ^= (encrypted_address >> 0x3C);
    encrypted_address -= not_peb;
    encrypted_address *= Wrapping(0x0F8C1AE84FC79FF);
    let mut rax = encrypted_address + Wrapping(0x3B5A731BEABDCDB8);
    rax ^= not_peb;
    rax -= Wrapping(0x2BAE3C661FD220ED);
    encrypted_address = last_key * rax;
    encrypted_address += Wrapping(0x696390902E76219);
    encrypted_address ^= Wrapping(0x0C7E065DF67BE4140);

    trace!("Found decrypted bone_base address: 0x{:X}", encrypted_address.0);

    Ok(encrypted_address.0)
}

fn get_not_peb() -> u64 {
    !get_process_info().peb_base_address
}

fn get_last_key(game_base_address: Address, reversed_address_offset: Address, displacement_offset: Address) -> Option<Address> {
    let reversed_address: Address = read_memory(game_base_address + reversed_address_offset);
    let last_key = read_memory(!reversed_address + displacement_offset);

    if last_key == 0 {
        return None;
    }

    Some(last_key)
}

fn get_last_key_byteswap(game_base_address: Address, reversed_address_offset: Address, displacement_offset: Address) -> Option<Address> {
    let reversed_address: Address = read_memory(game_base_address + reversed_address_offset);
    let last_key = read_memory(u64::from_be(reversed_address) + displacement_offset);

    if last_key == 0 {
        return None;
    }

    Some(last_key)
}
