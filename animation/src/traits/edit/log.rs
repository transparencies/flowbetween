///
/// Trait implemented by things that can be edited (or described) by sequences
/// of commands
/// 
pub trait EditLog<Edit> {
    ///
    /// Retrieves the number of edits in this log
    ///
    fn length(&self) -> usize;

    ///
    /// Reads a range of edits from this log
    /// 
    fn read(&self, start: usize, end: usize) -> Vec<Edit>;

    ///
    /// Sets the pending edits for this log (replacing any existing
    /// pending edits)
    /// 
    fn set_pending(&mut self, edits: &[Edit]);

    ///
    /// Commits any pending edits for this log
    /// 
    fn commit_pending(&mut self);

    ///
    /// Cancels any pending edits for this log
    /// 
    fn cancel_panding(&mut self);

    // TODO: undos, redos?
}
