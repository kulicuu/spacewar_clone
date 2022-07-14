#![allow(unused)]

mod utils;
mod game_state;
mod events;
mod explosion;
mod structures;
mod player;
mod torp;

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

use crate::utils::time_polyfill::Instant;

const AMORTIZATION: f32 = 0.95;
const LOCALIZED_SCALE : f32 = 0.001;
const CORRECTION : f32 = LOCALIZED_SCALE / 2.0;
const RESOLUTION : f32 = 8.0;
const SCALE : f32 = 0.08;
const HALF : f32 = SCALE / 2.0;
const STEP : f32 = SCALE / RESOLUTION;
const NUM_PARTICLES : u32 = 9680;

fn main()
{
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas33").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

    let gl: GL = canvas
        .get_context("webgl2")
        .unwrap()
        .unwrap()
        .dyn_into::<GL>()
        .unwrap();
    let gl : Arc<GL> = Arc::new(gl);


    

    // let vertex_array_a = explosion_stuff.vertex_array_a;
    // let vertex_array_b = explosion_stuff.vertex_array_b;
    // let transform_feedback_a = explosion_stuff.transform_feedback_a;
    // let transform_feedback_b = explosion_stuff.transform_feedback_b;
    // let position_buffer_a = explosion_stuff.position_buffer_a;
    // let position_buffer_b = explosion_stuff.position_buffer_b;
    // let velocity_buffer_a = explosion_stuff.velocity_buffer_a;
    // let velocity_buffer_b = explosion_stuff.velocity_buffer_b;
    // let color_buffer = explosion_stuff.color_buffer;
    // let current_transform_feedback = explosion_stuff.current_transform_feedback;
    // let current_vertex_array = explosion_stuff.current_vertex_array;
    // let position_data = explosion_stuff.position_data;
    // let velocity_data = explosion_stuff.velocity_data;
    // let color_data = explosion_stuff.color_data;

    let expl_shader_program = explosion::setup_shader(gl.clone()).unwrap();
    let explosion_stuff = explosion::prepare_explosion(gl.clone()).unwrap();
    
    
    let player_draw_stuff = player::setup_prepare_player_draw(gl.clone()).unwrap();
    let torp_draw_stuff = torp::setup_prepare_torp_draw(gl.clone()).unwrap();

    let mut game = game_state::create_game_state().unwrap(); // Todo: fix naming conflict between module and object.
    events::set_player_one_events(game.clone());
    events::set_player_two_events(game.clone());



    let start_time = Instant::now();
    let mut cursor: u128 = start_time.elapsed().as_millis();

    let mut expl_switch = Arc::new(Mutex::new(AtomicBool::new(true)));

    gl.clear_color(0.98, 0.983, 0.992, 1.0);

    let render_loop_closure = Rc::new(RefCell::new(None));
    let alias_rlc = render_loop_closure.clone();
    *alias_rlc.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let now = start_time.elapsed().as_millis();  // total elapsed time from start
        let frame_delta = now - cursor;
        cursor = now;

        gl.clear(GL::COLOR_BUFFER_BIT);


        game_state::update_game_state(frame_delta, game.clone());



        let tkp1 = *(game.lock().unwrap().torp_kills_player_1.lock().unwrap());
        if tkp1.0 {

            let offset_uniform_data = [tkp1.1, tkp1.2];
            
            explosion::set_uniforms(
                gl.clone(),
                offset_uniform_data,
            );

            explosion::draw_explosion(
                gl.clone(),
                expl_shader_program.clone(),
                explosion_stuff.clone(),
                expl_switch.clone(),
            )

        } else {
            player::draw_player_one(
                gl.clone(),
                game.clone(),
                player_draw_stuff.clone(),
            );
        }



        let tkp2 = *(game.lock().unwrap().torp_kills_player_2.lock().unwrap());
        if tkp2.0 {

            let offset_uniform_data = [tkp2.1, tkp2.2];
            
            explosion::set_uniforms(
                gl.clone(),
                offset_uniform_data,
            );

            explosion::draw_explosion(
                gl.clone(),
                expl_shader_program.clone(),
                explosion_stuff.clone(),
                expl_switch.clone(),
            )

        } else {
            player::draw_player_two(
                gl.clone(),
                game.clone(),
                player_draw_stuff.clone(),
            );
        }





        torp::draw_torps(
            gl.clone(),
            game.clone(),
            torp_draw_stuff.clone(),
        );





        request_animation_frame(render_loop_closure.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(alias_rlc.borrow().as_ref().unwrap());    
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window().unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}


