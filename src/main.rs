use atomic_float::AtomicF64;
use device_query::{DeviceEvents, DeviceState};
use dotenv::dotenv;
use serde_json::json;
use std::{
    error::Error,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use tokio::time;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let device_state = DeviceState::new();
    let count = Arc::new(AtomicUsize::new(0));
    let right_click = Arc::new(AtomicUsize::new(0));
    let left_click = Arc::new(AtomicUsize::new(0));
    let total_distance = Arc::new(AtomicF64::new(0.0));
    let last_pos_x = Arc::new(AtomicF64::new(0.0));
    let last_pos_y = Arc::new(AtomicF64::new(0.0));

    let c = Arc::clone(&count);
    tokio::spawn(async move {
        let mut interval = time::interval(time::Duration::from_secs(100));
        loop {
            interval.tick().await;
            let keypresses = c.load(Ordering::Relaxed);
            if let Err(e) = send_keypress(keypresses as u64).await {
                eprintln!("Error while sending post request: {}", e);
            } else {
                c.store(0, Ordering::Relaxed);
            }
        }
    });

    let mouse_travel = Arc::clone(&total_distance);
    let lp_x = Arc::clone(&last_pos_x);
    let lp_y = Arc::clone(&last_pos_y);
    let right = Arc::clone(&right_click);
    let left = Arc::clone(&left_click);
    tokio::spawn(async move {
        let mut interval = time::interval(time::Duration::from_secs(100));
        loop {
            interval.tick().await;
            let dist = mouse_travel.load(Ordering::Relaxed) * 0.0002645833;
            let r = right.load(Ordering::Relaxed);
            let l = left.load(Ordering::Relaxed);
            if let Err(e) = send_distance(dist as f64, r as u64, l as u64).await {
                eprintln!("Error while sending post request: {}", e);
            } else {
                mouse_travel.store(0.0, Ordering::Relaxed);
                right.store(0, Ordering::Relaxed);
                left.store(0, Ordering::Relaxed)
            }
        }
    });

    let _guard = device_state.on_mouse_move(move |pos| {
        let x = lp_x.load(Ordering::Relaxed);
        let y = lp_y.load(Ordering::Relaxed);

        let dx = pos.0 as f64 - x;
        let dy = pos.1 as f64 - y;
        let d = f64::sqrt(dx * dx + dy * dy);
        lp_x.store(pos.0 as f64, Ordering::Relaxed);
        lp_y.store(pos.1 as f64, Ordering::Relaxed);

        total_distance.fetch_add(d, Ordering::Relaxed);
    });

    let _guard = device_state.on_mouse_down(move |button| {
        if *button == 1 {
            right_click.fetch_add(1, Ordering::Relaxed);
        } else if *button == 3 {
            right_click.fetch_add(1, Ordering::Relaxed);
        }
    });

    let _guard = device_state.on_key_up(move |_| {
        count.fetch_add(1, Ordering::Relaxed);
    });

    loop {
        tokio::time::sleep(time::Duration::from_millis(100)).await;
    }
}

async fn send_keypress(count: u64) -> Result<(), Box<dyn Error>> {
    let client = reqwest::Client::new();

    let body = json!({
        "counter": count
    });

    let _ = client
        .post("")
        .bearer_auth("")
        .body(body.to_string())
        .send()
        .await?;
    Ok(())
}

async fn send_distance(
    distance: f64,
    right_click: u64,
    left_click: u64,
) -> Result<(), Box<dyn Error>> {
    let client = reqwest::Client::new();

    let body = json!({
        "mouseDistance": distance as u64,
        "rightClick": right_click,
        "leftClick": left_click
    });

    let _ = client
        .post("")
        .bearer_auth("")
        .body(body.to_string())
        .send()
        .await?;

    Ok(())
}
