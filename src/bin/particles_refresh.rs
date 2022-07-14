#![allow(unused)]
#![feature(drain_filter)]

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
use std::time::*;

use std::convert::{TryInto};
use std::ops::{Add, Sub, AddAssign, SubAssign};
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;

use gloo_console::log;
use std::f32::consts::PI;

const AMORTIZATION: f32 = 0.95;

const LOCALIZED_SCALE : f32 = 0.9;
const CORRECTION : f32 = LOCALIZED_SCALE / 2.0;
const RESOLUTION : f32 = 8.0;
const SCALE : f32 = 0.08;
const HALF : f32 = SCALE / 2.0;
const STEP : f32 = SCALE / RESOLUTION;
const NUM_PARTICLES : u32 = 9680;

pub fn particles_refresh()
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

    let (
        player_vertex_buffer,
        player_js_vertices,
        player_shader_program,
        player_vertices_position,
        player_pos_deltas_loc,
        player_vifo_theta_loc,
        time_location,
    ) = setup_shaders(gl.clone()).unwrap();

    let (
        torp_vertex_buffer,
        torp_js_vertices,
        torp_shader_program,
        torp_vertices_position,
        torp_pos_deltas_loc,
        torp_vifo_theta_loc,
        time_location_2,
    ) = setup_torp_shaders(gl.clone()).unwrap();

    let mut switch = Arc::new(Mutex::new(AtomicBool::new(true)));

    let game_state = create_game_state().unwrap();

    set_player_one_events(game_state.clone());
    set_player_two_events(game_state.clone());

    // let game_state = game_state.clone();
    let mut cursor = game_state.lock().unwrap().start_time.elapsed().as_millis();

    gl.clear_color(0.99, 0.99, 0.99, 1.0);
    // gl.clear_color(0.01, 0.01, 0.01, 1.0);
    gl.enable(GL::BLEND);
    gl.blend_func(GL::ONE, GL::ONE_MINUS_SRC_ALPHA);

    // let bare_shader_program = setup_bare_test_shaders(gl.clone()).unwrap();

    // let particles_stuff = prepare_particles(gl.clone).unwrap();
    let particles_stuff = Arc::new(prepare_particles(gl.clone()).unwrap());


    let render_loop_closure = Rc::new(RefCell::new(None));
    let alias_rlc = render_loop_closure.clone();
    *alias_rlc.borrow_mut() = Some(Closure::wrap(Box::new(move || {

        // Redo this.
        // let now = game_state.lock().unwrap().start_time.elapsed().as_millis(); 
        // let time_delta = now - cursor;
        // cursor = now;

        gl.clear(GL::COLOR_BUFFER_BIT);

        let game_state = update_game_state(time_delta, game_state.clone()).unwrap();




        let tkp1 = *game_state.lock().unwrap().torp_kills_player_1.lock().unwrap();


        draw_particles(
            gl.clone(),
            particles_stuff.shader_program.clone(),
            particles_stuff.vertex_array_a.clone(),
            particles_stuff.vertex_array_b.clone(),
            particles_stuff.current_vertex_array.clone(),
            particles_stuff.current_transform_feedback.clone(),
            particles_stuff.transform_feedback_a.clone(),
            particles_stuff.transform_feedback_b.clone(),
            particles_stuff.position_buffer_a.clone(),
            particles_stuff.velocity_buffer_a.clone(),
            particles_stuff.position_buffer_b.clone(),
            particles_stuff.velocity_buffer_b.clone(),
            switch.clone(),
            // particles_stuff.offset_uniforms_loc.clone(),
            0.0,
            0.0,
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

fn draw_particles
(
    gl: Arc<GL>,
    shader_program: Arc<web_sys::WebGlProgram>,
    vertex_array_a: Arc<web_sys::WebGlVertexArrayObject>,
    vertex_array_b: Arc<web_sys::WebGlVertexArrayObject>,
    mut current_vertex_array: Arc<Mutex<web_sys::WebGlVertexArrayObject>>,
    mut current_transform_feedback: Arc<Mutex<web_sys::WebGlTransformFeedback>>,
    transform_feedback_a: Arc<web_sys::WebGlTransformFeedback>,
    transform_feedback_b: Arc<web_sys::WebGlTransformFeedback>,
    position_buffer_a: Arc<web_sys::WebGlBuffer>,
    velocity_buffer_a: Arc<web_sys::WebGlBuffer>,
    position_buffer_b: Arc<web_sys::WebGlBuffer>,
    velocity_buffer_b: Arc<web_sys::WebGlBuffer>,
    switch: Arc<Mutex<AtomicBool>>,
    // offset_uniforms_loc: Arc<u32>,
    kx: f32,
    ky: f32,
)   
{
    // gl.link_program(&shader_program);
    gl.use_program(Some(&shader_program));
    
    gl.bind_vertex_array(Some(current_vertex_array.lock().unwrap().as_ref()));
    gl.bind_transform_feedback(GL::TRANSFORM_FEEDBACK, Some(current_transform_feedback.lock().unwrap().as_ref()));

    let s = *switch.lock().unwrap().get_mut();
    // if *current_transform_feedback.lock().unwrap() == *transform_feedback_a.lock().unwrap() {
    if !s {
        gl.bind_buffer_base(GL::TRANSFORM_FEEDBACK_BUFFER, 0, Some(&position_buffer_a));
        gl.bind_buffer_base(GL::TRANSFORM_FEEDBACK_BUFFER, 1, Some(&velocity_buffer_a));
    } else {
        gl.bind_buffer_base(GL::TRANSFORM_FEEDBACK_BUFFER, 0, Some(&position_buffer_b));
        gl.bind_buffer_base(GL::TRANSFORM_FEEDBACK_BUFFER, 1, Some(&velocity_buffer_b));
    }

    gl.begin_transform_feedback(GL::POINTS);
    gl.draw_arrays(GL::POINTS, 0, NUM_PARTICLES as i32);
    gl.end_transform_feedback();

    if s {
        *current_vertex_array.lock().unwrap() = (*vertex_array_b).clone();
        *current_transform_feedback.lock().unwrap() = (*transform_feedback_a).clone();
    } else 
    {
        *current_vertex_array.lock().unwrap() = (*vertex_array_a).clone();
        *current_transform_feedback.lock().unwrap() = (*transform_feedback_b).clone();
    }
    *switch.lock().unwrap().get_mut() = !s; 
    gl.bind_vertex_array(None);
}

fn setup_torp_shaders
<'a>
(
    gl: Arc<GL>,
)
-> Result<(
    Arc<WebGlBuffer>, // torp vertex buffer
    Arc<js_sys::Float32Array>, // torp_js_vertices,
    Arc<web_sys::WebGlProgram>, // torp_shader_program,
    Arc<u32>, // torp_vertices_position
    Arc<WebGlUniformLocation>, //torp_pos_deltas_loc
    Arc<WebGlUniformLocation>, // torp_vifo_theta_loc
    Arc::<WebGlUniformLocation>, // time_location
), &'a str>
{
    let torpedo_100_vertices: Vec<f32> = vec![
        0.007, 0.0,
        -0.0038, -0.0038, 
        -0.0038, 0.0038, 
    ];

    let torpedo_100_vert_code = include_str!("../shaders/torpedo_100.vert");
    let torpedo_100_vert_shader = gl.create_shader(GL::VERTEX_SHADER).unwrap();
    gl.shader_source(&torpedo_100_vert_shader, torpedo_100_vert_code);
    gl.compile_shader(&torpedo_100_vert_shader);
    let torpedo_100_vert_shader_log = gl.get_shader_info_log(&torpedo_100_vert_shader);
    log!("torpedo_100 shader log: ", torpedo_100_vert_shader_log);

    let frag_code = include_str!("../shaders/basic.frag");
    let frag_shader = gl.create_shader(GL::FRAGMENT_SHADER).unwrap();
    gl.shader_source(&frag_shader, frag_code);
    gl.compile_shader(&frag_shader);
    let basic_frag_shader_log = gl.get_shader_info_log(&frag_shader);

    let torp_shader_program = gl.create_program().unwrap();
    gl.attach_shader(&torp_shader_program, &torpedo_100_vert_shader);
    gl.attach_shader(&torp_shader_program, &frag_shader);
    gl.link_program(&torp_shader_program);

    let time_location = gl.get_uniform_location(&torp_shader_program, "u_time");

    let torp_vertex_buffer = gl.create_buffer().unwrap();
    let torp_js_vertices = js_sys::Float32Array::from(torpedo_100_vertices.as_slice());
    let torp_pos_deltas_loc = gl.get_uniform_location(&torp_shader_program, "pos_deltas");
    let torp_vifo_theta_loc =  gl.get_uniform_location(&torp_shader_program, "vifo_theta");
    let torp_vertices_position = gl.get_attrib_location(&torp_shader_program, "b_position") as u32;

    let frag_code = include_str!("../shaders/basic.frag");
    let frag_shader = gl.create_shader(GL::FRAGMENT_SHADER).unwrap();
    gl.shader_source(&frag_shader, frag_code);
    gl.compile_shader(&frag_shader);
    let basic_frag_shader_log = gl.get_shader_info_log(&frag_shader);

    Ok((
        Arc::new(torp_vertex_buffer),
        Arc::new(torp_js_vertices),
        Arc::new(torp_shader_program),
        Arc::new(torp_vertices_position),
        Arc::new(torp_pos_deltas_loc.unwrap()),
        Arc::new(torp_vifo_theta_loc.unwrap()),
        Arc::new(time_location.unwrap()),
    ))
}

// fn setup_explosion_shaders
// <'a>
// (
//     gl: Arc<GL>,
// )
// -> Result<
//         Arc<web_sys::WebGlProgram>
// , &'a str>
// {
//     let vert_code = include("")

//     let particles_vert_code = include_str!("../shaders/particles.vert");
//     let particles_vert_shader = gl.create_shader(GL::VERTEX_SHADER).unwrap();
//     gl.shader_source(&particles_vert_shader, particles_vert_code);
//     gl.compile_shader(&particles_vert_shader);
//     let particles_vert_shader_log = gl.get_shader_info_log(&particles_vert_shader);
//     log!("particles_vert_shader log: ", particles_vert_shader_log);

//     let particles_frag_code = include_str!("../shaders/particles.frag");
//     let particles_frag_shader = gl.create_shader(GL::FRAGMENT_SHADER).unwrap();
//     gl.shader_source(&particles_frag_shader, particles_frag_code);
//     gl.compile_shader(&particles_frag_shader);
//     let particles_frag_shader_log = gl.get_shader_info_log(&particles_vert_shader);
//     log!("particles_frag_shader log: ", particles_frag_shader_log);


//     let particles_shader_program = gl.create_program().unwrap();
//     gl.attach_shader(&particles_shader_program, &particles_vert_shader);
//     gl.attach_shader(&particles_shader_program, &particles_frag_shader);

//     let data = r#"
//         ["vPosition", "vVelocity"]
//         "#;
//     let v: Value = serde_json::from_str(data).unwrap();
//     let js_val = wasm_bindgen::JsValue::from_serde(&v).unwrap();
//     gl.transform_feedback_varyings(&particles_shader_program, &js_val, GL::SEPARATE_ATTRIBS);
    
//     gl.link_program(&particles_shader_program);
//     gl.use_program(Some(&particles_shader_program));

//     Ok(Arc::new(particles_shader_program))
// }


fn setup_particle_shaders
<'a>
(
    gl: Arc<GL>,
)
-> Result<
        Arc<web_sys::WebGlProgram> // particles_shader_pragram
, &'a str>
{
    let particles_vert_code = include_str!("../shaders/particles.vert");
    let particles_vert_shader = gl.create_shader(GL::VERTEX_SHADER).unwrap();
    gl.shader_source(&particles_vert_shader, particles_vert_code);
    gl.compile_shader(&particles_vert_shader);
    let particles_vert_shader_log = gl.get_shader_info_log(&particles_vert_shader);
    log!("particles_vert_shader log: ", particles_vert_shader_log);

    let particles_frag_code = include_str!("../shaders/particles.frag");
    let particles_frag_shader = gl.create_shader(GL::FRAGMENT_SHADER).unwrap();
    gl.shader_source(&particles_frag_shader, particles_frag_code);
    gl.compile_shader(&particles_frag_shader);
    let particles_frag_shader_log = gl.get_shader_info_log(&particles_vert_shader);
    log!("particles_frag_shader log: ", particles_frag_shader_log);


    let particles_shader_program = gl.create_program().unwrap();
    gl.attach_shader(&particles_shader_program, &particles_vert_shader);
    gl.attach_shader(&particles_shader_program, &particles_frag_shader);

    let data = r#"
        ["vPosition", "vVelocity"]
        "#;
    let v: Value = serde_json::from_str(data).unwrap();
    let js_val = wasm_bindgen::JsValue::from_serde(&v).unwrap();
    gl.transform_feedback_varyings(&particles_shader_program, &js_val, GL::SEPARATE_ATTRIBS);
    
    gl.link_program(&particles_shader_program);
    gl.use_program(Some(&particles_shader_program));

    Ok(Arc::new(particles_shader_program))
}

fn setup_shaders
<'a>
(
    gl: Arc<GL>,
)
-> Result<(
    Arc<WebGlBuffer>, // player_vertex_buffer
    Arc<js_sys::Float32Array>, // player_js_vertices,
    Arc<web_sys::WebGlProgram>, // player_shader_program,
    Arc<u32>, // player_vertices_position
    Arc<WebGlUniformLocation>, //player_pos_deltas_loc
    Arc<WebGlUniformLocation>, //player_vifo_theta_loc
    Arc::<WebGlUniformLocation>, // time_location
), &'a str>
{
    let vehicle_100_vertices: Vec<f32> = vec![
        0.021, 0.0, 
         -0.008, -0.008,
        -0.008, 0.008,
    ];

    let vehicle_100_vert_code = include_str!("../shaders/vehicle_100.vert");
    let vehicle_100_vert_shader = gl.create_shader(GL::VERTEX_SHADER).unwrap();
    gl.shader_source(&vehicle_100_vert_shader, vehicle_100_vert_code);
    gl.compile_shader(&vehicle_100_vert_shader);
    let vehicle_100_vert_shader_log = gl.get_shader_info_log(&vehicle_100_vert_shader);
    log!("vehicle_100 shader log: ", vehicle_100_vert_shader_log);
    
    let frag_code = include_str!("../shaders/basic.frag");
    let frag_shader = gl.create_shader(GL::FRAGMENT_SHADER).unwrap();
    gl.shader_source(&frag_shader, frag_code);
    gl.compile_shader(&frag_shader);
    let basic_frag_shader_log = gl.get_shader_info_log(&frag_shader);

    let player_shader_program = gl.create_program().unwrap();
    gl.attach_shader(&player_shader_program, &vehicle_100_vert_shader);
    gl.attach_shader(&player_shader_program, &frag_shader);
    gl.link_program(&player_shader_program);
    
    let time_location = gl.get_uniform_location(&player_shader_program, "u_time");

    let player_vertex_buffer = gl.create_buffer().unwrap();
    let player_js_vertices = js_sys::Float32Array::from(vehicle_100_vertices.as_slice());
    let player_pos_deltas_loc = gl.get_uniform_location(&player_shader_program, "pos_deltas");
    let player_vifo_theta_loc =  gl.get_uniform_location(&player_shader_program, "vifo_theta");
    let player_vertices_position = gl.get_attrib_location(&player_shader_program, "a_position") as u32;
    
    Ok((
        Arc::new(player_vertex_buffer),
        Arc::new(player_js_vertices),
        Arc::new(player_shader_program),
        Arc::new(player_vertices_position),
        Arc::new(player_pos_deltas_loc.unwrap()),
        Arc::new(player_vifo_theta_loc.unwrap()),
        Arc::new(time_location.unwrap()),
    ))
}

fn set_player_two_events
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
        let mut torpedo = Torp200 {
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

fn set_player_one_events
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
                let mut torpedo = Torp200 {
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

fn draw_torps
(
    gl: Arc<GL>,
    game_state: Arc<Mutex<GameState>>,
    torp_vertex_buffer: Arc<WebGlBuffer>,
    torp_js_vertices: Arc<js_sys::Float32Array>,
    shader_program: Arc<web_sys::WebGlProgram>,
    torp_vertices_position: Arc<u32>,
    time_location: Arc<WebGlUniformLocation>,
    torp_pos_deltas_loc: Arc<WebGlUniformLocation>,
    torp_vifo_theta_loc: Arc<WebGlUniformLocation>,
)
{
    gl.use_program(Some(&shader_program));
    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&torp_vertex_buffer));
    gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &torp_js_vertices, GL::STATIC_DRAW);
    gl.vertex_attrib_pointer_with_i32(*torp_vertices_position, 2, GL::FLOAT, false, 0, 0);
    gl.enable_vertex_attrib_array(*torp_vertices_position);

    // gl.use_program(Some(&shader_program));
    gl.uniform1f(Some(&time_location), 0.4 as f32);

    for (idx, torp) in game_state.lock().unwrap()
    .torps_in_flight.lock().unwrap()
    .iter().enumerate() {
        let new_pos_dx = torp.lock().unwrap().position_dx;
        let new_pos_dy = torp.lock().unwrap().position_dy;
        let torp_vifo_theta = torp.lock().unwrap().vifo_theta;
        gl.uniform2f(Some(&torp_pos_deltas_loc), new_pos_dx, new_pos_dy);
        gl.uniform1f(Some(&torp_vifo_theta_loc), torp_vifo_theta.0);
        gl.draw_arrays(GL::TRIANGLES, 0, 6);
    }
}

fn draw_player_one
(
    gl: Arc<GL>,
    game_state: Arc<Mutex<GameState>>,
    player_vertex_buffer: Arc<WebGlBuffer>,
    player_js_vertices: Arc<js_sys::Float32Array>,
    shader_program: Arc<web_sys::WebGlProgram>,
    player_vertices_position: Arc<u32>,
    time_location: Arc<WebGlUniformLocation>,
    player_pos_deltas_loc: Arc<WebGlUniformLocation>,
    player_vifo_theta_loc: Arc<WebGlUniformLocation>,
)
{
    gl.use_program(Some(&shader_program));
    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&player_vertex_buffer));
    gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &player_js_vertices, GL::STATIC_DRAW);
    gl.vertex_attrib_pointer_with_i32(*player_vertices_position, 2, GL::FLOAT, false, 0, 0);
    gl.enable_vertex_attrib_array(*player_vertices_position);
    gl.uniform1f(Some(&time_location), 0.4 as f32);
    let new_pos_dx = game_state.lock().unwrap().player_one.lock().unwrap().position_dx;
    let new_pos_dy = game_state.lock().unwrap().player_one.lock().unwrap().position_dy;
    gl.uniform2f(Some(&player_pos_deltas_loc), new_pos_dx, new_pos_dy);    
    let new_vifo_theta = game_state.lock().unwrap().player_one.lock().unwrap().vifo_theta;
    gl.uniform1f(Some(&player_vifo_theta_loc), new_vifo_theta.0);
    gl.draw_arrays(GL::TRIANGLES, 0, 6);
    gl.bind_buffer(GL::ARRAY_BUFFER, None);

}

fn draw_player_two
(
    gl: Arc<GL>,
    game_state: Arc<Mutex<GameState>>,
    player_vertex_buffer: Arc<WebGlBuffer>,
    player_js_vertices: Arc<js_sys::Float32Array>,
    shader_program: Arc<web_sys::WebGlProgram>,
    player_vertices_position: Arc<u32>,
    time_location: Arc<WebGlUniformLocation>,
    player_pos_deltas_loc: Arc<WebGlUniformLocation>,
    player_vifo_theta_loc: Arc<WebGlUniformLocation>,
)
{
    gl.use_program(Some(&shader_program));
    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&player_vertex_buffer));
    gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &player_js_vertices, GL::STATIC_DRAW);
    gl.vertex_attrib_pointer_with_i32(*player_vertices_position, 2, GL::FLOAT, false, 0, 0);
    gl.enable_vertex_attrib_array(*player_vertices_position);
    gl.uniform1f(Some(&time_location), 0.4 as f32);
    let new_pos_dx = game_state.lock().unwrap().player_two.lock().unwrap().position_dx;
    let new_pos_dy = game_state.lock().unwrap().player_two.lock().unwrap().position_dy;
    gl.uniform2f(Some(&player_pos_deltas_loc), new_pos_dx, new_pos_dy);
    let new_vifo_theta = game_state.lock().unwrap().player_two.lock().unwrap().vifo_theta;
    gl.uniform1f(Some(&player_vifo_theta_loc), new_vifo_theta.0);
    gl.draw_arrays(GL::TRIANGLES, 0, 6);
    gl.bind_buffer(GL::ARRAY_BUFFER, None);
}

