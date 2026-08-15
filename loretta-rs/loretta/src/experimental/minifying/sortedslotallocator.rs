// Ported from Loretta.CodeAnalysis.Lua.Experimental.Minifying.SortedSlotAllocator (b767b4e): SortedSlotAllocator
// C# source: src/Compilers/Lua/Experimental/Minifying/SortedSlotAllocator.cs

use std::collections::BTreeSet;

use crate::experimental::minifying::islotallocator::ISlotAllocator;

/// The sorted slot allocator.
/// Will always use the lowest free slot.
pub struct SortedSlotAllocator {
    free_slots: BTreeSet<i32>,
    current_slot: i32,
}

impl SortedSlotAllocator {
    /// Creates a new SortedSlotAllocator.
    pub fn new() -> Self {
        Self {
            free_slots: BTreeSet::new(),
            current_slot: 0,
        }
    }
}

impl Default for SortedSlotAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl ISlotAllocator for SortedSlotAllocator {
    fn allocate_slot(&mut self) -> i32 {
        if let Some(&slot) = self.free_slots.iter().next() {
            self.free_slots.remove(&slot);
            slot
        } else {
            let slot = self.current_slot;
            self.current_slot = self.current_slot.wrapping_add(1);
            slot
        }
    }

    fn release_slot(&mut self, slot: i32) {
        if !self.free_slots.insert(slot) {
            panic!("Slot {slot} was released two times.");
        }
    }
}
