//! Modular evidence cases for the Ripex 0.3 release report.
//!
//! Each supported parser mode owns its curated correctness cases and malformed
//! inputs in a separate module. The report runner combines those cases with the
//! checked-in source corpus and a Tree-sitter parse baseline.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

struct TrackingAllocator;

static CURRENT_BYTES: AtomicUsize = AtomicUsize::new(0);
static PEAK_BYTES: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = System.alloc(layout);
        if !pointer.is_null() {
            let current = CURRENT_BYTES.fetch_add(layout.size(), Ordering::Relaxed) + layout.size();
            let mut peak = PEAK_BYTES.load(Ordering::Relaxed);
            while current > peak {
                match PEAK_BYTES.compare_exchange_weak(
                    peak,
                    current,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => break,
                    Err(observed) => peak = observed,
                }
            }
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        System.dealloc(pointer, layout);
        CURRENT_BYTES.fetch_sub(layout.size(), Ordering::Relaxed);
    }
}

#[global_allocator]
static ALLOCATOR: TrackingAllocator = TrackingAllocator;

pub fn allocation_baseline() -> usize {
    let current = CURRENT_BYTES.load(Ordering::Relaxed);
    PEAK_BYTES.store(current, Ordering::Relaxed);
    current
}

pub fn peak_allocation_since(baseline: usize) -> usize {
    PEAK_BYTES.load(Ordering::Relaxed).saturating_sub(baseline)
}

mod c;
mod cpp;
mod csharp;
mod go;
mod javascript;
mod python;
mod rust;
mod typescript;

#[derive(Clone, Copy)]
pub struct ExpectedFacts {
    pub symbols: &'static [&'static str],
    pub imports: &'static [&'static str],
    pub calls: &'static [&'static str],
    pub variables: &'static [&'static str],
}

#[derive(Clone, Copy)]
pub struct EvidenceCase {
    pub id: &'static str,
    pub language: &'static str,
    pub extension: &'static str,
    pub source: &'static str,
    pub expected: ExpectedFacts,
    pub malformed: &'static [&'static str],
}

pub fn all_cases() -> Vec<&'static EvidenceCase> {
    [
        c::cases(),
        cpp::cases(),
        csharp::cases(),
        go::cases(),
        javascript::cases(),
        python::cases(),
        rust::cases(),
        typescript::cases(),
    ]
    .into_iter()
    .flat_map(|cases| cases.iter())
    .collect()
}