fn draw_players
// <'a>
(
    gl: Arc<GL>,
    game_state: Arc<Mutex<GameState>>,
    player_vertex_buffer: Arc<WebGlBuffer>,
    player_js_vertices: Arc<js_sys::Float32Array>,
    shader_program: Arc<web_sys::WebGlProgram>,
    player_vertices_position: Arc<u32>,
    time_location: Arc<WebGlUniformLocation>,
    player_pos_deltas_loc: Arc<WebGlUniformLocation>,
    player_vifo_theta_loc: Arc<WebGlUniformLocation>,
)
{
    gl.use_program(Some(&shader_program));
    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&player_vertex_buffer));
    gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &player_js_vertices, GL::STATIC_DRAW);
    gl.vertex_attrib_pointer_with_i32(*player_vertices_position, 2, GL::FLOAT, false, 0, 0);

    gl.enable_vertex_attrib_array(*player_vertices_position);
    gl.uniform1f(Some(&time_location), 0.4 as f32);

    let new_pos_dx = game_state.lock().unwrap().player_one.lock().unwrap().position_dx;
    let new_pos_dy = game_state.lock().unwrap().player_one.lock().unwrap().position_dy;

    gl.uniform2f(Some(&player_pos_deltas_loc), new_pos_dx, new_pos_dy);
    
    let new_vifo_theta = game_state.lock().unwrap().player_one.lock().unwrap().vifo_theta;
    gl.uniform1f(Some(&player_vifo_theta_loc), new_vifo_theta.0);
    gl.draw_arrays(GL::TRIANGLES, 0, 6);
    gl.bind_buffer(GL::ARRAY_BUFFER, None);

    let new_pos_dx = game_state.lock().unwrap().player_two.lock().unwrap().position_dx;
    let new_pos_dy = game_state.lock().unwrap().player_two.lock().unwrap().position_dy;

    gl.uniform2f(Some(&player_pos_deltas_loc), new_pos_dx, new_pos_dy);
    
    let new_vifo_theta = game_state.lock().unwrap().player_two.lock().unwrap().vifo_theta;
    gl.uniform1f(Some(&player_vifo_theta_loc), new_vifo_theta.0);
    gl.draw_arrays(GL::TRIANGLES, 0, 6);
}

