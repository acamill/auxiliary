use std::{sync::Mutex, time::Duration};

use tracing::trace;

use crate::{error::Error, AppState, Product};

#[derive(Clone)]
pub struct PriceFetchConfig {
    pub interval: Duration,
}

pub async fn run<'a>(
    state: &'static AppState<'a>,
    cfg: PriceFetchConfig,
) -> Result<(), Error> {
    let handles = state.iter_products().map(|product| {
        let cfg = cfg.clone();

        tokio::task::spawn_blocking(move || loop {
            std::thread::sleep(cfg.interval);
            update_price(state, product)
        })
    });

    let _ = futures::future::join_all(handles).await;
    Ok(())
}

#[tracing::instrument(
    skip(state),
    name = "update_price"
    fields(current_price = tracing::field::Empty, confidence = tracing::field::Empty)
)]
fn update_price<'a>(state: &'static AppState<'a>, product: &'a Product) {
    // Fetch price from Pyth accounts
    let pi_new = state.pyth_price_info(product).unwrap();
    // Get write on the cache
    let price_cache = state.price_cache.read().expect("RwLock poisoned");

    tracing::Span::current().record("current_price", &pi_new.price);
    tracing::Span::current().record("confidence", &pi_new.conf);
    // tracing::Span::current().record("status", &pi_new.status);
    // tracing::Span::current().record("corp_act", &pi_new.corp_act); // Not implemented yet in Pyth

    if let Some(pi) = price_cache.get(&product) {
        trace!("Refresh");
        let mut pi = pi.lock().expect("Mutex poisoned");
        // Perform our normal work updating a specific element.
        // The entire HashMap only has a read lock, which
        // means that other threads can access it.
        *pi = pi_new;
    } else {
        trace!("Create");
        drop(price_cache);
        let mut price_cache =
            state.price_cache.write().expect("RwLock poisoned");
        price_cache.insert(product, Mutex::new(pi_new));
    }
}
