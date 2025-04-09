use tokio::sync::mpsc;
pub use crate::price_level::PxQtLadder;
pub type PriceLevelSender = mpsc::UnboundedSender<(PxQtLadder, PxQtLadder)>;
pub type PriceLevelReceiver = mpsc::UnboundedReceiver<(PxQtLadder, PxQtLadder)>;

pub fn create_databus() -> (PriceLevelSender, PriceLevelReceiver) {
    mpsc::unbounded_channel()
}
