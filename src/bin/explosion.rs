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
use crate::structures::{ExplosionStuff};

const AMORTIZATION: f32 = 0.95;
const LOCALIZED_SCALE : f32 = 0.001;
const CORRECTION : f32 = LOCALIZED_SCALE / 2.0;
const RESOLUTION : f32 = 8.0;
const SCALE : f32 = 0.08;
const HALF : f32 = SCALE / 2.0;
const STEP : f32 = SCALE / RESOLUTION;
const NUM_PARTICLES : u32 = 9680;

pub fn set_uniforms
(
    gl: Arc<GL>,
    offset_uniform_data: [f32; 2],
)
{
    // Careful with GL context here.  Likely neeed to more explicitly bind and/or unbind stuff here.

    let offset_uniform_buffer = gl.create_buffer();
    gl.bind_buffer_base(GL::UNIFORM_BUFFER, 0, offset_uniform_buffer.as_ref());
    let offset_uniform_data_js = js_sys::Float32Array::from(offset_uniform_data.as_slice());
    gl.buffer_data_with_array_buffer_view(GL::UNIFORM_BUFFER, &offset_uniform_data_js, GL::STATIC_DRAW);
}

pub fn draw_explosion
(
    gl: Arc<GL>,
    shader_program: Arc<WebGlProgram>,
    explosion_stuff: Arc<ExplosionStuff>,
    switch: Arc<Mutex<AtomicBool>>,
)
{


    gl.use_program(Some(&shader_program));
    
    let vertex_array_a = &explosion_stuff.vertex_array_a;
    let vertex_array_b = &explosion_stuff.vertex_array_b;
    let transform_feedback_a = &explosion_stuff.transform_feedback_a;
    let transform_feedback_b = &explosion_stuff.transform_feedback_b;
    let position_buffer_a = &explosion_stuff.position_buffer_a;
    let position_buffer_b = &explosion_stuff.position_buffer_b;
    let velocity_buffer_a = &explosion_stuff.velocity_buffer_a;
    let velocity_buffer_b = &explosion_stuff.velocity_buffer_b;
    let color_buffer = &explosion_stuff.color_buffer;
    let current_transform_feedback = &explosion_stuff.current_transform_feedback;
    let current_vertex_array = &explosion_stuff.current_vertex_array;
    let position_data = &explosion_stuff.position_data;
    let velocity_data = &explosion_stuff.velocity_data;
    let color_data = &explosion_stuff.color_data;

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
        *current_vertex_array.lock().unwrap() = (**vertex_array_b).clone();
        *current_transform_feedback.lock().unwrap() = (**transform_feedback_a).clone();
    } else 
    {
        *current_vertex_array.lock().unwrap() = (**vertex_array_a).clone();
        *current_transform_feedback.lock().unwrap() = (**transform_feedback_b).clone();
    }
    *switch.lock().unwrap().get_mut() = !s; 
    gl.bind_vertex_array(None);
}

pub fn setup_shader
(
    gl: Arc<GL>,
)
-> Result<Arc<web_sys::WebGlProgram>, String>
{
    let vert_code = include_str!("../shaders/explosion.vert");
    let vert_shader = gl.create_shader(GL::VERTEX_SHADER).unwrap();
    gl.shader_source(&vert_shader, vert_code);
    gl.compile_shader(&vert_shader);
    let vert_shader_log = gl.get_shader_info_log(&vert_shader);
    log!("vert shader log: ", vert_shader_log);

    let frag_code = include_str!("../shaders/explosion.frag");
    let frag_shader = gl.create_shader(GL::FRAGMENT_SHADER).unwrap();
    gl.shader_source(&frag_shader, frag_code);
    gl.compile_shader(&frag_shader);
    let frag_shader_log = gl.get_shader_info_log(&frag_shader);
    log!("frag shader log:", frag_shader_log);

    let shader_program = gl.create_program().unwrap();
    gl.attach_shader(&shader_program, &vert_shader);
    gl.attach_shader(&shader_program, &frag_shader);

    let data = r#"
    ["vPosition", "vVelocity"]
    "#;
    let v: Value = serde_json::from_str(data).unwrap();
    let js_val = wasm_bindgen::JsValue::from_serde(&v).unwrap();
    gl.transform_feedback_varyings(&shader_program, &js_val, GL::SEPARATE_ATTRIBS);
    gl.link_program(&shader_program);
    
    let offset_loc = Arc::new(Mutex::new(gl.get_uniform_block_index(&shader_program, "Offset")));
    gl.uniform_block_binding(&shader_program, *offset_loc.lock().unwrap(), 0);
    Ok(Arc::new(shader_program))
}


