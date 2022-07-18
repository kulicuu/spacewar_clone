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
use crate::structures::{
    PlayerDrawStuff, 
    GameState,
};

const AMORTIZATION: f32 = 0.95;
const LOCALIZED_SCALE : f32 = 0.001;
const CORRECTION : f32 = LOCALIZED_SCALE / 2.0;
const RESOLUTION : f32 = 8.0;
const SCALE : f32 = 0.08;
const HALF : f32 = SCALE / 2.0;
const STEP : f32 = SCALE / RESOLUTION;
const NUM_PARTICLES : u32 = 9680;


pub fn draw_player() {}


pub fn draw_player_one
(
    gl: Arc<GL>,
    game_state: Arc<Mutex<GameState>>,
    player_draw_stuff: Arc<PlayerDrawStuff>,
)
{

    let shader_program = &player_draw_stuff.shader_program;

    let vertex_buffer = &player_draw_stuff.vertex_buffer;
    let js_vertices = &player_draw_stuff.js_vertices;
    let vertices_position = &player_draw_stuff.vertices_position;
   
    let normals_buffer: &Arc<WebGlBuffer> = &player_draw_stuff.normals_buffer;
    let norms_js: &Arc<js_sys::Float32Array> = &player_draw_stuff.norms_js;
    let vertex_normals_position: &Arc<i32> = &player_draw_stuff.vertex_normals_position;
    
    let norm_mat_loc = &player_draw_stuff.norm_mat_loc;
    let vifo_theta_loc = &player_draw_stuff.vifo_theta_loc;
    let pos_deltas_loc = &player_draw_stuff.pos_deltas_loc;
    let time_loc = &player_draw_stuff.time_loc;
    

    gl.use_program(Some(&shader_program));





    // these shouldn't be done per draw.
    // gl.bind_buffer(GL::ARRAY_BUFFER, Some(&normals_buffer));
    // gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &norms_js, GL::STATIC_DRAW);
    // gl.vertex_attrib_pointer_with_i32(**vertex_normals_position as u32, 3, GL::FLOAT, false, 0, 0);
    // gl.enable_vertex_attrib_array(**vertex_normals_position as u32);
    // gl.bind_buffer(GL::ARRAY_BUFFER, None);

    // gl.bind_buffer(GL::ARRAY_BUFFER, Some(&vertex_buffer));
    // gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &js_vertices, GL::STATIC_DRAW);
    // gl.vertex_attrib_pointer_with_i32(**vertices_position as u32, 3, GL::FLOAT, false, 0, 0);
    // gl.enable_vertex_attrib_array(**vertices_position as u32);
    // gl.bind_buffer(GL::ARRAY_BUFFER, None);


    let eye: cgmath::Point3<f32> = cgmath::Point3::new(0.5, 0.5, 0.5);
    let center: cgmath::Point3<f32> = cgmath::Point3::new(0.0, 0.0, 0.0);
    let up: cgmath::Vector3<f32> = cgmath::Vector3::new(0.0, 0.0, 1.0);
    let m100: cgmath::Matrix4<f32> = cgmath::Matrix4::look_at_rh(eye, center, up);




    // log!("vertex_normals_poisition: ", **vertex_normals_position);
    // log!("vertices position", **vertices_position);


    // gl.uniform_matrix4fv_with_f32_sequence(
    //     &Some(&norm_mat_loc),
        
    // );


    gl.uniform1f(Some(&time_loc), 0.4 as f32);
    let new_pos_dx = game_state.lock().unwrap().player_one.lock().unwrap().position_dx;
    let new_pos_dy = game_state.lock().unwrap().player_one.lock().unwrap().position_dy;
    gl.uniform2f(Some(&pos_deltas_loc), new_pos_dx, new_pos_dy);    
    let new_vifo_theta = game_state.lock().unwrap().player_one.lock().unwrap().vifo_theta;
    gl.uniform1f(Some(&vifo_theta_loc), new_vifo_theta.0);
    // gl.draw_arrays(GL::TRIANGLES, 0, 6);
    gl.draw_arrays(GL::TRIANGLES, 0, 36);
    gl.bind_buffer(GL::ARRAY_BUFFER, None);
}

