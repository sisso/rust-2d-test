use approx::assert_relative_eq;
use nalgebra::{
    self as na, Matrix4, Point2, Rotation2, Similarity2, Similarity3, Vector2, Vector3,
};

pub type P2 = Point2<f32>;
pub type V2 = Vector2<f32>;
pub type Sim2 = Similarity2<f32>;
pub type M4 = Matrix4<f32>;
pub const PI: f32 = std::f32::consts::PI;

pub fn v2(x: f32, y: f32) -> V2 {
    Vector2::new(x, y)
}

pub fn p2(x: f32, y: f32) -> P2 {
    Point2::new(x, y)
}

#[derive(Debug, Clone)]
pub struct Transform2 {
    pos: P2,
    scale: f32,
    angle: f32,
    similarity: Similarity2<f32>,
}

impl Transform2 {
    pub fn new(pos: P2, scale: f32, rotation: f32) -> Self {
        Transform2 {
            pos,
            scale,
            angle: rotation,
            similarity: Similarity2::new(pos.coords.clone(), rotation, scale),
        }
    }

    pub fn identity() -> Self {
        Transform2 {
            pos: Point2::origin(),
            scale: 1.0,
            angle: 0.0,
            similarity: Similarity2::identity(),
        }
    }

    pub fn get_pos(&self) -> &P2 {
        &self.pos
    }

    pub fn get_angle(&self) -> f32 {
        self.angle
    }

    pub fn get_scale(&self) -> f32 {
        self.scale
    }

    pub fn set_pos(&mut self, p: P2) {
        self.pos = p;
        self.recriate_similarity();
    }

    pub fn set_scale(&mut self, s: f32) {
        self.scale = s;
        self.recriate_similarity();
    }

    pub fn set_angle(&mut self, r: f32) {
        self.angle = r;
        self.recriate_similarity();
    }

    pub fn translate(&mut self, v: V2) {
        self.pos = self.pos + v;
        self.recriate_similarity();
    }

    pub fn scale(&mut self, v: f32) {
        self.scale *= v;
        self.recriate_similarity();
    }

    pub fn get_similarity(&self) -> &Similarity2<f32> {
        &self.similarity
    }

    pub fn point_to_local(&self, p: &P2) -> P2 {
        self.similarity.transform_point(&p)
    }

    pub fn local_to_point(&self, p: &P2) -> P2 {
        self.similarity.inverse().transform_point(&p)
    }

    pub fn get_matrix(&self) -> M4 {
        let sim = Similarity3::new(
            Vector3::new(self.pos.coords.x, self.pos.coords.y, 0.0),
            Vector3::new(0.0, 0.0, self.angle),
            self.scale,
        );

        sim.into()
    }

    fn recriate_similarity(&mut self) {
        self.similarity = Similarity2::new(self.pos.coords.clone(), self.angle, self.scale);
    }
}

/// returns the value between v0 and v1 on t
pub fn lerp(v0: f32, v1: f32, t: f32) -> f32 {
    v0 + clamp01(t) * (v1 - v0)
}

/// returns % of t between v0 and v1
pub fn inverse_lerp(v0: f32, v1: f32, t: f32) -> f32 {
    if v0 == v1 {
        0.0
    } else {
        clamp01((t - v0) / (v1 - v0))
    }
}

///
/// Lerp between v0 and v1 giving the value of 5 between t0 and t1
///
/// t <= t0, returns v0
/// t >= t1, returns v1
///
pub fn lerp_2(v0: f32, v1: f32, t0: f32, t1: f32, t: f32) -> f32 {
    let tt = inverse_lerp(t0, t1, t);
    lerp(v0, v1, tt)
}

