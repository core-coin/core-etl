pub trait SmartContract: Send + Sync {
    fn check_if_call(&self, input: String) -> bool;
    fn extract_call_data(
        &self,
        sender: String,
        input: String,
    ) -> Vec<(usize, String, String, String)>;

    fn get_address(&self) -> String;
    fn get_table_name(&self) -> String;

    fn clone_dyn(&self) -> Box<dyn SmartContract>;
}

impl Clone for Box<dyn SmartContract> {
    fn clone(&self) -> Self {
        self.clone_dyn()
    }
}
