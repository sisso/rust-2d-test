use cgmath::{assert_relative_eq, prelude::*, InnerSpace, Point2, Vector2, VectorSpace};
use geo::algorithm::closest_point::ClosestPoint;
use geo::algorithm::euclidean_distance::EuclideanDistance;
use geo::{Line, Point, Polygon};

/// compute a vector that go from point to segment
pub fn compute_vector_from_point_to_segment(
    pos: Point<f32>,
    vec: Vector2<f32>,
    point: Vector2<f32>,
) -> Option<Vector2<f32>> {
    let proj = line_segment_project_percent(segment_0, segment_1, point);
    // println!("{:?}", proj);
    if proj >= 0.0 && proj <= 1.0 {
        let segment_point = (segment_1 - segment_0) * proj + segment_0;
        let vector = segment_point - point;
        // println!("{:?}", proj_v);
        Some(vector)
    } else {
        None
    }
}

/// project point into the line segmenet
// http://sites.science.oregonstate.edu/math/home/programs/undergrad/CalculusQuestStudyGuides/vcalc/dotprod/dotprod.html
pub fn line_segment_project(
    line_0: Vector2<f32>,
    line_1: Vector2<f32>,
    point: Vector2<f32>,
) -> Vector2<f32> {
    // multiply the proportion of the projection into the segment vector and add initial position
    (line_1 - line_0) * line_segment_project_percent(line_0, line_1, point) + line_0
}

/// return percentage of segment the point belong, < 0 for before, > 1 for after
pub fn line_segment_project_percent(
    line_0: Vector2<f32>,
    line_1: Vector2<f32>,
    point: Vector2<f32>,
) -> f32 {
    let line = line_1 - line_0;
    let line_mag = line.magnitude();
    line.dot(point - line_0) / (line_mag * line_mag)
}

#[test]
fn test_segement_projection() {
    let point: Vector2<f32> = Vector2::new(5.0, 5.0);
    let line_0 = Vector2::new(-10.0, -10.0);
    let line_1 = Vector2::new(10.0, 10.0);

    let proj = line_segment_project(line_0, line_1, point);
    assert_relative_eq!(proj, Vector2::new(5.0, 5.0));
}

#[test]
fn test_segement_projection_percent() {
    let point: Vector2<f32> = Vector2::new(10.0, 0.0);
    let line_0 = Vector2::new(-10.0, -10.0);
    let line_1 = Vector2::new(10.0, -10.0);

    let proj = line_segment_project_percent(line_0, line_1, point);
    assert_relative_eq!(proj, 1.0);
}
