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

use crate::utils::time_polyfill::Instant;

const AMORTIZATION: f32 = 0.95;
const LOCALIZED_SCALE : f32 = 0.001;
const CORRECTION : f32 = LOCALIZED_SCALE / 2.0;
const RESOLUTION : f32 = 8.0;
const SCALE : f32 = 0.08;
const HALF : f32 = SCALE / 2.0;
const STEP : f32 = SCALE / RESOLUTION;
const NUM_PARTICLES : u32 = 9680;

#[derive(Copy, Clone)]
pub struct Torp {
    pub position_dx: f32, // raw displacement in x, y
    pub position_dy: f32,
    // vehicle_inertial_frame_orientation_theta: f32,
    pub vifo_theta: Rad<f32>,
    // polar description
    pub velocity_theta: Rad<f32>,
    pub velocity_scalar: f32,
    // redundant alternate description of velocity, cartesian
    pub velocity_dx: f32,
    pub velocity_dy: f32, 
    pub time_fired: Instant,
}

#[derive(Copy, Clone)]
pub struct Vehicle {
    pub position_dx: f32, // raw displacement in x, y
    pub position_dy: f32,
    // vehicle_inertial_frame_orientation_theta: f32,
    pub vifo_theta: Rad<f32>,
    // polar description
    pub velocity_theta: Rad<f32>,
    pub velocity_scalar: f32,
    // redundant alternate description of velocity, cartesian
    pub velocity_dx: f32,
    pub velocity_dy: f32, 
}

// type CollisionSpace = (bool, bool, Arc<Mutex<Vec<usize>>>);
pub type CollisionSpace = (bool, bool, Vec<usize>);

pub struct GameState 
{
    pub player_one: Arc<Mutex<Vehicle>>,
    pub player_two:  Arc<Mutex<Vehicle>>,
    pub torps_in_flight: Arc<Mutex<Vec<Arc<Mutex<Torp>>>>>,
    pub start_time: Arc<Instant>,
    pub elapsed_time: Arc<Mutex<u128>>,
    pub game_over: Arc<Mutex<bool>>,
    // collisions_map: Arc<Mutex<HashMap<&'static str, Arc<Mutex<CollisionSpace>>>>>,
    pub collisions_map: Arc<Mutex<HashMap<String, Arc<Mutex<CollisionSpace>>>>>,
    pub result: Arc<Mutex<u8>>,
    pub mode: Arc<u8>, // 1 player vs computer, 2 player local, 2 player network
    pub torp_kills_player_1: Arc<Mutex<(bool, f32, f32)>>,
    pub torp_kills_player_2: Arc<Mutex<(bool, f32, f32)>>,
}

#[derive(Clone)]
pub struct ExplosionStuff {
    pub vertex_array_a: Arc<web_sys::WebGlVertexArrayObject>,
    pub vertex_array_b: Arc<web_sys::WebGlVertexArrayObject>,
    pub transform_feedback_a: Arc<web_sys::WebGlTransformFeedback>,
    pub transform_feedback_b: Arc<web_sys::WebGlTransformFeedback>,
    pub position_buffer_a: Arc<web_sys::WebGlBuffer>,
    pub position_buffer_b: Arc<web_sys::WebGlBuffer>,
    pub velocity_buffer_a: Arc<web_sys::WebGlBuffer>,
    pub velocity_buffer_b: Arc<web_sys::WebGlBuffer>,
    pub color_buffer: Arc<web_sys::WebGlBuffer>,
    pub current_transform_feedback: Arc<Mutex<web_sys::WebGlTransformFeedback>>,
    pub current_vertex_array: Arc<Mutex<web_sys::WebGlVertexArrayObject>>,
    pub position_data: Arc<Mutex<[f32; (NUM_PARTICLES * 3) as usize]>>,
    pub velocity_data: Arc<Mutex<[f32; (NUM_PARTICLES * 3) as usize]>>,
    pub color_data: Arc<Mutex<[f32; (NUM_PARTICLES * 3) as usize]>>,
}

#[derive(Clone)]
pub struct PlayerDrawStuff { // Stuff for drawing. Todo: better naming.
    pub shader_program: Arc<web_sys::WebGlProgram>, // player_shader_program,

    pub vertex_buffer: Arc<WebGlBuffer>, // player_vertex_buffer
    pub js_vertices: Arc<js_sys::Float32Array>, // player_js_vertices,
    pub vertices_position: Arc<i32>, // player_vertices_position

    pub normals_buffer: Arc<WebGlBuffer>,
    pub norms_js: Arc<js_sys::Float32Array>, // player_js_vertices,
    pub vertex_normals_position: Arc<i32>,
    
    pub pos_deltas_loc: Arc<WebGlUniformLocation>, //player_pos_deltas_loc
    pub vifo_theta_loc: Arc<WebGlUniformLocation>, //player_vifo_theta_loc
    pub time_loc: Arc::<WebGlUniformLocation>, // time_location
}

pub struct TorpDrawStuff { 
    pub shader_program: Arc<web_sys::WebGlProgram>, 
    pub vertex_buffer: Arc<WebGlBuffer>, 
    pub js_vertices: Arc<js_sys::Float32Array>, 
    pub vertices_position: Arc<u32>,
    pub pos_deltas_loc: Arc<WebGlUniformLocation>, 
    pub vifo_theta_loc: Arc<WebGlUniformLocation>, 
    pub time_loc: Arc::<WebGlUniformLocation>, 
}


