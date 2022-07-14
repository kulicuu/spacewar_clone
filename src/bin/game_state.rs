#![allow(unused)]

use web_sys::{
    HtmlCanvasElement, WebGl2RenderingContext as GL, 
    window, AngleInstancedArrays, KeyboardEvent,
    EventTarget, MouseEvent, WebGlBuffer, WebGlProgram,
    WebGlUniformLocation,
};
use serde_json::{Value};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use std::sync::{Arc, Mutex};
use cgmath::prelude::*;
use cgmath::Rad;
use std::cell::RefCell;
use std::rc::Rc;
use std::convert::{TryInto};
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use gloo_console::log;
use std::f32::consts::PI;

use crate::structures::{CollisionSpace, GameState, Torp, Vehicle};
use crate::utils;
use crate::utils::time_polyfill::Instant;

pub fn create_game_state
()
-> Result<Arc<Mutex<GameState>>, String>
{
    let mode = 0; // Notionally code for 2-player local.

    let player_one = Arc::new(Mutex::new(Vehicle {
        position_dx: 0.3,
        position_dy: 0.3,
        vifo_theta: Rad(0.3),
        velocity_theta: Rad(0.3),
        velocity_scalar: 0.0,
        velocity_dx: 0.0,
        velocity_dy: 0.0,
    }));

    let player_two = Arc::new(Mutex::new(Vehicle {
        position_dx: -0.4,
        position_dy: -0.4,
        vifo_theta: Rad(-0.3),
        velocity_theta: Rad(-0.3),
        velocity_scalar: 0.0,
        velocity_dx: 0.0,
        velocity_dy: 0.0,
    }));

    let torps_in_flight = Arc::new(Mutex::new(vec![]));

    let collisions: Arc<Mutex<
        HashMap<String, Arc<Mutex<CollisionSpace>>>
    >> = 
        Arc::new(Mutex::new(HashMap::new()));


    let start_time: Arc<utils::time_polyfill::Instant> = Arc::new(Instant::now());

    let game_state = GameState {
        player_one: player_one,
        player_two: player_two,
        torps_in_flight: torps_in_flight,
        // start_time: Arc::new(Instant::now()),
        start_time: start_time,
        elapsed_time: Arc::new(Mutex::new(0)),
        game_over: Arc::new(Mutex::new(false)),
        collisions_map: collisions,
        mode: Arc::new(mode),
        result: Arc::new(Mutex::new(0)),
        torp_kills_player_1: Arc::new(Mutex::new((false, 0.0, 0.0))),
        torp_kills_player_2: Arc::new(Mutex::new((false, 0.0, 0.0))),
    };

    Ok(Arc::new(Mutex::new(game_state)))
}

// fn update_positions 
pub fn update_game_state // A slight misnomer, as game state is also mutated by event-handlers.

