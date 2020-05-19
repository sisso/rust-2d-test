use cgmath::{
    assert_relative_eq, prelude::*, Deg, Euler, InnerSpace, Matrix3, Matrix4, Point2, Point3,
    Quaternion, Rad, Transform as CTransform, Vector2, Vector3, VectorSpace,
};

pub type P2 = Point2<f32>;
pub type V2 = Vector2<f32>;
pub type Transform = Matrix4<f32>;

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

pub fn rotate_vector(dir: V2, point: V2) -> V2 {
    let angle = dir.y.atan2(dir.x);

    let qt = Quaternion::from(Euler {
        x: Rad(0.0),
        y: Rad(0.0),
        z: Rad(angle),
    });

    let pointv3 = Vector3::new(point.x, point.y, 0.0);
    let rotated = qt * pointv3;
    Vector2::new(rotated.x, rotated.y)
}

pub fn rotate_vector_by_angle(vec: V2, angle: Rad<f32>) -> V2 {
    let qt = Quaternion::from(Euler {
        x: Rad(0.0),
        y: Rad(0.0),
        z: angle,
    });

    let pointv3 = Vector3::new(vec.x, vec.y, 0.0);
    let rotated = qt * pointv3;
    Vector2::new(rotated.x, rotated.y)
}

pub fn rotate_towards(vec: V2, dir: V2, max_angle: Rad<f32>) -> V2 {
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

    if delta > max_angle.0 {
        delta = max_angle.0;
    } else if delta < -max_angle.0 {
        delta = -max_angle.0;
    }

    let new_angle = current_angle + delta;
    Vector2::new(new_angle.cos(), new_angle.sin())
}

pub fn rad2deg(value: f32) -> f32 {
    Deg::from(Rad(value)).0
}

pub fn deg2rad(value: f32) -> f32 {
    Rad::from(Deg(value)).0
}

/// compute a vector that go from the point to the segment
pub fn compute_vector_from_point_to_segment(pos: P2, vec: V2, point: P2) -> Option<V2> {
    let proj = line_segment_project_percent(pos, vec, point);
    // println!("{:?}", proj);
    if proj >= 0.0 && proj <= 1.0 {
        let segment_point: V2 = vec * proj + pos.to_vec();
        let vector = segment_point - point.to_vec();
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
    vec * line_segment_project_percent(pos, vec, point) + pos.to_vec()
}

/// return percentage of segment the point belong, < 0 for before, > 1 for after
pub fn line_segment_project_percent(pos: P2, vec: V2, point: P2) -> f32 {
    let line_mag = vec.magnitude();
    vec.dot(point - pos) / (line_mag * line_mag)
}

#[test]
fn test_segement_projection() {
    let point: P2 = Point2::new(5.0, 5.0);
    let line_0 = Vector2::new(-10.0, -10.0);
    let line_1 = Vector2::new(10.0, 10.0);
    let line_pos = Point2::from_vec(line_0);
    let line_vec = line_1 - line_0;

    let proj = line_segment_project(line_pos, line_vec, point);
    assert_relative_eq!(proj, Vector2::new(5.0, 5.0));
}

#[test]
fn test_segment_projection_percent() {
    let point: P2 = Point2::new(10.0, 0.0);
    let line_0 = Vector2::new(-10.0, -10.0);
    let line_1 = Vector2::new(10.0, -10.0);
    let line_pos = Point2::from_vec(line_0);
    let line_vec = line_1 - line_0;

    let proj = line_segment_project_percent(line_pos, line_vec, point);
    assert_relative_eq!(proj, 1.0);
}

#[test]
fn test_rotate_towards() {
    let vector = Vector2::new(0.0, 1.0);
    let desired = Vector2::new(1.0, 0.0);

    let new_vector = rotate_towards(vector, desired, Deg(45.0).into());
    assert_relative_eq!(new_vector, Vector2::new(0.70710677, 0.70710677));

    let new_vector = rotate_towards(new_vector, desired, Deg(45.0).into());
    assert_relative_eq!(new_vector, Vector2::new(1.0, 0.0));
}

#[test]
fn test_transform_with_cgmath() {
    let m0 = Matrix4::<f32>::identity();
    let m1 = Matrix4::from_translation(Vector3::new(10.0, 2.0, 0.0));
    let m2 = m0 * m1;
    let m3 = Matrix4::from_angle_z(Deg(-90.0));
    let m4 = m2 * m3;
    let p = Point3::new(0.0, 1.0, 0.0);

    let result = m2.transform_point(p);
    println!("{:?}", result);

    let result = m3.transform_point(p);
    println!("{:?}", result);

    let result = m4.transform_point(p);
    println!("{:?}", result);
}

#[test]
fn test_nalgebra_glm() {
    use glm::*;
    use nalgebra_glm as glm;

    let v = glm::vec2(0.0, 1.0);
    let m1: glm::TMat3<f32> = glm::translation2d(&glm::vec2(5.0, 0.0));
    let m2: glm::TMat3<f32> = glm::rotation2d(glm::pi::<f32>() * -0.5);
    let m3: glm::TMat3<f32> = m1 * m2;

    let v3 = vec3(v.x, v.y, 1.0);

    println!("{:?}", v);
    println!("{:?}", m3 * glm::vec2_to_vec3(&v));
    println!("{:?}", m3 * v3);
}

#[test]
fn test_nalgebra() {
    use na::*;
    use nalgebra as na;

    let v = na::Vector2::new(0.0, 1.0);
    let p = na::Point2::from(v);
    let translation = na::Vector2::new(10.0, 0.0);

    println!("{:?}", v);
    println!("{:?}", p);
    println!("{:?}", p + translation);

    let v = p.coords;
    println!("{:?}", v);

    let s1 = Similarity2::new(translation, deg2rad(90.0), 1.0);
    println!("{:?}", s1 * p);
}
