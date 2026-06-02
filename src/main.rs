use smart_road::vehicle::route::{Direction, Path, Route};

fn main() {
    let cases = [
        (Direction::North, Route::Right),
        (Direction::North, Route::Straight),
        (Direction::North, Route::Left),
        (Direction::South, Route::Right),
        (Direction::South, Route::Straight),
        (Direction::South, Route::Left),
        (Direction::East, Route::Right),
        (Direction::East, Route::Straight),
        (Direction::East, Route::Left),
        (Direction::West, Route::Right),
        (Direction::West, Route::Straight),
        (Direction::West, Route::Left),
    ];

    println!("{:<8} {:<10} {:>5}  {:>8}  heading(start)", "From", "Route", "wpts", "length px");
    println!("{}", "-".repeat(52));

    for (dir, route) in cases {
        let path = Path::new(dir, route);
        let len = path.total_length();
        let heading = path.heading_at(0.0);
        let heading_deg = heading.to_degrees();
        println!(
            "{:<8} {:<10} {:>5}  {:>8.1}  {:.1}°",
            format!("{:?}", dir),
            format!("{:?}", route),
            path.waypoints.len(),
            len,
            heading_deg,
        );
    }
}
