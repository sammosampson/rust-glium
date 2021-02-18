use std::io::Cursor;

pub fn run() {
    #[allow(unused_imports)]
    use glium::{glutin, Surface};

    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    #[derive(Copy, Clone)]
    pub struct RenderPrimitive {
        position: [u16; 2],
        dimensions: [u16; 2],
        inner_colour: [f32; 4],
        outer_colour: [f32; 4],
        identification: [u16; 2],
        extra_data_1: [f32; 4],
        extra_data_2: [f32; 4]
    }

    impl RenderPrimitive {
        pub fn with_position_size_colours_identification_and_data(
            position: [u16; 2],
            dimensions: [u16; 2],
            inner_colour: [f32; 4],
            outer_colour: [f32; 4],
            identification: [u16; 2],
            extra_data_1: [f32; 4],
            extra_data_2: [f32; 4]) -> Self {
            Self {
                position,
                dimensions,
                inner_colour,
                outer_colour,
                identification,
                extra_data_1,
                extra_data_2
            }
        }
        
        pub fn circle(
            position: [u16; 2],
            radius: u16,
            inner_colour: [f32; 4],
            outer_colour: [f32; 4],
            stroke_width: f32) -> Self {
            RenderPrimitive::with_position_size_colours_identification_and_data(
                position,
                [radius, radius],
                inner_colour,
                outer_colour,
                [0, 0],
                [stroke_width, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0]
            ) 
        }

        pub fn rectangle(
            position: [u16; 2],
            dimensions: [u16; 2],
            inner_colour: [f32; 4],
            outer_colour: [f32; 4],
            stroke_width: f32,
            corner_radii: [f32; 4]) -> Self {
            RenderPrimitive::with_position_size_colours_identification_and_data(
                position,
                dimensions,
                inner_colour,
                outer_colour,
                [1, 0],
                [stroke_width, 0.0, 0.0, 0.0],
                corner_radii
            ) 
        }

        pub fn text(
            position: [u16; 2],
            dimensions: [u16; 2],
            colour: [f32; 4],
            glyph_index: u16) -> Self {
            RenderPrimitive::with_position_size_colours_identification_and_data(
                position,
                dimensions,
                colour,
                colour,
                [2, glyph_index],
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0]
            ) 
        }
 
        pub fn expand_dimensions(&mut self, increase_in_pixels: i16) {
            let unsigned_movement_in_pixels = increase_in_pixels.wrapping_abs() as u16;
            if increase_in_pixels > 0 {
                self.dimensions[0] = &self.dimensions[0] + unsigned_movement_in_pixels;
                self.dimensions[1] = &self.dimensions[1] + unsigned_movement_in_pixels;
            } else {
                
                self.dimensions[0] = &self.dimensions[0] - unsigned_movement_in_pixels;
                self.dimensions[1] = &self.dimensions[1] - unsigned_movement_in_pixels;
            }
        }
    }

    implement_vertex!(
        RenderPrimitive,
        position, 
        dimensions, 
        inner_colour,
        outer_colour,
        identification,
        extra_data_1,
        extra_data_2
    );

    let white = [1.0, 1.0, 1.0, 1.0];        
    let black = [0.0, 0.0, 0.0, 1.0];    
    
    let mut vertices = glium::vertex::VertexBuffer::<RenderPrimitive>::empty_dynamic(&display, 16384).unwrap();
    let buffer_start = std::time::Instant::now();
        
    let mut glyph_index:u16 = 35;
    for y in 0..100 {
        for x in 0..100 {
            let index = (y * 100) + x;
            let position = [0 + x as u16 * 25, 0 + y as u16 * 25];
            if index % 2 == 0 {
                &mut vertices.map_write().set(index, RenderPrimitive::circle(position, 20, white, white, 0.0));
            }
            else if index % 3 == 0 {
                &mut vertices.map_write().set(index, RenderPrimitive::rectangle(position, [20, 20], white, white, 0.0, [0.1, 0.3, 0.4, 0.2]));
            }
            else {
                &mut vertices.map_write().set(index, RenderPrimitive::text(position, [50, 50], black, glyph_index));
                if glyph_index > 500 {
                    glyph_index = 35
                }
                else {
                    glyph_index += 1;
                }
            }
        }
    }

    let buffer_time = std::time::Instant::now() - buffer_start;
    println!("buffer time: {:?}", buffer_time);

    let indices = glium::index::NoIndices(glium::index::PrimitiveType::Points);

    let vertex_shader_src = r#"
        #version 330 core

        uniform vec2 uResolution;
        
        layout (location = 0) in ivec2 position;
        layout (location = 1) in ivec2 dimensions;
        layout (location = 2) in vec4 inner_colour;
        layout (location = 3) in vec4 outer_colour;
        layout (location = 4) in ivec2 identification;
        layout (location = 5) in vec4 extra_data_1;
        layout (location = 6) in vec4 extra_data_2;
        
        out VS_OUT
        {
            vec2 dimensions;
            vec4 inner_colour;
            vec4 outer_colour;
            flat ivec2 identification;
            vec4 extra_data_1;
            vec4 extra_data_2;
        } vs_out;
        
        vec2 toClipSpace(vec2 resolution, vec2 from)
        {
            return vec2(
                from.x / (resolution.x / 2.0) - 1.0,
                1.0 - (from.y / (resolution.y / 2.0))
            );
        }
        
        void main()
        {
            gl_Position = vec4(toClipSpace(uResolution, vec2(position)), 0.0, 1.0);
            vs_out.dimensions = dimensions;
            vs_out.inner_colour = inner_colour;
            vs_out.outer_colour = outer_colour;
            vs_out.identification = identification;
            vs_out.extra_data_1 = extra_data_1;
            vs_out.extra_data_2 = extra_data_2;
        }
    "#;

    let geometry_shader_src = r#"
        #version 330 core

        layout(points) in;
        layout(triangle_strip, max_vertices = 4) out;

        uniform vec2 uResolution;

        in VS_OUT
        {
            vec2 dimensions;
            vec4 inner_colour;
            vec4 outer_colour;
            flat ivec2 identification;
            vec4 extra_data_1;
            vec4 extra_data_2;
        } gm_in[];

        out GM_OUT 
        {
            vec2 dimensions;
            vec2 texture_coord;
            vec4 inner_colour;
            vec4 outer_colour;
            flat ivec2 identification;
            vec4 extra_data_1;
            vec4 extra_data_2;
        } gm_out;

        void createVertex(vec2 pos, vec2 scale, vec2 corner, float u, float v) {
            vec2 scaled = scale * corner;
            vec2 transformed = pos + scaled;
            gl_Position = vec4(transformed, 0.0, 1.0);
            gm_out.texture_coord = vec2(u, v);
            gm_out.dimensions = gm_in[0].dimensions;
            gm_out.inner_colour = gm_in[0].inner_colour;
            gm_out.outer_colour = gm_in[0].outer_colour;
            gm_out.identification = gm_in[0].identification;
            gm_out.extra_data_1 = gm_in[0].extra_data_1;
            gm_out.extra_data_2 = gm_in[0].extra_data_2;
            EmitVertex();
        }

        void main()
        {
            vec2 pos = gl_in[0].gl_Position.xy;;
            vec2 size = gm_in[0].dimensions / uResolution.xy; 

            mat3 scale = mat3(
                size.x, 0.0, 0.0,
                0.0, size.y, 0.0,
                0.0, 0.0, 1.0
            );

            float one = 1.0;
            vec2 bottomLeft = vec2(-one, -one);
            vec2 bottomRight = vec2(one, -one);
            vec2 topLeft = vec2(-one, one);
            vec2 topRight = vec2(one, one);
            
            createVertex(pos, size, bottomLeft, 0.0, 1.0);
            createVertex(pos, size, bottomRight, 1.0, 1.0);
            createVertex(pos, size, topLeft, 0.0, 0.0);
            createVertex(pos, size, topRight, 1.0, 0.0);

            EndPrimitive();
        }
    "#;


    let fragment_shader_src = r#"
        #version 330 core

        uniform sampler2DArray font_buffer;
        float smoothness = 0.002;
        
        in GM_OUT 
        {
            vec2 dimensions;
            vec2 texture_coord;
            vec4 inner_colour;
            vec4 outer_colour;
            flat ivec2 identification;
            vec4 extra_data_1;
            vec4 extra_data_2;
        } fs_in;
        
        out vec4 Color;
        
        float median(float r, float g, float b)
        {
            return max(min(r, g), min(max(r, g), b));
        }
        
        float circle_signed_dist(vec2 position, float radius) 
        {
            return length(position) - radius;
        }
        
        float box_signed_dist(in vec2 position, in vec4 corner_radii)
        {
            vec2 bounds = vec2(0.5);
            vec2 quadrant_position = step(vec2(0.5), position);
            int corner_radius_index = int(quadrant_position.x) + int(quadrant_position.y) * 2;
            float corner_radius = corner_radii[corner_radius_index];
        
            vec2 centred_position = position - 0.5;    
            vec2 offset = abs(centred_position) - bounds + corner_radius;
            return min(max(offset.x, offset.y), 0.0) + length(max(offset, 0.0)) - corner_radius;
        }
        
        void main()
        {
            vec3 inner_colour = fs_in.inner_colour.rgb;
            vec3 outer_colour = fs_in.outer_colour.rgb;
            float stroke_width = fs_in.extra_data_1.r / fs_in.dimensions.x;
        
            float alpha = 0.00;
            vec3 current_colour;
            
            if(fs_in.identification.r == 0) 
            {
                float outer_radius = 0.5;
                float dist = circle_signed_dist(fs_in.texture_coord - 0.5, outer_radius);
                float outer = smoothstep(smoothness, -smoothness, dist);
                float inner = smoothstep(-stroke_width + smoothness, -stroke_width - smoothness, dist);
                alpha = smoothstep(0.00, -smoothness, dist);
                current_colour = mix(outer_colour, inner_colour, inner) * outer;
            }
            
            if(fs_in.identification.r == 1) 
            {
                vec4 corner_radii = fs_in.extra_data_2;
                float dist = box_signed_dist(fs_in.texture_coord, corner_radii);
                float outer = smoothstep(smoothness, -smoothness, dist);
                float inner = smoothstep(-stroke_width + smoothness, -stroke_width - smoothness, dist);
                alpha = smoothstep(0.00, -smoothness, dist);
                current_colour = mix(outer_colour, inner_colour, inner) * outer;
            }
        
            if(fs_in.identification.r == 2) 
            {
                vec3 sample = texture(font_buffer, vec3(fs_in.texture_coord, fs_in.identification.g)).rgb;
                float dist = median(sample.r, sample.g, sample.b);
                float width = fwidth(dist);
                alpha = smoothstep(0.5 - width, 0.5 + width, dist);
                current_colour = outer_colour;
            }
        
            Color = vec4(current_colour, alpha);
        }
    "#;

    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, Some(geometry_shader_src)).unwrap();
    
    let framebuffer_dimensions = display.get_framebuffer_dimensions();
    let resolution: [f32;2] = [framebuffer_dimensions.0 as f32, framebuffer_dimensions.1 as f32];
    
    println!("resolution: {:?}", resolution);

    let mut glyphs = vec!();
    let glyph_dimensions = (96, 96);    
    
    println!("loading font texture");
    
    let mut font_image = image::load(Cursor::new(&include_bytes!("../images/segoeui-1.png")[..]), image::ImageFormat::Png)
        .unwrap()
        .to_rgba8();
    
    println!("loaded font texture");
    
    let font_image_dimensions = font_image.dimensions(); 
    let glyph_count = font_image_dimensions.1 / glyph_dimensions.1;
    
    println!("making raw glyph texture array of count {}", glyph_count);
    
    for glyph_index in 0..glyph_count {
        let glyph_image = image::imageops::crop(&mut font_image, 0, glyph_index * glyph_dimensions.1, glyph_dimensions.0, glyph_dimensions.1).to_image();
        let glyph_image_dimensions = glyph_image.dimensions();
        glyphs.push(glium::texture::RawImage2d::from_raw_rgba(glyph_image.into_raw(), glyph_image_dimensions));
    }

    let font_buffer = glium::texture::texture2d_array::Texture2dArray::new(&display, glyphs).unwrap();
    
    println!("made raw glyph texture array of count {}", glyph_count);
    
    let mut time: f32 = 0.0;
    let mut changed = true; 

    event_loop.run(move |event, _, control_flow| {
        let next_frame_time = std::time::Instant::now() + std::time::Duration::from_nanos(16_666_667);
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

        match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                },
                _ => return,
            },
            glutin::event::Event::NewEvents(cause) => match cause {
                glutin::event::StartCause::ResumeTimeReached { .. } => (),
                glutin::event::StartCause::Init => (),
                _ => return,
            },
            _ => return,
        }

        if changed {
            changed = false;

            time += 0.01666666666;

            let uniforms = uniform! {
                uResolution: resolution,
                font_buffer: font_buffer.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear)
            };
        
            let params = glium::DrawParameters {
                blend: glium::Blend::alpha_blending(),
                .. Default::default()
            };

            let mut target = display.draw();
            let draw_frame_start = std::time::Instant::now();
            target.clear_color(0.3, 0.3, 0.5, 1.0);
            target.draw(&vertices, &indices, &program, &uniforms, &params).unwrap();
            target.finish().unwrap();
            let draw_time = std::time::Instant::now() - draw_frame_start;
            if time > 1.0 {
                time = 0.0;
                println!("frame draw time: {:?}", draw_time);
            }
        }
    });
}