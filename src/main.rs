use macroquad::prelude::*;

const CUBE_SIZE: f32 = 2.0;
const LAND_SIZE_X: i32 = 100;
const LAND_SIZE_Z: i32 = 50;
const TEXTURE_SIZE: u16 = 16;
const RENDER_DISTANCE: f32 = 50.0; // Only render blocks within this distance

fn create_grass_texture() -> Texture2D {
    let mut image = Image::gen_image_color(TEXTURE_SIZE, TEXTURE_SIZE, WHITE);

    for y in 0..TEXTURE_SIZE {
        for x in 0..TEXTURE_SIZE {
            let color_idx = rand::gen_range(0, 3);
            let color = match color_idx {
                0 => Color::new(0.0, 0.196, 0.0, 1.0),
                1 => Color::new(0.0, 0.592, 0.0, 1.0),
                _ => Color::new(0.0, 0.424, 0.0, 1.0),
            };
            image.set_pixel(x as u32, y as u32, color);
        }
    }

    Texture2D::from_image(&image)
}

fn create_dirt_texture() -> Texture2D {
    let mut image = Image::gen_image_color(TEXTURE_SIZE, TEXTURE_SIZE, WHITE);

    for y in 0..TEXTURE_SIZE {
        for x in 0..TEXTURE_SIZE {
            let color_idx = rand::gen_range(0, 3);
            let color = match color_idx {
                0 => Color::new(0.494, 0.176, 0.0, 1.0),
                1 => Color::new(0.494, 0.176, 0.0, 1.0),
                _ => Color::new(0.424, 0.165, 0.0, 1.0),
            };
            image.set_pixel(x as u32, y as u32, color);
        }
    }

    Texture2D::from_image(&image)
}

fn draw_textured_cube(position: Vec3, grass_tex: &Texture2D, dirt_tex: &Texture2D) {
    let half = CUBE_SIZE / 2.0;
    let x = position.x;
    let y = position.y;
    let z = position.z;

    let v = [
        vec3(x - half, y - half, z - half), // 0
        vec3(x + half, y - half, z - half), // 1
        vec3(x + half, y + half, z - half), // 2
        vec3(x - half, y + half, z - half), // 3
        vec3(x - half, y - half, z + half), // 4
        vec3(x + half, y - half, z + half), // 5
        vec3(x + half, y + half, z + half), // 6
        vec3(x - half, y + half, z + half), // 7
    ];

    // Top face (grass)
    draw_cube_face(&[v[3], v[2], v[6], v[7]], grass_tex);

    // Bottom face (dirt)
    draw_cube_face(&[v[0], v[1], v[5], v[4]], dirt_tex);

    // Front face (dirt)
    draw_cube_face(&[v[4], v[5], v[6], v[7]], dirt_tex);

    // Back face (dirt)
    draw_cube_face(&[v[1], v[0], v[3], v[2]], dirt_tex);

    // Left face (dirt)
    draw_cube_face(&[v[0], v[4], v[7], v[3]], dirt_tex);

    // Right face (dirt)
    draw_cube_face(&[v[5], v[1], v[2], v[6]], dirt_tex);
}

fn draw_cube_face(vertices: &[Vec3; 4], texture: &Texture2D) {
    unsafe {
        get_internal_gl().quad_gl.texture(Some(texture));
        get_internal_gl().quad_gl.draw_mode(DrawMode::Triangles);
        get_internal_gl().quad_gl.geometry(
            &[
                Vertex::new(vertices[0].x, vertices[0].y, vertices[0].z, 0.0, 0.0, WHITE),
                Vertex::new(vertices[1].x, vertices[1].y, vertices[1].z, 1.0, 0.0, WHITE),
                Vertex::new(vertices[2].x, vertices[2].y, vertices[2].z, 1.0, 1.0, WHITE),
                Vertex::new(vertices[0].x, vertices[0].y, vertices[0].z, 0.0, 0.0, WHITE),
                Vertex::new(vertices[2].x, vertices[2].y, vertices[2].z, 1.0, 1.0, WHITE),
                Vertex::new(vertices[3].x, vertices[3].y, vertices[3].z, 0.0, 1.0, WHITE),
            ],
            &[0, 1, 2, 3, 4, 5],
        );
        get_internal_gl().quad_gl.texture(None);
    }
}

