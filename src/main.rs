use raylib::prelude::*;
use rand::Rng;

const DISPLAY_WIDTH: i32 = 1000;
const DISPLAY_HEIGHT: i32 = 800;

const STARTING_BODY_COUNT: i32 = 1000;

const G: f32 = 1000.0;

struct Body {
    position: Vector2,
    velocity: Vector2,
    colour: Color,
    radius: f32,
    mass: f32,
    merged: bool,
    new: bool
}

fn colliding(body_a: &Body, body_b: &Body) -> bool {
    return (body_b.position - body_a.position).length() < body_a.radius + body_b.radius;
}

fn random_vector_circle(rng: &mut rand::rngs::ThreadRng, radius: f32) -> Vector2 {
    let r = (rng.gen_range(0.0..1.0) as f32).powf(1.0) * radius;
    let theta = rng.gen_range(0.0..std::f32::consts::TAU);
    return Vector2::new(theta.cos() * r, theta.sin() * r);
}

fn main() {
    let mut bodies: Vec<Body> = Vec::with_capacity(STARTING_BODY_COUNT as usize);
    let mut rng = rand::thread_rng();
    for _ in 0..STARTING_BODY_COUNT {
        let position_relative_to_centre = random_vector_circle(&mut rng, DISPLAY_WIDTH.min(DISPLAY_HEIGHT) as f32 / 2.0);
        let position = position_relative_to_centre + Vector2::new(DISPLAY_WIDTH as f32 / 2.0, DISPLAY_HEIGHT as f32 / 2.0);
        let new_body = Body {
            position: position,
            velocity: Vector2::new(-position_relative_to_centre.y, position_relative_to_centre.x).scale_by(0.1),
            // velocity: Vector2::new(0.0, 0.0),
            colour: Color::new(rng.gen_range(40..255), rng.gen_range(40..255), rng.gen_range(40..255), 255),
            radius: 1.0,
            mass: 1.0,
            merged: false,
            new: false
        };
        bodies.push(new_body);
    }

    let (mut handle, thread) = raylib::init()
        .size(DISPLAY_WIDTH, DISPLAY_HEIGHT)
        .title("Gravity")
        .vsync()
        .build();

    while !handle.window_should_close() {
        // Update
        let mut to_merge: Vec<(usize, usize)> = Vec::new();
        if bodies.len() > 1 {
            for i in 0..bodies.len()-1 {
                for j in i+1..bodies.len() {
                    let (bodies_low, bodies_high) = bodies.split_at_mut(j);
                    let body_a = &mut bodies_low[i];
                    let body_b = &mut bodies_high[0];
                    let a_to_b = body_b.position - body_a.position;
                    let a_to_b_distance_sqr = a_to_b.length_sqr();
                    let a_to_b_direction = a_to_b.normalized();
                    let force = G * body_a.mass * body_b.mass / a_to_b_distance_sqr;
                    body_a.velocity += a_to_b_direction.scale_by(force / body_a.mass * handle.get_frame_time());
                    body_b.velocity -= a_to_b_direction.scale_by(force / body_b.mass * handle.get_frame_time());
                    if colliding(body_a, body_b) {
                        to_merge.push((i, j));
                    }
                }
            }
        }
        for (i, j) in to_merge.iter() {
            let (bodies_low, bodies_high) = bodies.split_at_mut(*j);
            let body_a = &mut bodies_low[*i];
            let body_b = &mut bodies_high[0];
            if !body_a.merged && !body_b.merged {
                // Convert body_a into new merged body with summed area/mass/momentum and lerped (using masses) position and colour
                let a_area = std::f32::consts::TAU / 2.0 * body_a.radius.powf(2.0);
                let b_area = std::f32::consts::TAU / 2.0 * body_b.radius.powf(2.0);
                body_a.radius = (a_area + b_area).sqrt() / std::f32::consts::PI.sqrt(); // Get radius of circle with summed area
                let lerp_i = body_b.mass / (body_b.mass + body_a.mass);
                body_a.position = body_a.position * (1.0 - lerp_i) + body_b.position * lerp_i;
                body_a.colour.r = (((body_a.colour.r as f32 / 255.0) * (1.0 - lerp_i) + (body_b.colour.r as f32 / 255.0) * lerp_i) * 255.0) as u8;
                body_a.colour.g = (((body_a.colour.g as f32 / 255.0) * (1.0 - lerp_i) + (body_b.colour.g as f32 / 255.0) * lerp_i) * 255.0) as u8;
                body_a.colour.b = (((body_a.colour.b as f32 / 255.0) * (1.0 - lerp_i) + (body_b.colour.b as f32 / 255.0) * lerp_i) * 255.0) as u8;
                let total_mass = body_a.mass + body_b.mass;
                let total_momentum = body_a.velocity.scale_by(body_a.mass) + body_b.velocity.scale_by(body_b.mass);
                body_a.mass = total_mass;
                body_a.velocity = total_momentum.scale_by(1.0 / total_mass);

                // For later
                body_a.merged = true; // Don't use to merge again and delete if not replaced with new body
                body_b.merged = true; // Ditto
                body_a.new = true; // Don't delete this one
            }
        }
        let mut i = 0;
        while i < bodies.len() {
            let body = &mut bodies[i];
            if body.merged && !body.new {
                bodies.remove(i);
                continue; // Skip incrementing index
            } else if body.new {
                body.merged = false;
                body.new = false;
            }
            i += 1;
        }
        for body in bodies.iter_mut() {
            body.position += body.velocity * handle.get_frame_time();
        }

        // Draw
        let mut draw_handle = handle.begin_drawing(&thread);
        draw_handle.clear_background(Color::BLACK);
        for body in bodies.iter() {
            draw_handle.draw_circle_v(body.position, body.radius, body.colour);
        }
    }
}
