use crate::constants::NAME;
use crate::{cbc20, CBC20_NAME};
use contracts::SmartContract;
use std::{i64, vec};

#[derive(Debug, Clone)]
pub struct Cbc20 {
    address: String,
    transfer_selectors: Vec<String>,
}

impl Cbc20 {
    pub fn new(address: String) -> Self {
        Cbc20 {
            address,
            transfer_selectors: vec![
                "4b40e901".to_string(), // transfer(address,uint256)
                "e86e7c5f".to_string(), // batchTransfer(address[],uint256[])
                "31f2e679".to_string(), // transferFrom(address,address,uint256)
            ],
        }
    }
}

impl SmartContract for Cbc20 {
    fn check_if_transfer(&self, input: String) -> bool {
        for selector in &self.transfer_selectors {
            if input.starts_with(selector) {
                return true;
            }
        }
        return false;
    }

    fn extract_transfer_data(
        &self,
        sender: String,
        input: String,
    ) -> Vec<(String, String, String)> {
        match &input[..8] {
            // Example: 4b40e901 + 00000000000000000000ab416902d2548d52352a05423d13266ee7aaf140a068 + 0000000000000000000000000000000000000000000000000000000000000001
            "4b40e901" => {
                let from = sender;
                let to = input[28..72].to_string();
                let value = input[72..136].to_string();
                return vec![(from, to, value)];
            }
            /*
            /// Example:
            ///	e86e7c5f +                                                         // function signature
            ///	0000000000000000000000000000000000000000000000000000000000000040 + // offset to the first array - 64 bytes
            /// 00000000000000000000000000000000000000000000000000000000000000a0 + // offset to the second array - 160 bytes
            /// 0000000000000000000000000000000000000000000000000000000000000002 + // number of elements in the first array (address[])
            /// 00000000000000000000ab416902d2548d52352a05423d13266ee7aaf140a068 + // first address
            /// 00000000000000000000ab7153b962840676c37ba604c7816b0967cdb645cc54 + // second address
            /// 0000000000000000000000000000000000000000000000000000000000000002 + // number of elements in the second array (uint256[])
            /// 0000000000000000000000000000000000000000000000000000000000000001 + // first value
            /// 0000000000000000000000000000000000000000000000000000000000000001 + // second value
             */
            "e86e7c5f" => {
                let mut res = vec![];
                let offset = 136;
                let count = usize::from_str_radix(&input[136..200], 16).unwrap();
                for i in 0..count {
                    let to = input[offset + 84 + i * 64..offset + 128 + i * 64].to_string();
                    let value = input
                        [offset + 128 + count * 64 + i * 64..offset + 192 + count * 64 + i * 64]
                        .to_string();
                    res.push((sender.clone(), to, value));
                }
                return res;
            }
            // Example: 4b40e901 + 00000000000000000000ab416902d2548d52352a05423d13266ee7aaf140a068 + 00000000000000000000ab416902d2548d52352a05423d13266ee7aaf140a068 + 0000000000000000000000000000000000000000000000000000000000000001
            "31f2e679" => {
                panic!("Not implemented");
                let from = input[28..72].to_string();
                let to = input[92..136].to_string();
                let value = input[136..200].to_string();
                return vec![(from, to, value)];
            }
            _ => panic!("Unsupported transfer function: {:?}", input),
        }
    }

    fn get_address(&self) -> String {
        self.address.clone()
    }

    fn get_table_name(&self) -> String {
        return format!("{}_{}_transfers", NAME, &self.get_address()[..8]);
    }

    fn clone_dyn(&self) -> Box<dyn SmartContract> {
        Box::new(self.clone()) // Forward to the derive(Clone) impl
    }
}
