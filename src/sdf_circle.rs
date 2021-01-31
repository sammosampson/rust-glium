pub fn run() {
    #[allow(unused_imports)]
    use glium::{glutin, Surface};

    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    #[derive(Copy, Clone)]
    pub struct QuadShapeVertex {
        position: [f32; 2],
        dimensions: [f32; 2]
    }

    impl QuadShapeVertex {
        pub fn with_position_and_size(
            position: [f32; 2],
            dimensions: [f32; 2]) -> Self {
            Self {
                position,
                dimensions
            }
        }   
    }

    implement_vertex!(QuadShapeVertex, position, dimensions);

    let shape = glium::vertex::VertexBuffer::new(
        &display, 
        &[
            QuadShapeVertex::with_position_and_size([100.0, 100.0], [100.0, 100.0]),
            QuadShapeVertex::with_position_and_size([200.0, 100.0], [200.0, 200.0]),
            QuadShapeVertex::with_position_and_size([100.0, 400.0], [50.0, 50.0]),
            QuadShapeVertex::with_position_and_size([800.0, 100.0], [25.0, 25.0])
        ]).unwrap();

    let indices = glium::index::NoIndices(glium::index::PrimitiveType::Points);

    let vertex_shader_src = r#"
        #version 330 core

        uniform vec2 uResolution;
        //vec2 uResolution = vec2(1800.0, 1400.0);
        
        layout (location = 0) in vec2 position;
        layout (location = 1) in vec2 dimensions;
        
        out vec2 Dimensions;
        
        vec2 toClipSpace(vec2 from)
        {
            return vec2(
                from.x / (uResolution.x / 2.0) - 1.0,
                1.0 - (from.y / (uResolution.y / 2.0))
            );
        }
        
        void main()
        {
            gl_Position = vec4(toClipSpace(position), 0.0, 1.0);
            Dimensions = dimensions / uResolution.xy;
        }
    "#;

    let geometry_shader_src = r#"
        #version 330 core

        layout(points) in;
        layout(triangle_strip, max_vertices = 4) out;
        
        in vec2 Dimensions[];
        out vec2 textureCoord;
        
        void createVertex(vec2 pos, vec2 scale, vec2 corner, float u, float v) {
            vec2 scaled = scale * corner;
            vec2 transformed = pos + scaled;
            gl_Position = vec4(transformed, 0.0, 1.0);
            textureCoord = vec2(u, v);
            EmitVertex();
        }
        
        void main()
        {
            vec2 pos = gl_in[0].gl_Position.xy;;
            vec2 size = Dimensions[0]; 
        
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

        uniform samplerBuffer uDataBuffer;
        uniform vec3 outerColor;
        uniform vec3 innerColor;
        uniform float innerRadius;
        uniform float smoothness;
        
        in vec2 textureCoord;
        
        out vec4 Color;
        
        void main()
        {
            //int offset = 6; // 0 = red, 3 = green, 6 = magenta
            float r = texelFetch(uDataBuffer, 0).r;
            float g = texelFetch(uDataBuffer, 1).r;
            float b = texelFetch(uDataBuffer, 2).r;
                    
            vec3 innerColor1 = vec3(r, g, b);
        
            float outerRadius = 0.5;
            vec2 uv = textureCoord - 0.5;
            float dist = length(uv);
            float outer = smoothstep(outerRadius + smoothness, outerRadius - smoothness, dist);
            float inner = smoothstep(innerRadius + smoothness, innerRadius - smoothness, dist);
            float alpha = smoothstep(outerRadius, outerRadius - smoothness, dist);
            
            vec3 currentColor = mix(outerColor, innerColor1, inner) * outer;
            
            Color = vec4(currentColor, alpha);
        }
    "#;

    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, Some(geometry_shader_src)).unwrap();
    
    let framebuffer_dimensions = display.get_framebuffer_dimensions();
    let data:[f32;3] = [1.0, 1.0, 0.0];
    let buffer_texture = glium::texture::buffer_texture::BufferTexture::immutable(
        &display,
        &data, 
        glium::texture::buffer_texture::BufferTextureType::Float).unwrap();
    let inner_radius: f32 = 0.45;
    let smoothness: f32 = 0.002;
    let resolution: [f32;2] = [framebuffer_dimensions.0 as f32, framebuffer_dimensions.1 as f32];
    println!("{} {}", resolution[0], resolution[1]);
    let inner_color: [f32;3] = [0.5, 0.5, 0.5];
    let outer_color: [f32;3] = [0.0, 0.0, 0.0];

    let uniforms = uniform! {
        uResolution: resolution,
        uDataBuffer: buffer_texture,
        outerColor: outer_color,
        innerColor: inner_color,
        innerRadius: inner_radius,
        smoothness: smoothness
    };

    let params = glium::DrawParameters {
        blend: glium::Blend::alpha_blending(),
        .. Default::default()
    };

    let mut target = display.draw();
    target.clear_color(0.3, 0.3, 0.5, 1.0);
    target.draw(&shape, &indices, &program, &uniforms, &params).unwrap();
    target.finish().unwrap();

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
    });
}