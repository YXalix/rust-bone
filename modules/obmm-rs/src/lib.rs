//! obmm-rs: Rust bindings for OBMM (Ownership-Based Memory Management)
//! 
//! This crate provides Rust bindings and utilities for interacting with OBMM,
//! enabling memory exporting, importing, and management in a safe and ergonomic way.
#![deny(absolute_paths_not_starting_with_crate, 
    explicit_outlives_requirements, 
    keyword_idents,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_abi,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    non_ascii_idents,
    noop_method_call,
    rust_2021_incompatible_closure_captures,
    rust_2021_incompatible_or_patterns,
    rust_2021_prefixes_incompatible_syntax,
    rust_2021_prelude_collisions,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unsafe_op_in_unsafe_fn,
    unstable_features,
    unused_extern_crates,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_results,
    variant_size_differences,
    warnings,
    clippy::all,
    clippy::pedantic,
    clippy::cargo,
    clippy::as_conversions,
    clippy::clone_on_ref_ptr,
    clippy::create_dir,
    clippy::dbg_macro,
    clippy::decimal_literal_representation,
    clippy::disallowed_script_idents,
    clippy::else_if_without_else,
    clippy::exhaustive_enums,
    clippy::exhaustive_structs,
    clippy::exit,
    clippy::expect_used,
    clippy::filetype_is_file,
    clippy::float_arithmetic,
    clippy::float_cmp_const,
    clippy::get_unwrap,
    clippy::if_then_some_else_none,
    clippy::indexing_slicing,
    clippy::inline_asm_x86_intel_syntax,
    clippy::arithmetic_side_effects,
    clippy::let_underscore_must_use,
    clippy::lossy_float_literal,
    clippy::map_err_ignore,
    clippy::mem_forget,
    clippy::missing_docs_in_private_items,
    clippy::missing_enforced_import_renames,
    clippy::missing_inline_in_public_items,
    clippy::modulo_arithmetic,
    clippy::multiple_inherent_impl,
    clippy::pattern_type_mismatch,
    clippy::print_stderr,
    clippy::print_stdout,
    clippy::rc_buffer,
    clippy::rc_mutex,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::same_name_method,
    clippy::self_named_module_files,
    clippy::shadow_unrelated,
    clippy::str_to_string,
    clippy::string_add,
    clippy::todo,
    clippy::unimplemented,
    clippy::unnecessary_self_imports,
    clippy::unneeded_field_pattern,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    clippy::verbose_file_reads,
    clippy::wildcard_enum_match_arm,
)]

use std::ffi::c_void;
use bitflags::bitflags;
use serde::{Serialize, Deserialize};

/// Maximum number of NUMA nodes supported
pub const MAX_NUMA_NODES: usize = 16;
/// Invalid memory ID constant
pub const OBMM_INVALID_MEMID: u64 = 0;
/// Maximum number of local NUMA nodes supported
pub const OBMM_MAX_LOCAL_NUMA_NODES: usize = 16;
/// Memory ID type
pub type MemId = u64;

bitflags! {
    /// Privilege data for UB memory regions
    #[derive(Serialize, Deserialize, Default, Debug, PartialEq, Eq)]
    #[serde(transparent)]
    pub struct UbPrivData: u16 {
        /// Owner Chip ID
        const OCHIP = 1 << 5;
        /// Cacheable flag
        const CACHEABLE = 1 << 6;
    }
}

bitflags! {
    /// Export flags for memory exporting
    #[derive(Default, Debug)]
    pub struct ObmmExportFlags: u64 {
        /// Allow memory mapping
        const ALLOWMMAP = 1 << 0;
        /// Export to remote NUMA nodes
        const REMOTENUMA = 1 << 1;
    }
}

bitflags! {
    /// Unexport flags for memory unexporting
    #[derive(Default, Debug)]
    pub struct ObmmUnexportFlags: u64 {
        /// Force unexport
        const FORCE = 1 << 0;
    }
}

/// Memory descriptor structure
#[repr(C)]
#[derive(Default, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ObmmMemDesc<T> {
    /// Base address of the memory region
    pub addr: u64,
    /// Length of the memory region
    pub length: u64,
    /// 128bit eid, ordered by little-endian
    pub seid: [u8; 16],
    /// 128bit deid, ordered by little-endian
    pub deid: [u8; 16],
    /// Token ID
    pub tokenid: u32,
    /// Source CNA
    pub scna: u32,
    /// Destination CNA
    pub dcna: u32,
    /// Length of privilege data
    pub priv_len: u16,
    /// Privilege data
    pub priv_data: T,
}


