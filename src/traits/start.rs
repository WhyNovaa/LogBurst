pub trait Start {
    fn new() -> Self;
    fn start(self);
}