// fn update_positions 
fn update_game_state // A slight misnomer, as game state is also mutated by event-handlers.
<'a>
(
    time_delta: u128,
    game_state: Arc<Mutex<GameState>>,
)
-> Result<Arc<Mutex<GameState>>, &'a str>
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

    let mut v : Vec<Arc<Mutex<Torp200>>> = vec![];
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
            log!("Torpedo kills player one.");
            
            *game_state.lock().unwrap().torp_kills_player_1.lock().unwrap() = (true, p1_kx, p1_ky);

        }
        if (space.0 == true) && (space.1 == true) {
            log!("vehicle collision");
        }
        if (space.1 == true) && (space.2.len() > 0) {
            log!("Torpedo kills player two.");
        }

    }

    collisions_map.clear();
    Ok(game_state)
}



fn create_game_state
<'a>
()
-> Result<Arc<Mutex<GameState>>, &'a str>
{
    let mode = 0; // Notionally code for 2-player local.

    let player_one = Arc::new(Mutex::new(Vehicle_100 {
        position_dx: 0.3,
        position_dy: 0.3,
        vifo_theta: Rad(0.3),
        velocity_theta: Rad(0.3),
        velocity_scalar: 0.0,
        velocity_dx: 0.0,
        velocity_dy: 0.0,
    }));

    let player_two = Arc::new(Mutex::new(Vehicle_100 {
        position_dx: -0.4,
        position_dy: -0.4,
        vifo_theta: Rad(-0.3),
        velocity_theta: Rad(-0.3),
        velocity_scalar: 0.0,
        velocity_dx: 0.0,
        velocity_dy: 0.0,
    }));

    let torps_in_flight = Arc::new(Mutex::new(vec![]));
    let collisions_two : Arc<Mutex<
        HashMap<&'static str, Arc<Mutex<CollisionSpace>>>
    >> = 
        Arc::new(Mutex::new(HashMap::new()));

    // let collisions : Arc<Mutex<Vec<_>>> = Arc::new(Mutex::new(vec![]));

    let game_state = GameState {
        player_one: player_one,
        player_two: player_two,
        torps_in_flight: torps_in_flight,
        start_time: Arc::new(Instant::now()),
        elapsed_time: Arc::new(Mutex::new(0)),
        game_over: Arc::new(Mutex::new(false)),
        collisions_map: collisions_two,
        mode: Arc::new(mode),
        result: Arc::new(Mutex::new(0)),
        torp_kills_player_1: Arc::new(Mutex::new((false, 0.0, 0.0))),
    };

    Ok(Arc::new(Mutex::new(game_state)))
}