pub fn draw_player_two
(
    gl: Arc<GL>,
    game_state: Arc<Mutex<GameState>>,
    player_draw_stuff: Arc<PlayerDrawStuff>,
)
{
    let shader_program = &player_draw_stuff.shader_program;
    let vertex_buffer = &player_draw_stuff.vertex_buffer;
    let js_vertices = &player_draw_stuff.js_vertices;
    let vertices_position = &player_draw_stuff.vertices_position;
    let vifo_theta_loc = &player_draw_stuff.vifo_theta_loc;
    let pos_deltas_loc = &player_draw_stuff.pos_deltas_loc;
    let time_loc = &player_draw_stuff.time_loc;
    
    gl.use_program(Some(&shader_program));
    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&vertex_buffer));
    gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &js_vertices, GL::STATIC_DRAW);
    gl.vertex_attrib_pointer_with_i32(**vertices_position as u32, 3, GL::FLOAT, false, 0, 0);
    gl.enable_vertex_attrib_array(**vertices_position as u32);
    gl.uniform1f(Some(&time_loc), 0.4 as f32);
    let new_pos_dx = game_state.lock().unwrap().player_two.lock().unwrap().position_dx;
    let new_pos_dy = game_state.lock().unwrap().player_two.lock().unwrap().position_dy;
    gl.uniform2f(Some(&pos_deltas_loc), new_pos_dx, new_pos_dy);    
    let new_vifo_theta = game_state.lock().unwrap().player_two.lock().unwrap().vifo_theta;
    gl.uniform1f(Some(&vifo_theta_loc), new_vifo_theta.0);
    gl.draw_arrays(GL::TRIANGLES, 0, 36);
    gl.bind_buffer(GL::ARRAY_BUFFER, None);
}


