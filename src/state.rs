use std::{
    collections::HashMap,
    str::FromStr,
    sync::{Arc, Mutex, RwLock},
};

use anchor_client::{
    solana_client::rpc_client::RpcClient,
    solana_sdk::{
        commitment_config::CommitmentConfig, pubkey::Pubkey,
        signer::keypair::Keypair,
    },
    Client, Cluster, Program,
};

use crate::Error;

// https://pyth.network/developers/accounts/?cluster=devnet#
pub mod pyth_account {
    #[cfg(feature = "devnet")]
    pub const MAPPING: &str = "BmA9Z6FjioHJPpjT39QazZyhDRUdZy2ezwx4GiDdE2u2";
    #[cfg(feature = "mainnet")]
    pub const MAPPING: &str = "AHtgzX45WTKfkPG53L6WYhGEXwQkN1BVknET3sVsLL8J";

    #[cfg(feature = "devnet")]
    pub const SOL_USD: &str = "3Mnn2fX6rQyUsyELYms1sBJyChWofzSNRoqYzvgMVz5E";
    #[cfg(feature = "mainnet")]
    pub const SOL_USD: &str = "9mpaSy5ocwPvoaxWZc4S8MhUUeUKmmFqymBJTfY6CJ6o";

    #[cfg(feature = "devnet")]
    pub const SOL_USD_PRICE: &str =
        "J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix";
    #[cfg(feature = "mainnet")]
    pub const SOL_USD_PRICE: &str =
        "H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG";
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Product {
    SolUsd,
}

pub struct AppState<'a> {
    payer: Keypair,
    commitment: CommitmentConfig,
    pub cluster: Cluster,
    pub rpc: RpcClient,
    products: Vec<Product>,
    // Thread safe and mutated by `price_engine`
    pub price_cache:
        Arc<RwLock<HashMap<&'a Product, Mutex<pyth_client::PriceInfo>>>>,
}

impl<'a> AppState<'a> {
    pub fn new(
        cluster: Cluster,
        payer: Keypair,
        products: Vec<Product>,
    ) -> Self {
        let program = Client::new_with_options(
            cluster.clone(),
            std::rc::Rc::new(Keypair::from_bytes(&payer.to_bytes()).unwrap()),
            CommitmentConfig::confirmed(),
        )
        .program(pyth_client::ID);

        Self {
            payer,
            commitment: CommitmentConfig::confirmed(),
            cluster,
            rpc: program.rpc(),
            products,
            price_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn payer(&self) -> Pubkey {
        use anchor_client::solana_sdk::signer::Signer;
        self.payer.pubkey()
    }

    pub fn client(&self) -> Client {
        Client::new_with_options(
            self.cluster.clone(),
            std::rc::Rc::new(
                Keypair::from_bytes(&self.payer.to_bytes()).unwrap(),
            ),
            self.commitment,
        )
    }

    pub fn pyth_program(&self) -> Program {
        self.client().program(pyth_client::ID)
    }

    pub fn iter_products(&self) -> impl Iterator<Item = &Product> {
        self.products.iter()
    }

    // pub fn pyth_mapping(&self) -> Result<pyth_client::Mapping, Error> {
    //     let mapping_account_key =
    //         Pubkey::from_str(pyth_account::MAPPING).unwrap();
    //     let mapping_data = self.rpc.get_account_data(&mapping_account_key)?;
    //     Ok(pyth_client::load_mapping(&mapping_data)?.clone())
    // }

    // pub fn pyth_product(
    //     &self,
    //     product: &Product,
    // ) -> Result<pyth_client::Product, Error> {
    //     let product_account_id = match product {
    //         Product::SolUsd => pyth_account::SOL_USD,
    //     };
    //     let product_account_key = Pubkey::from_str(product_account_id).unwrap();
    //     let product_data = self.rpc.get_account_data(&product_account_key)?;
    //     Ok(pyth_client::load_product(&product_data)?.clone())
    // }

    // Aggregate
    pub fn pyth_price_info(
        &self,
        product: &Product,
    ) -> Result<pyth_client::PriceInfo, Error> {
        let price_account_id = match product {
            Product::SolUsd => pyth_account::SOL_USD_PRICE,
        };
        let price_account_key = Pubkey::from_str(price_account_id).unwrap();
        let price_data = self.rpc.get_account_data(&price_account_key)?;
        Ok(pyth_client::load_price(&price_data)?.agg)
    }
}
