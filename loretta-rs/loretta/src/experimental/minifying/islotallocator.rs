// Ported from Loretta.CodeAnalysis.Lua.Experimental.Minifying.ISlotAllocator (b767b4e): ISlotAllocator
// C# source: src/Compilers/Lua/Experimental/Minifying/ISlotAllocator.cs

/// The slot allocator to use for renaming.
pub trait ISlotAllocator {
    /// Allocates a slot for the provided variable.
    /// Returns the slot that was allocated to the variable.
    fn allocate_slot(&mut self) -> i32;

    /// Releases a slot for usage by other variables.
    /// `slot` is the slot the variable is located in.
    fn release_slot(&mut self, slot: i32);
}
