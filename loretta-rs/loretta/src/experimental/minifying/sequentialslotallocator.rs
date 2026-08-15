// Ported from Loretta.CodeAnalysis.Lua.Experimental.Minifying.SequentialSlotAllocator (b767b4e): SequentialSlotAllocator
// C# source: src/Compilers/Lua/Experimental/Minifying/SequentialSlotAllocator.cs

use crate::experimental::minifying::islotallocator::ISlotAllocator;

/// A sequential slot allocator.
/// Never returns previously used slots and is the fastest one.
pub struct SequentialSlotAllocator {
    slot: i32,
}

impl SequentialSlotAllocator {
    /// Creates a new SequentialSlotAllocator.
    pub fn new() -> Self {
        Self { slot: -1 }
    }
}

impl Default for SequentialSlotAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl ISlotAllocator for SequentialSlotAllocator {
    fn allocate_slot(&mut self) -> i32 {
        self.slot = self.slot.wrapping_add(1);
        self.slot
    }

    fn release_slot(&mut self, _slot: i32) {
        // Do nothing.
    }
}
