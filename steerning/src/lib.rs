use cgmath::{
    assert_relative_eq, prelude::*, Euler, InnerSpace, Point2, Quaternion, Rad, Vector2, Vector3,
    VectorSpace,
};

pub type P2 = Point2<f32>;
pub type V2 = Vector2<f32>;

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
fn test_segement_projection_percent() {
    let point: P2 = Point2::new(10.0, 0.0);
    let line_0 = Vector2::new(-10.0, -10.0);
    let line_1 = Vector2::new(10.0, -10.0);
    let line_pos = Point2::from_vec(line_0);
    let line_vec = line_1 - line_0;

    let proj = line_segment_project_percent(line_pos, line_vec, point);
    assert_relative_eq!(proj, 1.0);
}