// type CollisionSpace = (bool, bool, Arc<Mutex<Vec<usize>>>);
type CollisionSpace = (bool, bool, Vec<usize>);

struct GameState 
{
    player_one: Arc<Mutex<Vehicle_100>>,
    player_two:  Arc<Mutex<Vehicle_100>>,
    torps_in_flight: Arc<Mutex<Vec<Arc<Mutex<Torp200>>>>>,
    start_time: Arc<Instant>,
    elapsed_time: Arc<Mutex<u128>>,
    game_over: Arc<Mutex<bool>>,
    collisions_map: Arc<Mutex<HashMap<&'static str, Arc<Mutex<CollisionSpace>>>>>,
    // model an explosion around a vector sum of the collided vehicles, with extra effects. covering torpedo collisions
    // This would be a good place to use Rust traits.
    result: Arc<Mutex<u8>>,
    mode: Arc<u8>, // 1 player vs computer, 2 player local, 2 player network
    torp_kills_player_1: Arc<Mutex<(bool, f32, f32)>>,
}

#[derive(Copy, Clone)]
struct Torp200 {
    position_dx: f32, // raw displacement in x, y
    position_dy: f32,
    // vehicle_inertial_frame_orientation_theta: f32,
    vifo_theta: Rad<f32>,
    // polar description
    velocity_theta: Rad<f32>,
    velocity_scalar: f32,
    // redundant alternate description of velocity, cartesian
    velocity_dx: f32,
    velocity_dy: f32, 
    time_fired: Instant,
}


