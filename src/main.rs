use axum::{serve, Router, routing::{post, get}, Json};
use tokio::net::TcpListener;
use std::collections::HashMap;
use std::sync::Arc;
use axum::extract::State;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

struct ParkingGarageStatus {
    lot: HashMap<String, bool>,
    amount_in_use: i16
}

#[derive(Deserialize)]
struct ParkingGarageUpdate {
    spot: String,
    taken: bool
}

#[derive(Serialize)]
struct ParkingGarageData {
    lot: HashMap<String, bool>,
    amount_in_use: i16
}

#[tokio::main]
async fn main() {
    println!("Parking spot tracker running.");

    let parking_garage = Arc::new(RwLock::new(ParkingGarageStatus {
        lot: HashMap::new(),
        amount_in_use: 0
    }));

    let app = Router::new()
        .route("/pt/management/health", get(|| async { "OK" }))
        .route("/pt/parking", post(post_garage_update))
        .route("/pt/parking", get(get_garage_status))
        .with_state(parking_garage);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    serve(listener, app).await.unwrap();
}

async fn post_garage_update(
    State(parking_garage): State<Arc<RwLock<ParkingGarageStatus>>>,
    Json(payload): Json<ParkingGarageUpdate>
) -> Json<&'static str> {
    let mut parking_garage_state = parking_garage.write().await;
    let spot = &payload.spot;
    let taken = payload.taken;

    if let Some(current_taken) = parking_garage_state.lot.get_mut(spot) {
        if *current_taken != taken {
            *current_taken = taken;
            parking_garage_state.amount_in_use += if taken { 1 } else { -1 };
            println!("Spot {} updated to taken: {}", spot, taken);
        } else {
            println!("Spot {} status unchanged: taken: {}", spot, taken);
        }
    } else {
        parking_garage_state.lot.insert(spot.clone(), taken);
        if taken {
            parking_garage_state.amount_in_use += 1;
        }
        println!("Spot {} added to lot, taken: {}", spot, taken);
    }

    Json("OK")
}

async fn get_garage_status(
    State(parking_garage): State<Arc<RwLock<ParkingGarageStatus>>>
) -> Json<ParkingGarageData> { // Changed return type here
    let parking_garage_state = parking_garage.read().await;

    let garage_data = ParkingGarageData {
        lot: parking_garage_state.lot.clone(),
        amount_in_use: parking_garage_state.amount_in_use,
    };

    Json(garage_data)
}
