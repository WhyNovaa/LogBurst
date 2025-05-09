pub trait AsyncNew {
    async fn new() -> Self;
}
