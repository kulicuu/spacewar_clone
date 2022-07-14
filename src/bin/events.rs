
#![allow(unused)]

use crate::structures::{CollisionSpace, GameState, Torp, Vehicle};
use crate::utils;
use crate::utils::time_polyfill::Instant;

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

use std::time::*;
use std::ops::{Add, Sub, AddAssign, SubAssign};
use std::cell::RefCell;
use std::rc::Rc;
use std::convert::{TryInto};
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::f32::consts::PI;

use gloo_console::log;

pub fn set_player_two_events
<'a>
(
    game_state: Arc<Mutex<GameState>>,
)
{
    let document = web_sys::window().unwrap().document().unwrap();
    let et_keys : EventTarget = document.into();
    let keypress_cb = Closure::wrap(Box::new(move |event: KeyboardEvent| {
        match event.key_code() {
    74 => game_state.lock().unwrap().player_two.lock().unwrap().vifo_theta -= Rad(0.1),
    79 => {
        let vniv_scalar = 0.08;
        let vniv_theta = game_state.lock().unwrap().player_two.lock().unwrap().vifo_theta;
        let vniv_dx = Rad::cos(vniv_theta) * vniv_scalar;
        let vniv_dy = Rad::sin(vniv_theta) * vniv_scalar;
        // let vehicle_new_summed_velocity_dx = 
        let vnsv_dx = vniv_dx + game_state.lock().unwrap().player_two.lock().unwrap().velocity_dx;
        let vnsv_dy = vniv_dy + game_state.lock().unwrap().player_two.lock().unwrap().velocity_dy;
        let vnsv_theta = Rad::atan(vnsv_dy / vnsv_dx);
        // let vnsv_scalar = (vnsv_dx as f32) / (Rad::cos(Rad(vnsv_theta)) as f32);
        let vnsv_scalar = vnsv_dx / Rad::cos(vnsv_theta);
        let vnsv_scalar_2 = vnsv_dy / Rad::sin(vnsv_theta);
        // // assert vnvs_scalar == vnsv_scalar_2;
        game_state.lock().unwrap().player_two.lock().unwrap().velocity_dx = vnsv_dx;
        game_state.lock().unwrap().player_two.lock().unwrap().velocity_dy = vnsv_dy;
        game_state.lock().unwrap().player_two.lock().unwrap().velocity_theta = vnsv_theta.into();
        game_state.lock().unwrap().player_two.lock().unwrap().velocity_scalar = vnsv_scalar;
    },
    186 => game_state.lock().unwrap().player_two.lock().unwrap().vifo_theta += Rad(0.1),
    32 => {
        let ticv_scalar = 0.34;
        let ticv_theta = game_state.lock().unwrap().player_two.lock().unwrap().vifo_theta;
        // let torpedo_own_impulse_velocity_dx =
        let ticv_dx = Rad::cos(ticv_theta) * ticv_scalar;
        let ticv_dy = Rad::sin(ticv_theta) * ticv_scalar;
        // let torpedo_summed_velocity_dx =
        let tsv_dx = ticv_dx + game_state.lock().unwrap().player_two.lock().unwrap().velocity_dx;
        let tsv_dy = ticv_dy + game_state.lock().unwrap().player_two.lock().unwrap().velocity_dy;
        // let torpedo_summed_velocity_theta = Rad::atan(tsv_dy / tsv_dx);
        let tsv_theta = Rad::atan(tsv_dy / tsv_dx);
        let tsv_scalar = tsv_dx / Rad::cos(tsv_theta);
        let tsv_scalar_2 = tsv_dy / Rad::sin(tsv_theta);
        // assert tsv_scalar == tsv_scalar_2;
        let t_dx = game_state.lock().unwrap().player_two.lock().unwrap().position_dx;
        let t_dy = game_state.lock().unwrap().player_two.lock().unwrap().position_dy;
        // let time_fired: 
        let mut torpedo = Torp {
            position_dx:  t_dx,
            position_dy: t_dy,
            vifo_theta: ticv_theta,
            velocity_theta: tsv_theta,
            velocity_scalar: tsv_scalar,
            velocity_dx: tsv_dx,
            velocity_dy: tsv_dy,
            time_fired: Instant::now(),
        };
        game_state.lock().unwrap().torps_in_flight.lock().unwrap().push(Arc::new(Mutex::new(torpedo)));
    },
            _ => (),
        }

    }) as Box<dyn FnMut(KeyboardEvent)>);
    et_keys
        .add_event_listener_with_callback("keydown", keypress_cb.as_ref().unchecked_ref())
        .unwrap();
    keypress_cb.forget();
}

