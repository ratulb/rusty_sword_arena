extern crate glium;
use glium::Surface;
use glium::glutin::{self, ElementState};
use std::f64::consts::PI;

use super::*;

#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
}
implement_vertex!(Vertex, position, color);

pub fn angle_between(pos : Position, target_pos : Position) -> Angle {
    (pos.x - target_pos.x).atan2(pos.y - target_pos.y)
}

fn create_circle_vertices(radius : f32, num_vertices : usize, color : Color) -> Vec<Vertex> {
    let mut v = Vec::<Vertex>::with_capacity(num_vertices+2);
    // The center of the circle/fan
    v.push(Vertex {
        position: [0.0, 0.0],
        color: [color.r, color.g, color.b],
    });
    for x in 0..=num_vertices {
        let inner : f64 = 2.0 * PI / num_vertices as f64 * x as f64;
        // Color the forward-facing vertex of the circle differently so we can have a small "sword"
        // indicator of our forward-facing direction
        let color = if x == (num_vertices * 3)/4 { Color { r: 0.1, g: 0.1, b: 0.1 } } else { color };
        v.push(Vertex {
            position: [inner.cos() as f32 * radius, inner.sin() as f32 * radius],
            color: [color.r, color.g, color.b],
        });
    }
    v
}

//pub struct GlThingy<I, U>
//    where I : glium::index::Index,
//          U : glium::uniforms::Uniforms {
//    vertex_buffer : glium::VertexBuffer,
//    index : I,
//    program : glium::Program,
//    uniforms : U,
//}

pub struct Shape {
    pub pos : Position,
    pub direction : Angle,
    vertex_buffer : glium::vertex::VertexBuffer<Vertex>,
}

impl Shape {
    pub fn new_circle(display : &Display, radius : f32, pos : Position, direction : Angle, color : Color) -> Self {
        let vertex_buffer = glium::VertexBuffer::new(&display.display, &create_circle_vertices(radius, 32, color)).unwrap();
        Self {
            pos,
            direction,
            vertex_buffer,
        }
    }
}


pub struct Display {
    events_loop : glutin::EventsLoop,
    pub display : glium::Display,
    program : glium::Program,
    horiz_axis : f32,
    vert_axis : f32,
    mouse_pos : Position,
    screen_to_opengl : Box<FnMut((f64, f64)) -> Position>,
    game_settings : GameSettings,
}


impl Display {
    pub fn new(width : u32, height : u32, game_settings : &GameSettings) -> Self {
        let events_loop = glutin::EventsLoop::new();
        let window = glutin::WindowBuilder::new()
            .with_dimensions(width, height)
            .with_title("Rusty Sword Arena!");
        let context = glutin::ContextBuilder::new();
        let display = glium::Display::new(window, context, &events_loop).unwrap();

        // Create a closure that captures the hidpi_factor to do local screen coordinate conversion
        // for us.
        let hidpi_factor = display.gl_window().window().hidpi_factor();
        let screen_to_opengl = Box::new(move |screen_coord : (f64, f64)| -> Position {
            let x = (screen_coord.0 as f32 / (0.5 * hidpi_factor * width as f32)) - 1.0;
            let y = 1.0 - (screen_coord.1 as f32 / (0.5 * hidpi_factor * height as f32));
            Position { x, y }
        });

        let vertex_shader_src = r#"
        #version 140

        in vec2 position;
        in vec3 color;
        out vec3 v_color;

        uniform mat4 matrix;

        void main() {
            v_color = color;
            gl_Position = matrix * vec4(position, 0.0, 1.0);
        }
        "#;

        let fragment_shader_src = r#"
            #version 140

            in vec3 v_color;
            out vec4 color;

            void main() {
                color = vec4(v_color, 1.0);
            }
        "#;

        let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();
        Self {
            events_loop,
            display,
            program,
            horiz_axis : 0.0,
            vert_axis : 0.0,
            mouse_pos : Position { x : 0.0, y : 0.0 },
            screen_to_opengl,
            game_settings : game_settings.clone(),
        }
    }

    pub fn draw(&self, shapes : &Vec<Shape>) {
        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleFan);


        let mut target = self.display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);

        for shape in shapes {
            let uniforms = uniform! {
                matrix: [
                    [shape.direction.cos() as f32, -shape.direction.sin() as f32, 0.0, 0.0],
                    [shape.direction.sin() as f32, shape.direction.cos() as f32, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [shape.pos.x, shape.pos.y, 0.0, 1.0f32],
                ]
            };
            target.draw(&shape.vertex_buffer, &indices, &self.program, &uniforms,
                        &Default::default()).unwrap();
        }
        target.finish().unwrap();
    }

    /// Get events that the graphics system may have seen (window, keyboard, mouse)
    pub fn events(&mut self) -> Vec<Event> {
        let screen_to_opengl = &mut (self.screen_to_opengl);
        let mut events = Vec::<Event>::new();
        self.events_loop.poll_events(|ev| {
            if let glium::glutin::Event::WindowEvent {event, ..} = ev {
                match event {
                    // Time to close the app?
                    glutin::WindowEvent::Closed => events.push(Event::WindowClosed),
                    // Mouse moved
                    glutin::WindowEvent::CursorMoved { device_id : _, position, modifiers : _ } => {
                        let mouse_pos = screen_to_opengl(position);
                        events.push(Event::MouseMoved { position : mouse_pos });
                    },
                    // Keyboard button
                    glutin::WindowEvent::KeyboardInput { device_id : _, input } => {
                        let amount : f32;
                        let key_state = match input.state {
                            ElementState::Pressed => { KeyState::Pressed },
                            ElementState::Released => { KeyState::Released },
                        };
                        use glium::glutin::VirtualKeyCode::*;
                        if let Some(vkey) = input.virtual_keycode {
                            match vkey {
                                W | Up | Comma => { events.push(Event::KeyboardInput { key_state, key_value : KeyValue::Up }) },
                                S | Down | O   => { events.push(Event::KeyboardInput { key_state, key_value : KeyValue::Down }) },
                                A | Left       => { events.push(Event::KeyboardInput { key_state, key_value : KeyValue::Left }) },
                                D | Right | E  => { events.push(Event::KeyboardInput { key_state, key_value : KeyValue::Right }) },
                                _ => (),
                            }
                        }
                    },
                    _ => (),
                }
            }
        });
        events
    }

}