impl<T> ObmmMemDesc<T>  
    where
    T: Default + Serialize + for<'de> Deserialize<'de>,
{
    /// Create a new `ObmmMemDesc` with default values
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        ObmmMemDesc::<T>::default()
    }

    /// Deserialize the `ObmmMemDesc` from json format
    /// # Arguments
    /// * `json_str` - JSON string representation
    /// # Returns
    /// # Errors
    /// `ObmmMemDesc` on success, `anyhow::Error` on failure
    #[inline]
    pub fn from_json(json_str: &str) -> anyhow::Result<Self> {
        let desc: ObmmMemDesc<T> = serde_json::from_str(json_str)?;
        Ok(desc)
    }

    /// Serialize the `ObmmMemDesc` to json format
    /// # Returns
    /// # Errors
    /// JSON string on success, `anyhow::Error` on failure
    #[inline]
    pub fn to_json(&self) -> anyhow::Result<String> {
        let json_str = serde_json::to_string(self)?;
        Ok(json_str)
    }

    /// Read the `ObmmMemDesc` from a json file
    /// # Arguments
    /// * `mem_id` - Memory ID
    /// # Returns
    /// # Errors
    /// `ObmmMemDesc` on success, `anyhow::Error` on failure
    #[inline]
    pub fn from_json_file(mem_id: MemId) -> anyhow::Result<Self> {
        let file_path = format!("/tmp/memlink/memdesc_{mem_id}.json");
        let json_str = std::fs::read_to_string(file_path)?;
        let desc: ObmmMemDesc<T> = serde_json::from_str(&json_str)?;
        Ok(desc)
    }

    /// Write the `ObmmMemDesc` to a json file
    /// # Arguments
    /// * `mem_id` - Memory ID
    /// # Returns
    /// # Errors
    /// Ok(()) on success, `anyhow::Error` on failure
    #[inline]
    pub fn to_json_file(&self, mem_id: MemId) -> anyhow::Result<()> {
        let file_path = format!("/tmp/memlink/memdesc_{mem_id}.json");
        let json_str = serde_json::to_string_pretty(self)?;
        std::fs::write(file_path, json_str)?;
        Ok(())
    }
}

/// Export memory region
/// # Arguments
/// * `length` - Array of lengths for each NUMA node
/// * `flags` - Export flags
/// # Returns
/// # Errors
/// Tuple of Memory ID and Memory Descriptor on success, `anyhow::Error` on failure
#[cfg(feature = "hook")]
#[inline]
pub fn mem_export<T: Default>(length: &[usize], _: ObmmExportFlags) -> anyhow::Result<(MemId, ObmmMemDesc<T>)> {
    let mut desc = ObmmMemDesc::<T>::default();
    // hooked implementation
    let memid = 1;
    desc.addr = 0xffff_fc00_0000;
    desc.length = length.iter().sum::<usize>().try_into()?;
    if memid == OBMM_INVALID_MEMID {
        Err(anyhow::anyhow!("Failed to export memory"))
    } else {
        Ok((memid, desc))
    }
}

/// Export memory region
/// # Arguments
/// * `length` - Array of lengths for each NUMA node
/// * `flags` - Export flags
/// # Returns
/// Tuple of Memory ID and Memory Descriptor on success, anyhow::Error on failure
#[cfg(not(feature = "hook"))]
pub fn mem_export<T: Default>(length: &[usize], flags: ObmmExportFlags) -> anyhow::Result<(MemId, ObmmMemDesc<T>)> {
    let mut desc = ObmmMemDesc::<T>::default();
    let memid = unsafe {
        obmm_export(
            length.as_ptr(),
            flags.bits(),
            &mut desc as *mut ObmmMemDesc<T> as *mut c_void,
        )
    };
    if memid == OBMM_INVALID_MEMID {
        Err(anyhow::anyhow!("Failed to export memory"))
    } else {
        Ok((memid, desc))
    }
}

/// Unexport memory region
/// # Arguments
/// * `memid` - Memory ID to unexport
/// * `flags` - Unexport flags
/// # Returns
/// Ok(()) on success, Err(i32) on failure
/// # Errors
#[cfg(feature = "hook")]
#[inline]
pub fn mem_unexport(_: MemId, _: ObmmUnexportFlags) -> Result<(), i32> {
    // hooked implementation
    Ok(())
}

/// Unexport memory region
/// # Arguments
/// * `memid` - Memory ID to unexport
/// * `flags` - Unexport flags
/// # Returns
/// Ok(()) on success, Err(i32) on failure
#[cfg(not(feature = "hook"))]
pub fn mem_unexport(memid: MemId, flags: ObmmUnexportFlags) -> Result<(), i32> {
    let ret = unsafe { obmm_unexport(memid, flags.bits()) };
    if ret == 0 {
        Ok(())
    } else {
        Err(ret)
    }
}