pub fn prepare_explosion
(
    gl: Arc<GL>,
    // probably will need to inject the shader program in order to 
    // set context with use_program(&explosion_shader_program)
)
-> Result<Arc<ExplosionStuff>, String>
{
    let position_data: Arc<Mutex<[f32; (NUM_PARTICLES * 3) as usize]>> = Arc::new(Mutex::new([0.0; (NUM_PARTICLES * 3) as usize]));
    let velocity_data: Arc<Mutex<[f32; (NUM_PARTICLES * 3) as usize]>> = Arc::new(Mutex::new([0.0; (NUM_PARTICLES *3) as usize]));
    let color_data: Arc<Mutex<[f32; (NUM_PARTICLES * 3) as usize]>> = Arc::new(Mutex::new([0.0; (NUM_PARTICLES * 3) as usize]));

    for i in 0..NUM_PARTICLES {
        let vec3i : usize = (i as usize) * 3;

        let random_theta = js_sys::Math::random() * 2.0 * (PI as f64);
        let random_psi = js_sys::Math::random() * 2.0 * (PI as f64);
        let r = js_sys::Math::random() * 0.001;

        let x = Rad::cos(Rad(random_theta)) * r;
        let y = Rad::sin(Rad(random_theta)) * r;
        let z = Rad::sin(Rad(random_psi)) * r;

        position_data.lock().unwrap()[vec3i] = x as f32;
        position_data.lock().unwrap()[vec3i + 1] = y as f32;
        position_data.lock().unwrap()[vec3i + 2] = z as f32;

        color_data.lock().unwrap()[vec3i] = (js_sys::Math::random() as f32);
        color_data.lock().unwrap()[vec3i + 1] = (js_sys::Math::random() as f32);
        color_data.lock().unwrap()[vec3i + 2] = (js_sys::Math::random() as f32);
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

    let mut current_vertex_array: Arc<Mutex<_>> = Arc::new(Mutex::new((*vertex_array_a).clone()));
    let mut current_transform_feedback: Arc<Mutex<_>> = Arc::new(Mutex::new((*transform_feedback_b).clone()));

    Ok(
        Arc::new(
            ExplosionStuff {
                vertex_array_a: vertex_array_a,
                vertex_array_b: vertex_array_b,
                transform_feedback_a: transform_feedback_a,
                transform_feedback_b:  transform_feedback_b,
                position_buffer_a: position_buffer_a,
                position_buffer_b: position_buffer_b,
                velocity_buffer_a: velocity_buffer_a,
                velocity_buffer_b: velocity_buffer_b,
                color_buffer: color_buffer,
                current_transform_feedback: current_transform_feedback,
                current_vertex_array: current_vertex_array,
                position_data: position_data,
                velocity_data: velocity_data,
                color_data: color_data,
            }
        )
    )
}

pub fn refresh_explosion
(
    gl: Arc<GL>,
    shader_program: Arc<web_sys::WebGlProgram>,
    explosion_stuff: ExplosionStuff,
    offset_uniform_data: [f32; 2],
)
{
    let vertex_array_a = explosion_stuff.vertex_array_a;
    let vertex_array_b = explosion_stuff.vertex_array_b;
    let transform_feedback_a = explosion_stuff.transform_feedback_a;
    let transform_feedback_b = explosion_stuff.transform_feedback_b;
    let position_buffer_a = explosion_stuff.position_buffer_a;
    let position_buffer_b = explosion_stuff.position_buffer_b;
    let velocity_buffer_a = explosion_stuff.velocity_buffer_a;
    let velocity_buffer_b = explosion_stuff.velocity_buffer_b;
    let color_buffer = explosion_stuff.color_buffer;
    let current_transform_feedback = explosion_stuff.current_transform_feedback;
    let current_vertex_array = explosion_stuff.current_vertex_array;
    let position_data = explosion_stuff.position_data;
    let velocity_data = explosion_stuff.velocity_data;
    let color_data = explosion_stuff.color_data;

    gl.bind_vertex_array(None);
    gl.bind_transform_feedback(GL::TRANSFORM_FEEDBACK, None);
    gl.bind_buffer(GL::ARRAY_BUFFER, None);
    gl.bind_transform_feedback(GL::TRANSFORM_FEEDBACK, None);
    gl.bind_buffer_base(GL::TRANSFORM_FEEDBACK_BUFFER, 0, None);
    gl.bind_buffer_base(GL::TRANSFORM_FEEDBACK_BUFFER, 1, None);

    gl.bind_buffer_base(GL::UNIFORM_BUFFER, 0, None);
    let offset_uniform_buffer = gl.create_buffer();
    gl.bind_buffer_base(GL::UNIFORM_BUFFER, 0, offset_uniform_buffer.as_ref());
    let offset_uniform_data_js = js_sys::Float32Array::from(offset_uniform_data.as_slice());
    gl.buffer_data_with_array_buffer_view(GL::UNIFORM_BUFFER, &offset_uniform_data_js, GL::STATIC_DRAW);

    for i in 0..NUM_PARTICLES {
        let vec3i : usize = (i as usize) * 3;

        let random_theta = js_sys::Math::random() * 2.0 * (PI as f64);
        let random_psi = js_sys::Math::random() * 2.0 * (PI as f64);
        let r = js_sys::Math::random() * 0.001;

        let x = Rad::cos(Rad(random_theta)) * r;
        let y = Rad::sin(Rad(random_theta)) * r;
        let z = Rad::sin(Rad(random_psi)) * r;

        position_data.lock().unwrap()[vec3i] = x as f32;
        position_data.lock().unwrap()[vec3i + 1] = y as f32;
        position_data.lock().unwrap()[vec3i + 2] = z as f32;

        color_data.lock().unwrap()[vec3i] = (js_sys::Math::random() as f32);
        color_data.lock().unwrap()[vec3i + 1] = (js_sys::Math::random() as f32);
        color_data.lock().unwrap()[vec3i + 2] = (js_sys::Math::random() as f32);
    }

    gl.bind_vertex_array(Some(&vertex_array_a));
    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&position_buffer_a));
    let position_data_js = js_sys::Float32Array::from(position_data.lock().unwrap().as_slice());
    gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &position_data_js, GL::STREAM_COPY);
    gl.vertex_attrib_pointer_with_i32(0, 3, GL::FLOAT, false, 0, 0);
    gl.enable_vertex_attrib_array(0);


    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&velocity_buffer_a));
    let velocity_data_js = js_sys::Float32Array::from(velocity_data.lock().unwrap().as_slice());
    gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &velocity_data_js, GL::STREAM_COPY);
    gl.vertex_attrib_pointer_with_i32(1, 3, GL::FLOAT, false, 0, 0);
    gl.enable_vertex_attrib_array(1);


    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&color_buffer));
    let color_data_js = js_sys::Float32Array::from(color_data.lock().unwrap().as_slice());
    gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &color_data_js, GL::STATIC_DRAW);
    gl.vertex_attrib_pointer_with_i32(2, 3, GL::FLOAT, false, 0, 0);
    gl.enable_vertex_attrib_array(2);

    gl.bind_buffer(GL::ARRAY_BUFFER, None);


    gl.bind_transform_feedback(GL::TRANSFORM_FEEDBACK, Some(&transform_feedback_a));
    gl.bind_buffer_base(GL::TRANSFORM_FEEDBACK_BUFFER, 0, Some(&position_buffer_a));
    gl.bind_buffer_base(GL::TRANSFORM_FEEDBACK_BUFFER, 1, Some(&velocity_buffer_a));

    // let vertex_array_b = Arc::new(gl.create_vertex_array().unwrap());
    gl.bind_vertex_array(Some(&vertex_array_b));

    // let position_buffer_b = Arc::new(gl.create_buffer().unwrap());
    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&position_buffer_b));
    gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &position_data_js, GL::STREAM_COPY);
    gl.vertex_attrib_pointer_with_i32(0, 3, GL::FLOAT, false, 0, 0);
    gl.enable_vertex_attrib_array(0);

    // let velocity_buffer_b = Arc::new((gl.create_buffer().unwrap()));
    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&velocity_buffer_b));
    gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &velocity_data_js, GL::STREAM_COPY);
    gl.vertex_attrib_pointer_with_i32(1, 3, GL::FLOAT, false, 0, 0);
    gl.enable_vertex_attrib_array(1);

    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&color_buffer));
    gl.vertex_attrib_pointer_with_i32(2, 3, GL::FLOAT, false, 0, 0);
    gl.enable_vertex_attrib_array(2);

    gl.bind_buffer(GL::ARRAY_BUFFER, None);


    gl.bind_transform_feedback(GL::TRANSFORM_FEEDBACK, Some(&transform_feedback_b));
    gl.bind_buffer_base(GL::TRANSFORM_FEEDBACK_BUFFER, 0, Some(&position_buffer_b));
    gl.bind_buffer_base(GL::TRANSFORM_FEEDBACK_BUFFER, 1, Some(&velocity_buffer_b));

    gl.bind_vertex_array(None);
    gl.bind_transform_feedback(GL::TRANSFORM_FEEDBACK, None);

    *current_vertex_array.lock().unwrap() = (*vertex_array_a).clone();
    *current_transform_feedback.lock().unwrap() = (*transform_feedback_b).clone();
}


// struct ExplosionStuff {
//     vertex_array_a: Arc<web_sys::WebGlVertexArrayObject>,
//     vertex_array_b: Arc<web_sys::WebGlVertexArrayObject>,
//     transform_feedback_a: Arc<web_sys::WebGlTransformFeedback>,
//     transform_feedback_b: Arc<web_sys::WebGlTransformFeedback>,
//     position_buffer_a: Arc<web_sys::WebGlBuffer>,
//     position_buffer_b: Arc<web_sys::WebGlBuffer>,
//     velocity_buffer_a: Arc<web_sys::WebGlBuffer>,
//     velocity_buffer_b: Arc<web_sys::WebGlBuffer>,
//     color_buffer: Arc<web_sys::WebGlBuffer>,
//     current_transform_feedback: Arc<Mutex<web_sys::WebGlTransformFeedback>>,
//     current_vertex_array: Arc<Mutex<web_sys::WebGlVertexArrayObject>>,
//     position_data: Arc<Mutex<[f32; (NUM_PARTICLES * 3) as usize]>>,
//     velocity_data: Arc<Mutex<[f32; (NUM_PARTICLES * 3) as usize]>>,
//     color_data: Arc<Mutex<[f32; (NUM_PARTICLES * 3) as usize]>>,
// }