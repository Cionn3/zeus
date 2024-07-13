use eframe::egui::{
    epaint::{Vertex, WHITE_UV},
    Color32, Mesh, Pos2, Rect,
};

/// Generate a mesh for a vertical gradient color
pub fn vertical_gradient_mesh(rect: Rect, top_color: Color32, bottom_color: Color32) -> Mesh {
    let mut mesh = Mesh::default();

    let top_left = Vertex {
        pos: Pos2::new(rect.left(), rect.top()),
        uv: WHITE_UV,
        color: top_color,
    };

    let top_right = Vertex {
        pos: Pos2::new(rect.right(), rect.top()),
        uv: WHITE_UV,
        color: top_color,
    };

    let bottom_left = Vertex {
        pos: Pos2::new(rect.left(), rect.bottom()),
        uv: WHITE_UV,
        color: bottom_color,
    };

    let bottom_right = Vertex {
        pos: Pos2::new(rect.right(), rect.bottom()),
        uv: WHITE_UV,
        color: bottom_color,
    };

    mesh.vertices.push(top_left);
    mesh.vertices.push(top_right);
    mesh.vertices.push(bottom_left);
    mesh.vertices.push(bottom_right);

    mesh.indices.extend_from_slice(&[0, 1, 2, 2, 1, 3]);

    mesh
}



pub fn vertical_gradient_mesh_2(rect: Rect, colors: &[Color32]) -> Mesh {
    let mut mesh = Mesh::default();

    let steps = colors.len() - 1;
    let height_step = rect.height() / steps as f32;

    for (i, color) in colors.iter().enumerate() {
        let y_start = rect.top() + i as f32 * height_step;
        let y_end = y_start + height_step;

        if i < steps {
            let next_color = colors[i + 1];

            let top_left = Vertex {
                pos: Pos2::new(rect.left(), y_start),
                uv: WHITE_UV,
                color: *color,
            };

            let top_right = Vertex {
                pos: Pos2::new(rect.right(), y_start),
                uv: WHITE_UV,
                color: *color,
            };

            let bottom_left = Vertex {
                pos: Pos2::new(rect.left(), y_end),
                uv: WHITE_UV,
                color: next_color,
            };

            let bottom_right = Vertex {
                pos: Pos2::new(rect.right(), y_end),
                uv: WHITE_UV,
                color: next_color,
            };

            mesh.vertices.push(top_left);
            mesh.vertices.push(top_right);
            mesh.vertices.push(bottom_left);
            mesh.vertices.push(bottom_right);

            let idx = mesh.vertices.len() as u32;
            mesh.indices.extend_from_slice(&[idx - 4, idx - 3, idx - 2, idx - 2, idx - 3, idx - 1]);
        }
    }

    mesh
}