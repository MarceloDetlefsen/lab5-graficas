use crate::fragment::Fragment;
use crate::vertex::Vertex;

pub fn triangle(a: &Vertex, b: &Vertex, c: &Vertex) -> Vec<Fragment> {
    let mut fragments = Vec::new();
    for (start, end) in [(a, b), (b, c), (c, a)] {
        let dx = end.transformed_position.x - start.transformed_position.x;
        let dy = end.transformed_position.y - start.transformed_position.y;
        let steps = dx.abs().max(dy.abs()).ceil() as usize;
        for step in 0..=steps {
            let t = if steps == 0 { 0.0 } else { step as f32 / steps as f32 };
            let p = start.transformed_position + (end.transformed_position - start.transformed_position) * t;
            let color = start.color + (end.color - start.color) * t;
            fragments.push(Fragment::new(p.x, p.y, color, p.z));
        }
    }
    fragments
}
