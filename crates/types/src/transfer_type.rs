/// TransferType is an enum that represents the type of transfer. It can be To, From or All
#[derive(Debug, Clone)]
pub enum TransferType {
    // Transfers where address is the sender
    To,
    // Transfers where address is the receiver
    From,
    // All transfers
    All,
}