/// Import memory region
/// # Arguments
/// * `desc` - Memory Descriptor from remote
/// * `flags` - Import flags
/// * `base_dist` - Base distribution hint
/// # Returns
/// # Errors
/// Tuple of Memory ID and NUMA node on success, Err(i32) on failure
#[cfg(feature = "hook")]
#[inline]
pub fn mem_import(
    _: &ObmmMemDesc<UbPrivData>,
    _: ObmmExportFlags,
    _: i32,
) -> Result<(MemId, i32), i32> {
    // hooked implementation
    let memid = 1;
    let numa = 0;
    if memid == OBMM_INVALID_MEMID {
        Err(-1)
    } else {
        Ok((memid, numa))
    }
}

/// Import memory region
/// # Arguments
/// * `desc` - Memory Descriptor from remote
/// * `flags` - Import flags
/// * `base_dist` - Base distribution hint
/// # Returns
/// Tuple of Memory ID and NUMA node on success, Err(i32) on failure
#[cfg(not(feature = "hook"))]
pub fn mem_import(
    desc: &ObmmMemDesc<UbPrivData>,
    flags: ObmmExportFlags,
    base_dist: i32,
) -> Result<(MemId, i32), i32> {
    let mut numa: i32 = -1;
    let memid = unsafe {
        obmm_import(
            desc as *const ObmmMemDesc<UbPrivData> as *const c_void,
            flags.bits(),
            base_dist,
            &mut numa as *mut i32,
        )
    };
    if memid == OBMM_INVALID_MEMID {
        Err(-1)
    } else {
        Ok((memid, numa))
    }
}


/// Unimport memory region
/// # Arguments
/// * `memid` - Memory ID to unimport
/// * `flags` - Unimport flags
/// # Returns
/// Ok(()) on success, Err(i32) on failure
#[cfg(not(feature = "hook"))]
pub fn mem_unimport(memid: MemId, flags: ObmmExportFlags) -> Result<(), i32> {
    let ret = unsafe { obmm_unimport(memid, flags.bits()) };
    if ret == 0 {
        Ok(())
    } else {
        Err(ret)
    }
}