#[derive(Copy, Clone)]
struct Vehicle_100 {
    position_dx: f32, // raw displacement in x, y
    position_dy: f32,
    // vehicle_inertial_frame_orientation_theta: f32,
    vifo_theta: Rad<f32>,
    // polar description
    velocity_theta: Rad<f32>,
    velocity_scalar: f32,
    // redundant alternate description of velocity, cartesian
    velocity_dx: f32,
    velocity_dy: f32, 
}

struct ParticlesStuff {
    shader_program: Arc<web_sys::WebGlProgram>,
    vertex_array_a: Arc<web_sys::WebGlVertexArrayObject>,
    vertex_array_b: Arc<web_sys::WebGlVertexArrayObject>,
    current_vertex_array: Arc<Mutex<web_sys::WebGlVertexArrayObject>>,
    current_transform_feedback: Arc<Mutex<web_sys::WebGlTransformFeedback>>,
    transform_feedback_a: Arc<web_sys::WebGlTransformFeedback>,
    transform_feedback_b: Arc<web_sys::WebGlTransformFeedback>,
    position_buffer_a: Arc<web_sys::WebGlBuffer>,
    velocity_buffer_a: Arc<web_sys::WebGlBuffer>,
    position_buffer_b: Arc<web_sys::WebGlBuffer>,
    velocity_buffer_b: Arc<web_sys::WebGlBuffer>,
    // offset_uniforms_loc: Arc<u32>,
}