fn draw_camera_model(position: Vec3, forward: Vec3, up: Vec3) {
    let size = 0.5;
    let lens_length = 0.8;

    // Camera body (cube)
    draw_cube(position, vec3(size, size, size), None, RED);
    draw_cube_wires(position, vec3(size, size, size), BLACK);

    // Camera lens (cylinder approximation pointing forward)
    let lens_end = position + forward * lens_length;
    draw_line_3d(position, lens_end, DARKGRAY);
    draw_sphere(lens_end, 0.15, None, BLUE);

    // Up indicator (small line showing which way is up)
    let up_indicator = position + up * 0.5;
    draw_line_3d(position, up_indicator, GREEN);
}

#[macroquad::main("Minecraft World 100x50")]
async fn main() {
    let grass_texture = create_grass_texture();
    let dirt_texture = create_dirt_texture();

    grass_texture.set_filter(FilterMode::Nearest);
    dirt_texture.set_filter(FilterMode::Nearest);

    let mut camera_yaw = 0.8f32;
    let mut camera_pitch = 0.6f32;
    let mut camera_distance = 50.0f32;

    let mut last_mouse_pos: Option<Vec2> = None;
    let mut show_camera = false;

    loop {
        clear_background(Color::new(0.53, 0.81, 0.92, 1.0));

        // Handle mouse camera control
        if is_mouse_button_down(MouseButton::Left) {
            let mouse_pos = Vec2::from(mouse_position());

            if let Some(last_pos) = last_mouse_pos {
                let delta = mouse_pos - last_pos;
                camera_yaw += delta.x * 0.005;
                camera_pitch -= delta.y * 0.005;
                camera_pitch = camera_pitch.clamp(-1.5, 1.5);
            }

            last_mouse_pos = Some(mouse_pos);
        } else {
            last_mouse_pos = None;
        }

        // Handle zoom with mouse wheel
        let wheel = mouse_wheel().1;
        camera_distance -= wheel * 2.0;
        camera_distance = camera_distance.clamp(10.0, 150.0);

        // Toggle camera visibility with C key
        if is_key_pressed(KeyCode::C) {
            show_camera = !show_camera;
        }

        // Calculate camera position
        let cam_x = camera_yaw.sin() * camera_pitch.cos() * camera_distance;
        let cam_y = camera_pitch.sin() * camera_distance;
        let cam_z = camera_yaw.cos() * camera_pitch.cos() * camera_distance;
        let camera_pos = vec3(cam_x, cam_y, cam_z);

        set_camera(&Camera3D {
            position: camera_pos,
            up: vec3(0.0, 1.0, 0.0),
            target: vec3(0.0, 0.0, 0.0),
            ..Default::default()
        });

        // Calculate camera forward vector
        let forward = (vec3(0.0, 0.0, 0.0) - camera_pos).normalize();

        // Draw blocks within render distance
        let offset_x = (LAND_SIZE_X as f32 * CUBE_SIZE) / 2.0;
        let offset_z = (LAND_SIZE_Z as f32 * CUBE_SIZE) / 2.0;

        let mut blocks_rendered = 0;

        for x in 0..LAND_SIZE_X {
            for z in 0..LAND_SIZE_Z {
                let pos_x = x as f32 * CUBE_SIZE - offset_x;
                let pos_z = z as f32 * CUBE_SIZE - offset_z;
                let pos_y = 0.0;
                let block_pos = vec3(pos_x, pos_y, pos_z);

                // Only render blocks within render distance
                let distance = (block_pos - camera_pos).length();
                if distance <= RENDER_DISTANCE {
                    draw_textured_cube(block_pos, &grass_texture, &dirt_texture);
                    blocks_rendered += 1;
                }
            }
        }

        // Draw camera model
        if show_camera {
            draw_camera_model(camera_pos, forward, vec3(0.0, 1.0, 0.0));
        }

        // Reset to 2D for UI
        set_default_camera();

        draw_text(
            "Minecraft World - 100x50 Blocks",
            20.0,
            30.0,
            30.0,
            DARKGRAY,
        );
        draw_text("Click and drag to rotate", 20.0, 60.0, 20.0, DARKGRAY);
        draw_text("Mouse wheel to zoom", 20.0, 85.0, 20.0, DARKGRAY);
        draw_text(
            "Press C to toggle camera model",
            20.0,
            110.0,
            20.0,
            DARKGRAY,
        );
        draw_text(
            &format!(
                "Rendered: {} / {} | FPS: {}",
                blocks_rendered,
                LAND_SIZE_X * LAND_SIZE_Z,
                get_fps()
            ),
            20.0,
            135.0,
            20.0,
            DARKGRAY,
        );
        draw_text(
            &format!("Render Distance: {:.0}", RENDER_DISTANCE),
            20.0,
            160.0,
            20.0,
            DARKGRAY,
        );

        next_frame().await
    }
}