// FFI bindings to OBMM C library
unsafe extern "C" {
    /// Export memory regions for remote access
    ///
    /// # Arguments
    /// * `length` - Array of lengths for each NUMA node
    /// * `flags` - Export flags
    /// * `desc` - Output memory descriptor
    ///
    /// # Returns
    /// Memory ID on success, `OBMM_INVALID_MEMID` on failure
    pub fn obmm_export(
        length: *const usize,
        flags: u64,
        desc: *mut c_void,
    ) -> MemId;

    /// Unexport previously exported memory region
    ///
    /// # Arguments
    /// * `id` - Memory ID to unexport
    /// * `flags` - Unexport flags
    ///
    /// # Returns
    /// 0 on success, -1 on failure
    pub fn obmm_unexport(id: MemId, flags: u64) -> i32;

    /// Import remote memory region
    ///
    /// # Arguments
    /// * `desc` - Memory descriptor from remote
    /// * `flags` - Import flags
    /// * `base_dist` - Base distribution hint
    /// * `numa` - Output NUMA node ID
    ///
    /// # Returns
    /// Memory ID on success, `OBMM_INVALID_MEMID` on failure
    pub fn obmm_import(
        desc: *const c_void,
        flags: u64,
        base_dist: i32,
        numa: *mut i32,
    ) -> MemId;

    /// Unimport previously imported memory region
    ///
    /// # Arguments
    /// * `id` - Memory ID to unimport
    /// * `flags` - Unimport flags
    ///
    /// # Returns
    /// 0 on success, -1 on failure
    pub fn obmm_unimport(id: MemId, flags: u64) -> i32;

    /* debug interface */
    
    /// Query memory ID by physical address
    ///
    /// # Arguments
    /// * `pa` - Physical address
    /// * `id` - Output memory ID
    /// * `offset` - Output offset within memory region
    ///
    /// # Returns
    /// 0 on success, -1 on failure
    pub fn obmm_query_memid_by_pa(
        pa: u64,
        id: *mut MemId,
        offset: *mut u64,
    ) -> i32;

    /// Query physical address by memory ID and offset
    ///
    /// # Arguments
    /// * `id` - Memory ID
    /// * `offset` - Offset within memory region
    /// * `pa` - Output physical address
    ///
    /// # Returns
    /// 0 on success, -1 on failure
    pub fn obmm_query_pa_by_memid(
        id: MemId,
        offset: u64,
        pa: *mut u64,
    ) -> i32;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export() {
        let mut lengths = vec![0; MAX_NUMA_NODES];
        lengths[1] = 1024 * 1024 * 128; // 128MB on NUMA node 1
        let flags = ObmmExportFlags::ALLOWMMAP;
        match mem_export::<UbPrivData>(&lengths, flags) {
            Ok((memid, desc)) => {
                println!("Exported MemID: {}", memid);
                println!("Memory Descriptor: {:?}", desc);
                assert!(memid != OBMM_INVALID_MEMID);
                assert!(desc.length == 1024 * 1024 * 128);
            }
            Err(code) => {
                panic!("mem_export failed with code {}", code);
            }
        }
    }

    #[test]
    fn test_import() {
        let desc = ObmmMemDesc::<UbPrivData> {
            addr: 0xffff_fc00_0000,
            length: 1024 * 1024 * 128,
            seid: [0; 16],
            deid: [0; 16],
            tokenid: 0,
            scna: 0,
            dcna: 0,
            priv_len: 0,
            priv_data: UbPrivData::default(),
        };
        let flags = ObmmExportFlags::ALLOWMMAP;
        match mem_import(&desc, flags, 0) {
            Ok((memid, numa)) => {
                println!("Imported MemID: {}, NUMA Node: {}", memid, numa);
                assert!(memid != OBMM_INVALID_MEMID);
            }
            Err(code) => {
                panic!("mem_import failed with code {}", code);
            }
        }
    }

    #[test]
    fn test_serialization() {
        let desc = ObmmMemDesc::<UbPrivData> {
            addr: 0xffff_fc00_0000,
            length: 1024 * 1024 * 128,
            seid: [1; 16],
            deid: [2; 16],
            tokenid: 42,
            scna: 3,
            dcna: 4,
            priv_len: 2,
            priv_data: UbPrivData::OCHIP | UbPrivData::CACHEABLE,
        };
        let json_str = desc.to_json().unwrap();
        println!("Serialized JSON: {}", json_str);
        let deserialized_desc: ObmmMemDesc<UbPrivData> = ObmmMemDesc::from_json(&json_str).unwrap();
        assert_eq!(desc.addr, deserialized_desc.addr);
        assert_eq!(desc.length, deserialized_desc.length);
        assert_eq!(desc.seid, deserialized_desc.seid);
        assert_eq!(desc.deid, deserialized_desc.deid);
        assert_eq!(desc.tokenid, deserialized_desc.tokenid);
        assert_eq!(desc.scna, deserialized_desc.scna);
        assert_eq!(desc.dcna, deserialized_desc.dcna);
        assert_eq!(desc.priv_len, deserialized_desc.priv_len);
        assert_eq!(desc.priv_data, deserialized_desc.priv_data);
    }
    #[test]
    fn test_deserialization() -> anyhow::Result<()> {
        let json_str = r#"{
            "addr": 281474909601792,
            "length": 134217728,
            "seid": [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1],
            "deid": [2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2],
            "tokenid": 42,
            "scna": 3,
            "dcna": 4,
            "priv_len": 2,
            "priv_data": "OCHIP | CACHEABLE"
        }"#;
        let desc = ObmmMemDesc::<UbPrivData>::from_json(json_str)?;
        println!("Deserialized ObmmMemDesc: {:?}", desc);
        assert_eq!(desc.addr, 0xffff_fc00_0000);
        assert_eq!(desc.length, 1024 * 1024 * 128);
        assert_eq!(desc.tokenid, 42);
        assert_eq!(desc.priv_data, (UbPrivData::OCHIP | UbPrivData::CACHEABLE));
        Ok(())
    }

    #[test]
    fn test_json_file_io() -> anyhow::Result<()> {
        let desc = ObmmMemDesc::<UbPrivData> {
            addr: 0xffff_fc00_0000,
            length: 1024 * 1024 * 128,
            seid: [1; 16],
            deid: [2; 16],
            tokenid: 42,
            scna: 3,
            dcna: 4,
            priv_len: 2,
            priv_data: UbPrivData::OCHIP | UbPrivData::CACHEABLE,
        };
        let mem_id: MemId = 12345;
        desc.to_json_file(mem_id)?;
        let read_desc = ObmmMemDesc::<UbPrivData>::from_json_file(mem_id)?;
        assert_eq!(desc.addr, read_desc.addr);
        assert_eq!(desc.length, read_desc.length);
        assert_eq!(desc.seid, read_desc.seid);
        assert_eq!(desc.deid, read_desc.deid);
        assert_eq!(desc.tokenid, read_desc.tokenid);
        assert_eq!(desc.scna, read_desc.scna);
        assert_eq!(desc.dcna, read_desc.dcna);
        assert_eq!(desc.priv_len, read_desc.priv_len);
        assert_eq!(desc.priv_data, read_desc.priv_data);
        Ok(())
    }
}