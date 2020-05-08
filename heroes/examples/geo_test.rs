use cgmath::{assert_relative_eq, prelude::*, InnerSpace, Point2, Vector2, VectorSpace};
use geo::algorithm::closest_point::ClosestPoint;
use geo::algorithm::euclidean_distance::EuclideanDistance;
use geo::{Line, Point, Polygon};

fn main() {
    println!("done");
}

#[test]
fn test_1() {
    let line = Line::new(Point::new(2.0, 3.0), Point::new(4.0, 1.0));

    let p1 = Point::new(2.0, 1.0);
    let p2 = Point::new(4.0, -2.0);

    for p in &[p1, p2] {
        let dist = p.euclidean_distance(&line);
        let close = line.closest_point(&p);
        println!("p {:?} dist {:?} close {:?}", p, dist, close);
    }

    panic!("not");
}

fn check_point_to_segment(
    segment_0: Vector2<f32>,
    segment_1: Vector2<f32>,
    point: Vector2<f32>,
) -> Option<f32> {
    let proj = line_segment_project(segment_0, segment_1, point);
    None
}

#[test]
fn test_simplify() {
    let segment_0 = Vector2::new(0.0, 0.0);
    let segment_1 = Vector2::new(10.0, 0.0);

    assert_eq!(
        check_point_to_segment(segment_0, segment_1, Vector2::new(5.0, 5.0)),
        Some(5.0)
    );

    assert_eq!(
        check_point_to_segment(segment_0, segment_1, Vector2::new(5.0, -5.0)),
        Some(5.0)
    );

    assert_eq!(
        check_point_to_segment(segment_0, segment_1, Vector2::new(-5.0, 5.0)),
        None
    );

    assert_eq!(
        check_point_to_segment(segment_0, segment_1, Vector2::new(15.0, 5.0)),
        None
    );
}

/// project point into the line segmenet
// http://sites.science.oregonstate.edu/math/home/programs/undergrad/CalculusQuestStudyGuides/vcalc/dotprod/dotprod.html
fn line_segment_project(
    line_0: Vector2<f32>,
    line_1: Vector2<f32>,
    point: Vector2<f32>,
) -> Vector2<f32> {
    // multiply the proportion of the projection into the segment vector and add initial position
    (line_1 - line_0) * line_segment_project_percent(line_0, line_1, point) + line_0
}

/// return percentage of segment the point belong, < 0 for before, > 1 for after
fn line_segment_project_percent(
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