pub fn clamp01(v: f32) -> f32 {
    if v < 0.0 {
        0.0
    } else if v > 1.0 {
        1.0
    } else {
        v
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_lerp_2() {
        assert_eq!(lerp_2(0.0, 1.0, 0.0, 1.0, 0.5), 0.5);
        assert_eq!(lerp_2(0.0, 2.0, 0.0, 1.0, 0.5), 1.0);
        assert_eq!(lerp_2(0.0, 1.0, 0.0, 2.0, 1.0), 0.5);
    }
}

pub fn angle_vector(v: V2) -> f32 {
    v.y.atan2(v.x)
}

// TODO: remove?
pub fn rotate_vector(dir: V2, point: P2) -> P2 {
    let angle = angle_vector(dir);
    rotate_vector_by_angle(point, angle)
}

// TODO: remove?
pub fn rotate_vector_by_angle(point: P2, angle: f32) -> P2 {
    let rotation = Rotation2::new(angle);
    rotation * point
}

pub fn rotate_towards(vec: V2, dir: V2, max_angle: f32) -> V2 {
    let current_angle = vec.y.atan2(vec.x);
    let target_angle = dir.y.atan2(dir.x);

    let mut delta = target_angle - current_angle;
    let pi = std::f32::consts::PI;
    let pi2 = 2.0 * pi;

    while delta > pi {
        delta -= pi2;
    }

    while delta < -pi {
        delta += pi2;
    }

    if delta > max_angle {
        delta = max_angle;
    } else if delta < -max_angle {
        delta = -max_angle;
    }

    let new_angle = current_angle + delta;
    v2(new_angle.cos(), new_angle.sin())
}

pub fn rad2deg(value: f32) -> f32 {
    180.0 * (value / PI)
}

pub fn deg2rad(value: f32) -> f32 {
    (value / 180.0) * PI
}

/// compute a vector that go from the point to the segment
pub fn compute_vector_from_point_to_segment(pos: P2, vec: V2, point: P2) -> Option<V2> {
    let proj = line_segment_project_percent(pos, vec, point);
    // println!("{:?}", proj);
    if proj >= 0.0 && proj <= 1.0 {
        let segment_point: V2 = proj * vec + pos.coords;
        let vector = segment_point - point.coords;
        // println!("{:?}", proj_v);
        Some(vector)
    } else {
        None
    }
}

/// project point into the line segmenet
// http://sites.science.oregonstate.edu/math/home/programs/undergrad/CalculusQuestStudyGuides/vcalc/dotprod/dotprod.html
pub fn line_segment_project(pos: P2, vec: V2, point: P2) -> V2 {
    // multiply the proportion of the projection into the segment vector and add initial position
    vec * line_segment_project_percent(pos, vec, point) + pos.coords
}

/// return percentage of segment the point belong, < 0 for before, > 1 for after
pub fn line_segment_project_percent(pos: P2, vec: V2, point: P2) -> f32 {
    let line_mag = vec.magnitude();
    vec.dot(&(point.coords - pos.coords)) * (1.0 / (line_mag * line_mag))
}

#[test]
fn test_segement_projection() {
    let point: P2 = Point2::new(5.0, 5.0);
    let line_0 = Vector2::new(-10.0, -10.0);
    let line_1 = Vector2::new(10.0, 10.0);
    let line_pos = Point2::from(line_0);
    let line_vec = line_1 - line_0;

    let proj = line_segment_project(line_pos, line_vec, point);
    assert_relative_eq!(proj, Vector2::new(5.0, 5.0));
}

#[test]
fn test_segment_projection_percent() {
    let point: P2 = Point2::new(10.0, 0.0);
    let line_0 = Vector2::new(-10.0, -10.0);
    let line_1 = Vector2::new(10.0, -10.0);
    let line_pos = Point2::from(line_0);
    let line_vec = line_1 - line_0;

    let proj = line_segment_project_percent(line_pos, line_vec, point);
    assert_relative_eq!(proj, 1.0);
}

#[test]
fn test_rotate_towards() {
    let vector = Vector2::new(0.0, 1.0);
    let desired = Vector2::new(1.0, 0.0);

    let new_vector = rotate_towards(vector, desired, deg2rad(45.0).into());
    assert_relative_eq!(new_vector, Vector2::new(0.70710677, 0.70710677));

    let new_vector = rotate_towards(new_vector, desired, deg2rad(45.0).into());
    assert_relative_eq!(new_vector, Vector2::new(1.0, 0.0));
}

#[test]
fn test_nalgebra() {
    // giving a vector
    let v = Vector2::new(0.0, 1.0);
    // to a point
    let p = Point2::from(v);
    let p = Point2::origin() + v;
    // back to coords
    let v = p.coords;

    // translation
    let translation = Vector2::new(10.0, 0.0);

    // transformation
    let s1 = Similarity2::new(translation, deg2rad(-90.0), 1.0);
    assert_relative_eq!(s1 * p, Point2::new(11.0, 0.0));
    assert_relative_eq!(s1 * v, Vector2::new(1.0, 0.0));
}