fn prepare_particles(
    gl: Arc<GL>,
)
-> Result<ParticlesStuff, String>
{
    // let (particles_shader_program, offset_uniforms_loc) = setup_particle_shaders(gl.clone()).unwrap();

    let particles_shader_program = setup_particle_shaders(gl.clone()).unwrap();
    let mass_uniforms_location = Arc::new(Mutex::new(gl.get_uniform_block_index(&particles_shader_program, "Mass")));
    gl.uniform_block_binding(&particles_shader_program, *mass_uniforms_location.lock().unwrap(), 0);

    let position_data : Arc<Mutex<[f32; (NUM_PARTICLES * 3) as usize]>> = Arc::new(Mutex::new([0.0; (NUM_PARTICLES * 3) as usize]));
    let velocity_data : Arc<Mutex<[f32; (NUM_PARTICLES * 3) as usize]>> = Arc::new(Mutex::new([0.0; (NUM_PARTICLES *3) as usize]));
    let color_data : Arc<Mutex<[f32; (NUM_PARTICLES * 3) as usize]>> = Arc::new(Mutex::new([0.0; (NUM_PARTICLES * 3) as usize]));

    for i in 0..NUM_PARTICLES {
        let vec3i : usize = (i as usize) * 3;

        position_data.lock().unwrap()[vec3i] = (js_sys::Math::random() as f32) * LOCALIZED_SCALE - CORRECTION;
        position_data.lock().unwrap()[vec3i + 1] = (js_sys::Math::random() as f32) * LOCALIZED_SCALE - CORRECTION;
        position_data.lock().unwrap()[vec3i + 2] = (js_sys::Math::random() as f32) * LOCALIZED_SCALE - CORRECTION;

        color_data.lock().unwrap()[vec3i] = js_sys::Math::random() as f32;
        color_data.lock().unwrap()[vec3i + 1] = js_sys::Math::random() as f32;
        color_data.lock().unwrap()[vec3i + 2] = js_sys::Math::random() as f32;
    }

    let vertex_array_a = Arc::new(gl.create_vertex_array().unwrap());
    gl.bind_vertex_array(Some(&vertex_array_a));

    let position_buffer_a = Arc::new(gl.create_buffer().unwrap());
    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&position_buffer_a));
    let position_data_js = js_sys::Float32Array::from(position_data.lock().unwrap().as_slice());
    gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &position_data_js, GL::STREAM_COPY);
    gl.vertex_attrib_pointer_with_i32(0, 3, GL::FLOAT, false, 0, 0);
    gl.enable_vertex_attrib_array(0);

    let velocity_buffer_a = Arc::new(gl.create_buffer().unwrap());
    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&velocity_buffer_a));
    let velocity_data_js = js_sys::Float32Array::from(velocity_data.lock().unwrap().as_slice());
    gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &velocity_data_js, GL::STREAM_COPY);
    gl.vertex_attrib_pointer_with_i32(1, 3, GL::FLOAT, false, 0, 0);
    gl.enable_vertex_attrib_array(1);

    let color_buffer = Arc::new(gl.create_buffer().unwrap());
    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&color_buffer));
    let color_data_js = js_sys::Float32Array::from(color_data.lock().unwrap().as_slice());
    gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &color_data_js, GL::STATIC_DRAW);
    gl.vertex_attrib_pointer_with_i32(2, 3, GL::FLOAT, false, 0, 0);
    gl.enable_vertex_attrib_array(2);

    gl.bind_buffer(GL::ARRAY_BUFFER, None);

    // Transform feedback handles output 
    // https://github.com/tsherif/webgl2examples/blob/master/particles.html

    let transform_feedback_a = Arc::new(gl.create_transform_feedback().unwrap());
    gl.bind_transform_feedback(GL::TRANSFORM_FEEDBACK, Some(&transform_feedback_a));
    gl.bind_buffer_base(GL::TRANSFORM_FEEDBACK_BUFFER, 0, Some(&position_buffer_a));
    gl.bind_buffer_base(GL::TRANSFORM_FEEDBACK_BUFFER, 1, Some(&velocity_buffer_a));

    let vertex_array_b = Arc::new(gl.create_vertex_array().unwrap());
    gl.bind_vertex_array(Some(&vertex_array_b));

    let position_buffer_b = Arc::new(gl.create_buffer().unwrap());
    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&position_buffer_b));
    gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &position_data_js, GL::STREAM_COPY);
    gl.vertex_attrib_pointer_with_i32(0, 3, GL::FLOAT, false, 0, 0);
    gl.enable_vertex_attrib_array(0);

    let velocity_buffer_b = Arc::new((gl.create_buffer().unwrap()));
    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&velocity_buffer_b));
    gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &velocity_data_js, GL::STREAM_COPY);
    gl.vertex_attrib_pointer_with_i32(1, 3, GL::FLOAT, false, 0, 0);
    gl.enable_vertex_attrib_array(1);

    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&color_buffer));
    gl.vertex_attrib_pointer_with_i32(2, 3, GL::FLOAT, false, 0, 0);
    gl.enable_vertex_attrib_array(2);

    gl.bind_buffer(GL::ARRAY_BUFFER, None);

    let transform_feedback_b = Arc::new(gl.create_transform_feedback().unwrap());
    gl.bind_transform_feedback(GL::TRANSFORM_FEEDBACK, Some(&transform_feedback_b));
    gl.bind_buffer_base(GL::TRANSFORM_FEEDBACK_BUFFER, 0, Some(&position_buffer_b));
    gl.bind_buffer_base(GL::TRANSFORM_FEEDBACK_BUFFER, 1, Some(&velocity_buffer_b));

    gl.bind_vertex_array(None);
    gl.bind_transform_feedback(GL::TRANSFORM_FEEDBACK, None);

    let mut current_vertex_array : Arc<Mutex<_>> = Arc::new(Mutex::new((*vertex_array_a).clone()));
    let mut current_transform_feedback : Arc<Mutex<_>> = Arc::new(Mutex::new((*transform_feedback_b).clone()));

    let mut mass_uniform_data : [f32; 20] = [0.0; 20];

    // let localized_scale = 0.5;
    // let correction = localized_scale / 2.0;

    mass_uniform_data[0] = (js_sys::Math::random() / ((NUM_PARTICLES as f64) * 1.5)) as f32;
    mass_uniform_data[1] = (js_sys::Math::random() / ((NUM_PARTICLES as f64) * 1.5)) as f32;
    mass_uniform_data[2] = (js_sys::Math::random() / (NUM_PARTICLES as f64) * 1.5) as f32;

    mass_uniform_data[4] = ((((js_sys::Math::random() as f32) * LOCALIZED_SCALE) as f32) - CORRECTION) as f32;
    mass_uniform_data[5] = ((((js_sys::Math::random() as f32) * LOCALIZED_SCALE) as f32) - CORRECTION) as f32;
    mass_uniform_data[6] = ((((js_sys::Math::random() as f32) * LOCALIZED_SCALE) as f32) - CORRECTION) as f32;

    mass_uniform_data[8] = ((((js_sys::Math::random() as f32) * LOCALIZED_SCALE) as f32) - CORRECTION) as f32;
    mass_uniform_data[9] = ((((js_sys::Math::random() as f32) * LOCALIZED_SCALE) as f32) - CORRECTION) as f32;
    mass_uniform_data[10] = ((((js_sys::Math::random() as f32) * LOCALIZED_SCALE) as f32) - CORRECTION) as f32;

    mass_uniform_data[16] = ((((js_sys::Math::random() as f32) * LOCALIZED_SCALE) as f32) - CORRECTION) as f32;
    mass_uniform_data[17] = ((((js_sys::Math::random() as f32) * LOCALIZED_SCALE) as f32) - CORRECTION) as f32;
    mass_uniform_data[18] = ((((js_sys::Math::random() as f32) * LOCALIZED_SCALE) as f32) - CORRECTION) as f32;

    let mass_uniform_buffer = gl.create_buffer();
    gl.bind_buffer_base(GL::UNIFORM_BUFFER, 0, mass_uniform_buffer.as_ref());
    let mass_uniform_data_js = js_sys::Float32Array::from(mass_uniform_data.as_slice());
    gl.buffer_data_with_array_buffer_view(GL::UNIFORM_BUFFER, &mass_uniform_data_js, GL::STATIC_DRAW);


    Ok(ParticlesStuff {
        shader_program: particles_shader_program.clone(),
        vertex_array_a: vertex_array_a.clone(),
        vertex_array_b: vertex_array_b,
        current_vertex_array: current_vertex_array,
        current_transform_feedback: current_transform_feedback,
        transform_feedback_a: transform_feedback_a,
        transform_feedback_b: transform_feedback_b,
        position_buffer_a: position_buffer_a,
        velocity_buffer_a: velocity_buffer_a,
        position_buffer_b: position_buffer_b,
        velocity_buffer_b: velocity_buffer_b,
        // offset_uniforms_loc: offset_uniforms_loc,
    })

}