// setup shaders and some objects:
pub fn setup_prepare_player_draw
(gl: Arc<GL>)
-> Result<Arc<PlayerDrawStuff>, String>
{

    let vehicle_100_vertices: Vec<f32> = vec![
        0.021, 0.0, 
         -0.008, -0.008,
        -0.008, 0.008,
    ];


    let vehicle_200_vertices: Vec<f32> = vec![
        0.021, 0.0, 0.0,
         -0.008, -0.008, 0.0,
        -0.008, 0.008, 0.0, // that's the base triangle of the pseudo-tetrahedron. 

        0.021, 0.0, 0.0,
        -0.008, 0.008, 0.0,
        -0.005, 0.0, 0.005,

        0.021, 0.0, 0.0,
        -0.008, -0.008, 0.0,
        -0.005, 0.0, 0.005,

        -0.008, -0.008, 0.0,
        -0.008, 0.008, 0.0,
        -0.005, 0.0, 0.005,
    ];

    let v1 = cgmath::Vector3::new(vehicle_200_vertices[0], vehicle_200_vertices[1], vehicle_200_vertices[2]);
    let v2 = cgmath::Vector3::new(vehicle_200_vertices[3], vehicle_200_vertices[4], vehicle_200_vertices[5]);
    let v3 = cgmath::Vector3::new(vehicle_200_vertices[6], vehicle_200_vertices[7], vehicle_200_vertices[8]);

    let u1 = v1 - v2;
    let u2 = v3 - v2;

    let face_1_normal = u1.cross(u2);

    let v1 = cgmath::Vector3::new(vehicle_200_vertices[9], vehicle_200_vertices[10], vehicle_200_vertices[11]);
    let v2 = cgmath::Vector3::new(vehicle_200_vertices[12], vehicle_200_vertices[13], vehicle_200_vertices[14]);
    let v3 = cgmath::Vector3::new(vehicle_200_vertices[15], vehicle_200_vertices[16], vehicle_200_vertices[17]);

    let u1 = v1 - v2;
    let u2 = v3 - v2;

    let face_2_normal = u1.cross(u2);

    let v1 = cgmath::Vector3::new(vehicle_200_vertices[18], vehicle_200_vertices[19], vehicle_200_vertices[20]);
    let v2 = cgmath::Vector3::new(vehicle_200_vertices[21], vehicle_200_vertices[22], vehicle_200_vertices[23]);
    let v3 = cgmath::Vector3::new(vehicle_200_vertices[24], vehicle_200_vertices[25], vehicle_200_vertices[26]);

    let u1 = v1 - v2;
    let u2 = v3 - v2;

    let face_3_normal = u1.cross(u2);

    let v1 = cgmath::Vector3::new(vehicle_200_vertices[27], vehicle_200_vertices[28], vehicle_200_vertices[29]);
    let v2 = cgmath::Vector3::new(vehicle_200_vertices[30], vehicle_200_vertices[31], vehicle_200_vertices[32]);
    let v3 = cgmath::Vector3::new(vehicle_200_vertices[33], vehicle_200_vertices[34], vehicle_200_vertices[35]);

    let u1 = v1 - v2;
    let u2 = v3 - v2;

    let face_4_normal = u1.cross(u2);

    let v200_vertex_normals = vec![
        face_1_normal[0], face_1_normal[1], face_1_normal[2],
        face_1_normal[0], face_1_normal[1], face_1_normal[2],
        face_1_normal[0], face_1_normal[1], face_1_normal[2],

        face_2_normal[0], face_2_normal[1], face_2_normal[2],
        face_2_normal[0], face_2_normal[1], face_2_normal[2],
        face_2_normal[0], face_2_normal[1], face_2_normal[2],

        face_3_normal[0], face_3_normal[1], face_3_normal[2],
        face_3_normal[0], face_3_normal[1], face_3_normal[2],
        face_3_normal[0], face_3_normal[1], face_3_normal[2],

        face_4_normal[0], face_4_normal[1], face_4_normal[2],
        face_4_normal[0], face_4_normal[1], face_4_normal[2],
        face_4_normal[0], face_4_normal[1], face_4_normal[2],
    ];

    // let vert_code = include_str!("../shaders/vehicle_100.vert");
    let vert_code = include_str!("../shaders/vehicle_200.vert");
    let vert_shader = gl.create_shader(GL::VERTEX_SHADER).unwrap();
    gl.shader_source(&vert_shader, vert_code);
    gl.compile_shader(&vert_shader);
    let vert_shader_log = gl.get_shader_info_log(&vert_shader);
    log!("player vert shader log: ", vert_shader_log);
    
    // let frag_code = include_str!("../shaders/basic.frag");
    let frag_code = include_str!("../shaders/vehicle_200.frag");
    let frag_shader = gl.create_shader(GL::FRAGMENT_SHADER).unwrap();
    gl.shader_source(&frag_shader, frag_code);
    gl.compile_shader(&frag_shader);
    let frag_shader_log = gl.get_shader_info_log(&frag_shader);
    log!("player frag shader log", frag_shader_log);

    let shader_program = Arc::new(gl.create_program().unwrap());
    gl.attach_shader(&shader_program, &vert_shader);
    gl.attach_shader(&shader_program, &frag_shader);
    gl.link_program(&shader_program);
    
    let time_loc = Arc::new(gl.get_uniform_location(&shader_program, "u_time").unwrap());

    let normals_buffer = Arc::new(gl.create_buffer().unwrap());
    let norms_js = Arc::new(js_sys::Float32Array::from(v200_vertex_normals.as_slice()));
    let vertex_normals_position = Arc::new(gl.get_attrib_location(&shader_program, "a_normal") as i32);

    let vertex_buffer = Arc::new(gl.create_buffer().unwrap());
    // let js_vertices = Arc::new(js_sys::Float32Array::from(vehicle_100_vertices.as_slice()));
    let js_vertices = Arc::new(js_sys::Float32Array::from(vehicle_200_vertices.as_slice()));
    let vertices_position = Arc::new(gl.get_attrib_location(&shader_program, "a_position") as i32);

    let norm_mat_loc = Arc::new(gl.get_uniform_location(&shader_program, "norm_mat").unwrap());
    let pos_deltas_loc = Arc::new(gl.get_uniform_location(&shader_program, "pos_deltas").unwrap());
    let vifo_theta_loc =  Arc::new(gl.get_uniform_location(&shader_program, "vifo_theta").unwrap());


    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&normals_buffer));
    gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &norms_js, GL::STATIC_DRAW);
    gl.vertex_attrib_pointer_with_i32(*vertex_normals_position as u32, 3, GL::FLOAT, false, 0, 0);
    gl.enable_vertex_attrib_array(*vertex_normals_position as u32);
    gl.bind_buffer(GL::ARRAY_BUFFER, None);

    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&vertex_buffer));
    gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &js_vertices, GL::STATIC_DRAW);
    gl.vertex_attrib_pointer_with_i32(*vertices_position as u32, 3, GL::FLOAT, false, 0, 0);
    gl.enable_vertex_attrib_array(*vertices_position as u32);
    gl.bind_buffer(GL::ARRAY_BUFFER, None);


    Ok(
        Arc::new(
            PlayerDrawStuff {
                shader_program: shader_program,

                vertex_buffer: vertex_buffer,
                js_vertices: js_vertices,
                vertices_position: vertices_position,

                normals_buffer: normals_buffer,
                norms_js: norms_js,
                vertex_normals_position: vertex_normals_position,

                norm_mat_loc: norm_mat_loc,
                pos_deltas_loc: pos_deltas_loc,
                vifo_theta_loc: vifo_theta_loc,
                time_loc: time_loc
            }
        )
    )
}