//@ rules: REF-0601
// Check written syntax before rustc lowers it to async machinery.
pub async fn fetch() -> u8 { 3 } //~ TD0601
pub async fn caller() -> u8 { //~ TD0601
    fetch().await //~ TD0601
}
pub fn closures() {
    let _block = async { 4 }; //~ TD0601
    let _closure = async || 5; //~ TD0601
}
pub trait Service {
    async fn fetch(&self) -> u8; //~ TD0601
}
