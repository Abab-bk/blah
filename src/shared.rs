pub enum Operation {
    Next,                 // >
    Prev,                 // <
    Increment,            // +
    Decrement,            // -
    Output,               // .
    Input,                // ,
    JumpIfZero(usize),    // [
    JumpIfNonzero(usize), // ]
}