(
    time_delta: u128,
    game_state: Arc<Mutex<GameState>>,
)
// -> Result<Arc<Mutex<GameState>>, String>
{
    let mut collisions_map : HashMap<String, CollisionSpace> = HashMap::new();

    let delta_scalar = (time_delta as f32) * 0.001;
    let old_pos_dx = game_state.lock().unwrap().player_one.lock().unwrap().position_dx;
    let additional_dx = game_state.lock().unwrap().player_one.lock().unwrap().velocity_dx * (delta_scalar as f32);
    let mut new_pos_dx = old_pos_dx + additional_dx;
    if new_pos_dx < -1.0 {
        new_pos_dx = new_pos_dx + 2.0;
    }
    if new_pos_dx > 1.0 {
        new_pos_dx = new_pos_dx - 2.0;
    }
    let old_pos_dy = game_state.lock().unwrap().player_one.lock().unwrap().position_dy;
    let additional_dy = game_state.lock().unwrap().player_one.lock().unwrap().velocity_dy * (delta_scalar as f32);
    let mut new_pos_dy = old_pos_dy + additional_dy;
    if new_pos_dy < -1.0 {
        new_pos_dy += 2.0;
    }
    if new_pos_dy > 1.0 {
        new_pos_dy -= 2.0;
    }

    game_state.lock().unwrap().player_one.lock().unwrap().position_dx = new_pos_dx;
    game_state.lock().unwrap().player_one.lock().unwrap().position_dy = new_pos_dy;

    let p1_kx = new_pos_dx;
    let p1_ky = new_pos_dy;


    for i in -10..10 {
        for j in -10..10 {
            let base_dx = ((new_pos_dx * 1000.0) as i32) + i;
            let base_dy = ((new_pos_dy * 1000.0) as i32) + j;

            let s_key : String = [base_dx.to_string(), String::from(":"), base_dy.to_string()].concat();
            
            if collisions_map.contains_key(&s_key) {
                let (k, mut tuple) = collisions_map.remove_entry(&s_key).unwrap();
                tuple.0 = true;
                collisions_map.insert(k, tuple);
            } else {
                let tuple : CollisionSpace = (true, false, vec![]);
                collisions_map.insert(s_key, tuple);   
            }
        }
    }    
    
    let old_pos_dx = game_state.lock().unwrap().player_two.lock().unwrap().position_dx;
    let additional_dx = game_state.lock().unwrap().player_two.lock().unwrap().velocity_dx * (delta_scalar as f32);
    let mut new_pos_dx = old_pos_dx + additional_dx;
    if new_pos_dx < -1.0 {
        new_pos_dx = new_pos_dx + 2.0;
    }
    if new_pos_dx > 1.0 {
        new_pos_dx = new_pos_dx - 2.0;
    }

    let old_pos_dy = game_state.lock().unwrap().player_two.lock().unwrap().position_dy;
    let additional_dy = game_state.lock().unwrap().player_two.lock().unwrap().velocity_dy * (delta_scalar as f32);
    let mut new_pos_dy = old_pos_dy + additional_dy;
    if new_pos_dy < -1.0 {
        new_pos_dy += 2.0;
    }
    if new_pos_dy > 1.0 {
        new_pos_dy -= 2.0;
    }
    game_state.lock().unwrap().player_two.lock().unwrap().position_dx = new_pos_dx;
    game_state.lock().unwrap().player_two.lock().unwrap().position_dy = new_pos_dy;

    let p2_kx = new_pos_dx;
    let p2_ky = new_pos_dy;

    for i in -10..10 {
        for j in -10..10 {
            let base_dx = ((new_pos_dx * 1000.0) as i32) + i;
            let base_dy = ((new_pos_dy * 1000.0) as i32) + j;

            let s_key : String = [base_dx.to_string(), String::from(":"), base_dy.to_string()].concat();
            
            if collisions_map.contains_key(&s_key) {
                let (k, mut tuple) = collisions_map.remove_entry(&s_key).unwrap();
                tuple.1 = true;
                collisions_map.insert(k, tuple);
            } else {
                let tuple : CollisionSpace = (false, true, vec![]);
                collisions_map.insert(s_key, tuple);   
            }
        }
    } 

    let mut removals: Vec<usize> = vec![];

    let torps_in_flight = game_state.lock().unwrap().torps_in_flight.clone();

    let mut v : Vec<Arc<Mutex<Torp>>> = vec![];
    for (idx, torp) in torps_in_flight.lock().unwrap().iter().enumerate() {
        if !((new_pos_dx < -1.0) || (new_pos_dx > 1.0) || (new_pos_dy < -1.0) || (new_pos_dy > 1.0)) {
            v.push(Arc::new(Mutex::new(*torp.lock().unwrap())));
        } 
    }

    *torps_in_flight.lock().unwrap() = v;

    for (idx, torp) in torps_in_flight.lock().unwrap()
    .iter().enumerate() {
        let pos_dx = torp.lock().unwrap().position_dx;
        let pos_dy = torp.lock().unwrap().position_dy;
        let v_dx = torp.lock().unwrap().velocity_dx;
        let v_dy = torp.lock().unwrap().velocity_dy;
        let new_pos_dx = pos_dx + (delta_scalar * v_dx);
        let new_pos_dy = pos_dy + (delta_scalar * v_dy);

        torp.lock().unwrap().position_dx = new_pos_dx;
        torp.lock().unwrap().position_dy = new_pos_dy;

        // safety fused torps only arm after allowing some time to elapse.
        if torp.lock().unwrap().time_fired.elapsed().as_millis() > 500 {

    
            for i in -5..5 {
                for j in -5..5 {
    
                    let base_dx = ((new_pos_dx * 1000.0) as i32) + i;
                    let base_dy = ((new_pos_dy * 1000.0) as i32) + j;
        
                    let s_key : String = [base_dx.to_string(), String::from(":"), base_dy.to_string()].concat();
                    
                    if collisions_map.contains_key(&s_key) {
                        let (k, mut tuple) = collisions_map.remove_entry(&s_key).unwrap();
                        tuple.2.push(idx);
                        collisions_map.insert(k, tuple);
                    } else {
                        let tuple : CollisionSpace = (false, false, vec![]);
                        collisions_map.insert(s_key, tuple);   
                    }
                }
            }               
        } 
    }

    for (key, space) in collisions_map.iter() {
        if (space.0 == true) && (space.2.len() > 0) {
            // log!("Torpedo kills player one.");            
            *game_state.lock().unwrap().torp_kills_player_1.lock().unwrap() = (true, p1_kx, p1_ky);

        }
        if (space.0 == true) && (space.1 == true) {
            log!("vehicle collision");
        }
        if (space.1 == true) && (space.2.len() > 0) {
            // log!("Torpedo kills player two.");
            *game_state.lock().unwrap().torp_kills_player_2.lock().unwrap() = (true, p2_kx, p2_ky);

        }

    }

    collisions_map.clear();
    // Ok(game_state)
}

