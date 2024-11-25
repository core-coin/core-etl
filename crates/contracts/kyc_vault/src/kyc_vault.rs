use contracts::SmartContract;
use tracing::info;

use crate::KYC_VAULT_NAME;

#[derive(Debug, Clone)]
pub struct KYCVault {
    address: String,
    transfer_selectors: Vec<String>,
}

impl KYCVault {
    pub fn new(address: String) -> Self {
        KYCVault {
            address,
            transfer_selectors: vec![
                "1e584928".to_string(), // submit(bytes32,tuple(uint256,address,bytes32[])[])
            ],
        }
    }
}

impl SmartContract for KYCVault {
    fn check_if_call(&self, input: String) -> bool {
        for selector in &self.transfer_selectors {
            if input.starts_with(selector) {
                return true;
            }
        }
        return false;
    }

    fn extract_call_data(
        &self,
        sender: String,
        input: String,
    ) -> Vec<(usize, String, String, String)> {
        match &input[..8] {
            "1e584928" => {
                info!("TxHash {:?}", tx_hash);
                vec![]
            }
            _ => panic!("Unsupported transfer function: {:?}", input),
        }
    }

    fn get_address(&self) -> String {
        self.address.clone()
    }

    fn get_table_name(&self) -> String {
        return format!("{}_{}_transfers", KYC_VAULT_NAME, &self.get_address()[..8]);
    }

    fn clone_dyn(&self) -> Box<dyn SmartContract> {
        Box::new(self.clone()) // Forward to the derive(Clone) impl
    }
}
