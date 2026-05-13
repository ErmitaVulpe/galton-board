use bevy::ecs::resource::Resource;

#[derive(Debug, Resource)]
pub struct PascalTriangle(Vec<Vec<usize>>);

impl PascalTriangle {
    pub fn new() -> Self {
        Self(vec![vec![1]])
    }

    pub fn get_layer(&mut self, layer: usize) -> &[usize] {
        if layer >= self.0.len() {
            for i in self.0.len()..=layer {
                let mut buf = Vec::with_capacity(i);
                buf.push(1);

                let prev_layer = &self.0[i - 1];
                for i in 0..prev_layer.len() - 1 {
                    buf.push(prev_layer[i] + prev_layer[i + 1]);
                }

                buf.push(1);
                self.0.push(buf);
            }
        }

        &self.0[layer]
    }
}

impl Default for PascalTriangle {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn dupa() {
        let mut a = PascalTriangle::new();
        assert_eq!(a.get_layer(4), [1, 4, 6, 4, 1]);
        assert_eq!(
            a.get_layer(12),
            [1, 12, 66, 220, 495, 792, 924, 792, 495, 220, 66, 12, 1]
        );
    }
}