pub fn set_player_one_events
<'a>
(
    game_state: Arc<Mutex<GameState>>,
)
{
    let document = web_sys::window().unwrap().document().unwrap();
    let et_keys : EventTarget = document.into();
    let keypress_cb = Closure::wrap(Box::new(move |event: KeyboardEvent| {
        match event.key_code() {
            39 => game_state.lock().unwrap().player_one.lock().unwrap().vifo_theta -= Rad(0.1),
            38 => {
                let vniv_scalar = 0.08;
                let vniv_theta = game_state.lock().unwrap().player_one.lock().unwrap().vifo_theta;
                let vniv_dx = Rad::cos(vniv_theta) * vniv_scalar;
                let vniv_dy = Rad::sin(vniv_theta) * vniv_scalar;
                // let vehicle_new_summed_velocity_dx = 
                let vnsv_dx = vniv_dx + game_state.lock().unwrap().player_one.lock().unwrap().velocity_dx;
                let vnsv_dy = vniv_dy + game_state.lock().unwrap().player_one.lock().unwrap().velocity_dy;
                let vnsv_theta = Rad::atan(vnsv_dy / vnsv_dx);
                // let vnsv_scalar = (vnsv_dx as f32) / (Rad::cos(Rad(vnsv_theta)) as f32);
                let vnsv_scalar = vnsv_dx / Rad::cos(vnsv_theta);
                let vnsv_scalar_2 = vnsv_dy / Rad::sin(vnsv_theta);
                // // assert vnvs_scalar == vnsv_scalar_2;
                game_state.lock().unwrap().player_one.lock().unwrap().velocity_dx = vnsv_dx;
                game_state.lock().unwrap().player_one.lock().unwrap().velocity_dy = vnsv_dy;
                game_state.lock().unwrap().player_one.lock().unwrap().velocity_theta = vnsv_theta.into();
                game_state.lock().unwrap().player_one.lock().unwrap().velocity_scalar = vnsv_scalar;
            },
            37 => game_state.lock().unwrap().player_one.lock().unwrap().vifo_theta += Rad(0.1),
            96 => {
                let ticv_scalar = 0.34;
                let ticv_theta = game_state.lock().unwrap().player_one.lock().unwrap().vifo_theta;
                // let torpedo_own_impulse_velocity_dx =
                let ticv_dx = Rad::cos(ticv_theta) * ticv_scalar;
                let ticv_dy = Rad::sin(ticv_theta) * ticv_scalar;
                // let torpedo_summed_velocity_dx =
                let tsv_dx = ticv_dx + game_state.lock().unwrap().player_one.lock().unwrap().velocity_dx;
                let tsv_dy = ticv_dy + game_state.lock().unwrap().player_one.lock().unwrap().velocity_dy;
                // let torpedo_summed_velocity_theta = Rad::atan(tsv_dy / tsv_dx);
                let tsv_theta = Rad::atan(tsv_dy / tsv_dx);
                let tsv_scalar = tsv_dx / Rad::cos(tsv_theta);
                let tsv_scalar_2 = tsv_dy / Rad::sin(tsv_theta);
                // assert tsv_scalar == tsv_scalar_2;
                let t_dx = game_state.lock().unwrap().player_one.lock().unwrap().position_dx;
                let t_dy = game_state.lock().unwrap().player_one.lock().unwrap().position_dy;
                let mut torpedo = Torp {
                    position_dx:  t_dx,
                    position_dy: t_dy,
                    vifo_theta: ticv_theta,
                    velocity_theta: tsv_theta,
                    velocity_scalar: tsv_scalar,
                    velocity_dx: tsv_dx,
                    velocity_dy: tsv_dy,
                    time_fired: Instant::now(),
                };
                game_state.lock().unwrap().torps_in_flight.lock().unwrap().push(Arc::new(Mutex::new(torpedo)));
            },
            _ => (),
        }

    }) as Box<dyn FnMut(KeyboardEvent)>);
    et_keys
        .add_event_listener_with_callback("keydown", keypress_cb.as_ref().unchecked_ref())
        .unwrap();
    keypress_cb.forget();
}