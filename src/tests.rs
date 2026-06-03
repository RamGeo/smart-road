use crate::intersection::scheduler::Scheduler;
use crate::intersection::Intersection;
use crate::stats::Stats;
use crate::vehicle::route::{
    Direction, Path, Route, BOX_HALF, BOX_MAX_X, BOX_MAX_Y, BOX_MIN_X, BOX_MIN_Y, CENTER_X,
    CENTER_Y,
};
use crate::vehicle::{
    Vehicle, Velocity, FAST_SPEED, MEDIUM_SPEED, SLOW_SPEED, SAFE_DISTANCE, VEHICLE_SIZE,
};

fn vehicle_at_distance(id: u32, direction: Direction, route: Route, dist: f32) -> Vehicle {
    let mut v = Vehicle::new(id, direction, route);
    v.distance_travelled = dist;
    v
}

fn sample_vehicle(id: u32, crossing_time: f32, max_spd: f32, min_spd: f32) -> Vehicle {
    let mut v = Vehicle::new(id, Direction::North, Route::Straight);
    v.detected_by_scheduler = true;
    v.time_since_detected = crossing_time;
    v.max_speed_reached = max_spd;
    v.min_speed_reached = min_spd;
    v
}

const ALL_DIRECTIONS: [Direction; 4] = [
    Direction::North,
    Direction::South,
    Direction::East,
    Direction::West,
];
const ALL_ROUTES: [Route; 3] = [Route::Right, Route::Straight, Route::Left];

// ── route ─────────────────────────────────────────────────────────────────────

#[test]
fn all_twelve_paths_have_positive_length() {
    for &direction in &ALL_DIRECTIONS {
        for &route in &ALL_ROUTES {
            let path = Path::new(direction, route);
            assert!(
                path.waypoints.len() >= 2,
                "{direction:?} {route:?} needs at least two waypoints"
            );
            assert!(
                path.total_length() > 0.0,
                "{direction:?} {route:?} must have positive length"
            );
        }
    }
}

#[test]
fn position_at_endpoints_matches_waypoints() {
    for &direction in &ALL_DIRECTIONS {
        for &route in &ALL_ROUTES {
            let path = Path::new(direction, route);
            let start = path.waypoints[0];
            let end = *path.waypoints.last().unwrap();
            let total = path.total_length();

            assert_eq!(path.position_at(0.0), start);
            assert_eq!(path.position_at(total), end);
            assert_eq!(path.position_at(-10.0), start);
        }
    }
}

#[test]
fn straight_paths_have_constant_cardinal_heading() {
    let north_straight = Path::new(Direction::North, Route::Straight);
    let heading = north_straight.heading_at(50.0);
    assert!(
        (heading - std::f32::consts::FRAC_PI_2).abs() < 0.01,
        "northbound straight should face south (+y)"
    );

    let west_straight = Path::new(Direction::West, Route::Straight);
    let heading = west_straight.heading_at(50.0);
    assert!(
        heading.abs() < 0.01,
        "west-to-east straight should face east (+x)"
    );
}

#[test]
fn left_turn_changes_heading() {
    let path = Path::new(Direction::North, Route::Left);
    let early = path.heading_at(10.0);
    let late = path.heading_at(path.total_length() - 10.0);
    assert!(
        (early - late).abs() > 0.5,
        "left turn should change heading along the path"
    );
}

#[test]
fn intersection_box_is_centered() {
    assert_eq!(BOX_MIN_X + BOX_HALF, CENTER_X);
    assert_eq!(BOX_MAX_X - BOX_HALF, CENTER_X);
    assert_eq!(BOX_MIN_Y + BOX_HALF, CENTER_Y);
    assert_eq!(BOX_MAX_Y - BOX_HALF, CENTER_Y);
}

// ── vehicle ───────────────────────────────────────────────────────────────────

#[test]
fn speed_matches_velocity_level() {
    let mut v = Vehicle::new(0, Direction::North, Route::Straight);
    v.velocity = Velocity::Stopped;
    assert_eq!(v.speed(), 0.0);
    v.velocity = Velocity::Slow;
    assert_eq!(v.speed(), SLOW_SPEED);
    v.velocity = Velocity::Medium;
    assert_eq!(v.speed(), MEDIUM_SPEED);
    v.velocity = Velocity::Fast;
    assert_eq!(v.speed(), FAST_SPEED);
}

#[test]
fn update_advances_distance_proportional_to_speed() {
    let mut v = Vehicle::new(0, Direction::North, Route::Straight);
    v.velocity = Velocity::Medium;
    v.update(1.0);
    assert!((v.distance_travelled - MEDIUM_SPEED).abs() < 0.01);
    v.update(0.5);
    assert!((v.distance_travelled - MEDIUM_SPEED * 1.5).abs() < 0.01);
}

#[test]
fn update_tracks_speed_extremes_while_moving() {
    let mut v = Vehicle::new(0, Direction::North, Route::Straight);
    v.velocity = Velocity::Fast;
    v.update(0.1);
    assert_eq!(v.max_speed_reached, FAST_SPEED);
    assert_eq!(v.min_speed_reached, FAST_SPEED);

    v.velocity = Velocity::Slow;
    v.update(0.1);
    assert_eq!(v.max_speed_reached, FAST_SPEED);
    assert_eq!(v.min_speed_reached, SLOW_SPEED);
}

#[test]
fn stopped_velocity_does_not_update_min_speed() {
    let mut v = Vehicle::new(0, Direction::North, Route::Straight);
    v.velocity = Velocity::Stopped;
    v.update(0.1);
    assert_eq!(v.min_speed_reached, f32::MAX);
}

#[test]
fn time_since_detected_only_counts_after_scheduler_detection() {
    let mut v = Vehicle::new(0, Direction::North, Route::Straight);
    v.update(1.0);
    assert_eq!(v.time_since_detected, 0.0);

    v.detected_by_scheduler = true;
    v.update(0.5);
    assert!((v.time_since_detected - 0.5).abs() < 0.001);
}

#[test]
fn is_done_when_path_is_exhausted() {
    let mut v = Vehicle::new(0, Direction::North, Route::Straight);
    assert!(!v.is_done());
    v.distance_travelled = v.path.total_length();
    assert!(v.is_done());
    v.distance_travelled = v.path.total_length() + 100.0;
    assert!(v.is_done());
}

#[test]
fn in_intersection_detects_box_membership() {
    let mut v = Vehicle::new(0, Direction::North, Route::Straight);
    v.distance_travelled = 250.0;
    assert!(!v.in_intersection());
    v.distance_travelled = 350.0;
    assert!(v.in_intersection());
}

#[test]
fn distance_to_is_euclidean() {
    let mut a = Vehicle::new(0, Direction::North, Route::Straight);
    let mut b = Vehicle::new(1, Direction::North, Route::Straight);
    a.distance_travelled = 0.0;
    b.distance_travelled = 300.0;
    let (ax, ay) = a.position();
    let (bx, by) = b.position();
    let expected = ((bx - ax).powi(2) + (by - ay).powi(2)).sqrt();
    assert!((a.distance_to(&b) - expected).abs() < 0.01);
}

// ── scheduler ─────────────────────────────────────────────────────────────────

#[test]
fn same_lane_movements_do_not_conflict() {
    assert!(!Scheduler::paths_conflict(
        Direction::North,
        Route::Straight,
        Direction::North,
        Route::Straight,
    ));
    assert!(!Scheduler::paths_conflict(
        Direction::East,
        Route::Left,
        Direction::East,
        Route::Right,
    ));
}

#[test]
fn parallel_opposing_straights_on_separate_lanes_do_not_conflict() {
    assert!(!Scheduler::paths_conflict(
        Direction::North,
        Route::Straight,
        Direction::South,
        Route::Straight,
    ));
    assert!(!Scheduler::paths_conflict(
        Direction::East,
        Route::Straight,
        Direction::West,
        Route::Straight,
    ));
}

#[test]
fn crossing_straights_conflict() {
    assert!(Scheduler::paths_conflict(
        Direction::North,
        Route::Straight,
        Direction::East,
        Route::Straight,
    ));
    assert!(Scheduler::paths_conflict(
        Direction::North,
        Route::Straight,
        Direction::West,
        Route::Straight,
    ));
}

#[test]
fn perpendicular_right_turns_do_not_conflict() {
    assert!(!Scheduler::paths_conflict(
        Direction::North,
        Route::Right,
        Direction::East,
        Route::Right,
    ));
}

#[test]
fn schedule_stops_vehicle_when_intersection_is_occupied() {
    let mut vehicles = vec![
        vehicle_at_distance(0, Direction::North, Route::Straight, 350.0),
        vehicle_at_distance(1, Direction::East, Route::Straight, 470.0),
    ];
    vehicles[1].velocity = Velocity::Medium;

    let scheduler = Scheduler::new();
    scheduler.schedule(&mut vehicles, 0.016);

    assert_eq!(vehicles[1].velocity, Velocity::Stopped);
    assert!(vehicles[1].detected_by_scheduler);
}

#[test]
fn schedule_releases_vehicle_when_intersection_is_clear() {
    let stop_dist = BOX_MIN_Y - 30.0;
    let mut vehicles = vec![vehicle_at_distance(
        0,
        Direction::North,
        Route::Straight,
        stop_dist,
    )];
    vehicles[0].velocity = Velocity::Stopped;

    let scheduler = Scheduler::new();
    scheduler.schedule(&mut vehicles, 0.016);

    assert_eq!(vehicles[0].velocity, Velocity::Fast);
    assert!(vehicles[0].detected_by_scheduler);
}

#[test]
fn schedule_only_releases_one_conflicting_vehicle_per_frame() {
    let north_stop = BOX_MIN_Y - 30.0;
    let mut vehicles = vec![
        vehicle_at_distance(0, Direction::North, Route::Straight, north_stop),
        vehicle_at_distance(1, Direction::East, Route::Straight, 470.0),
    ];
    for v in &mut vehicles {
        v.velocity = Velocity::Stopped;
    }

    let scheduler = Scheduler::new();
    scheduler.schedule(&mut vehicles, 0.016);

    let released = vehicles
        .iter()
        .filter(|v| v.velocity == Velocity::Fast)
        .count();
    let stopped = vehicles
        .iter()
        .filter(|v| v.velocity == Velocity::Stopped)
        .count();
    assert_eq!(released, 1);
    assert_eq!(stopped, 1);
}

#[test]
fn vehicles_inside_box_are_marked_detected_but_not_stopped() {
    let mut vehicles = vec![vehicle_at_distance(
        0,
        Direction::North,
        Route::Straight,
        350.0,
    )];
    vehicles[0].velocity = Velocity::Fast;

    let scheduler = Scheduler::new();
    scheduler.schedule(&mut vehicles, 0.016);

    assert!(vehicles[0].detected_by_scheduler);
    assert_eq!(vehicles[0].velocity, Velocity::Fast);
}

// ── intersection ──────────────────────────────────────────────────────────────

#[test]
fn spawn_succeeds_when_lane_is_clear() {
    let mut inter = Intersection::new();
    assert!(inter.try_spawn(Direction::North, Route::Straight));
    assert_eq!(inter.vehicles.len(), 1);
}

#[test]
fn spawn_throttle_blocks_overlapping_vehicles() {
    let mut inter = Intersection::new();
    assert!(inter.try_spawn(Direction::North, Route::Straight));
    assert!(!inter.try_spawn(Direction::North, Route::Straight));
    assert_eq!(inter.vehicles.len(), 1);
}

#[test]
fn different_directions_can_spawn_simultaneously() {
    let mut inter = Intersection::new();
    assert!(inter.try_spawn(Direction::North, Route::Straight));
    assert!(inter.try_spawn(Direction::South, Route::Straight));
    assert_eq!(inter.vehicles.len(), 2);
}

#[test]
fn safety_distance_slows_follower_on_same_lane() {
    let mut vehicles = vec![
        Vehicle::new(0, Direction::North, Route::Straight),
        Vehicle::new(1, Direction::North, Route::Straight),
    ];
    vehicles[0].distance_travelled = 200.0;
    vehicles[1].distance_travelled = 200.0 - SAFE_DISTANCE + 5.0;

    Intersection::apply_safety_distances(&mut vehicles);

    assert_eq!(vehicles[1].velocity, Velocity::Slow);
    assert_eq!(vehicles[0].velocity, Velocity::Medium);
}

#[test]
fn safety_distance_stops_follower_when_leader_is_stopped() {
    let mut vehicles = vec![
        Vehicle::new(0, Direction::North, Route::Straight),
        Vehicle::new(1, Direction::North, Route::Straight),
    ];
    vehicles[0].distance_travelled = 200.0;
    vehicles[0].velocity = Velocity::Stopped;
    vehicles[1].distance_travelled = 200.0 - SAFE_DISTANCE + 5.0;

    Intersection::apply_safety_distances(&mut vehicles);

    assert_eq!(vehicles[1].velocity, Velocity::Stopped);
}

#[test]
fn safety_distance_clamps_physical_overlap() {
    let mut vehicles = vec![
        Vehicle::new(0, Direction::North, Route::Straight),
        Vehicle::new(1, Direction::North, Route::Straight),
    ];
    vehicles[0].distance_travelled = 200.0;
    vehicles[1].distance_travelled = 199.0;

    Intersection::apply_safety_distances(&mut vehicles);

    let gap = vehicles[0].distance_travelled - vehicles[1].distance_travelled;
    assert!((gap - VEHICLE_SIZE).abs() < 0.01);
}

#[test]
fn update_removes_completed_vehicles_and_records_stats() {
    let mut inter = Intersection::new();
    inter.try_spawn(Direction::North, Route::Straight);
    inter.vehicles[0].detected_by_scheduler = true;
    inter.vehicles[0].distance_travelled = inter.vehicles[0].path.total_length() + 1.0;

    inter.update(0.016);

    assert!(inter.vehicles.is_empty());
    assert_eq!(inter.stats.total_passed, 1);
}

#[test]
fn close_calls_ignored_when_vehicles_not_detected() {
    let mut inter = Intersection::new();
    inter.try_spawn(Direction::North, Route::Straight);
    inter.try_spawn(Direction::South, Route::Straight);
    inter.vehicles[0].distance_travelled = 350.0;
    inter.vehicles[1].distance_travelled = 350.0;
    inter.record_close_calls();
    assert_eq!(inter.stats.close_calls, 0);
}

#[test]
fn close_calls_recorded_when_conflicting_vehicles_too_close() {
    let mut inter = Intersection::new();
    inter.try_spawn(Direction::North, Route::Straight);
    inter.try_spawn(Direction::East, Route::Straight);

    let mut found = false;
    'search: for d0 in (0..800).step_by(2) {
        for d1 in (0..800).step_by(2) {
            inter.vehicles[0].distance_travelled = d0 as f32;
            inter.vehicles[1].distance_travelled = d1 as f32;
            inter.vehicles[0].detected_by_scheduler = true;
            inter.vehicles[1].detected_by_scheduler = true;
            if inter.vehicles[0].distance_to(&inter.vehicles[1]) >= SAFE_DISTANCE {
                continue;
            }
            inter.record_close_calls();
            assert_eq!(inter.stats.close_calls, 1);
            inter.record_close_calls();
            assert_eq!(
                inter.stats.close_calls, 1,
                "duplicate close-call pairs should not inflate the count"
            );
            found = true;
            break 'search;
        }
    }
    assert!(
        found,
        "expected conflicting paths to pass within safe distance somewhere in the grid"
    );
}

#[test]
fn max_simultaneous_tracks_peak_intersection_occupancy() {
    let mut inter = Intersection::new();
    inter.try_spawn(Direction::North, Route::Straight);
    inter.try_spawn(Direction::South, Route::Straight);
    inter.vehicles[0].distance_travelled = 350.0;
    inter.vehicles[1].distance_travelled = inter.vehicles[1].path.total_length() - 350.0;

    inter.update(0.016);

    assert!(inter.stats.max_simultaneous >= 1);
}

#[test]
fn simulation_advances_vehicle_without_collision_on_single_lane() {
    let mut inter = Intersection::new();
    assert!(inter.try_spawn(Direction::North, Route::Straight));
    let mut follower = Vehicle::new(1, Direction::North, Route::Straight);
    follower.distance_travelled = 50.0;
    inter.vehicles.push(follower);

    for _ in 0..500 {
        inter.update(0.016);
    }

    let same_lane: Vec<_> = inter
        .vehicles
        .iter()
        .filter(|v| v.direction == Direction::North && v.route == Route::Straight)
        .collect();
    for pair in same_lane.windows(2) {
        let gap = pair[0].distance_travelled - pair[1].distance_travelled;
        assert!(gap >= VEHICLE_SIZE - 0.1);
    }
}

// ── stats ─────────────────────────────────────────────────────────────────────

#[test]
fn record_vehicle_exit_updates_all_metrics() {
    let mut stats = Stats::new();
    let v = sample_vehicle(0, 2.5, FAST_SPEED, SLOW_SPEED);
    stats.record_vehicle_exit(&v);

    assert_eq!(stats.total_passed, 1);
    assert_eq!(stats.max_speed, FAST_SPEED);
    assert_eq!(stats.min_speed, SLOW_SPEED);
    assert_eq!(stats.max_crossing_time, 2.5);
    assert_eq!(stats.min_crossing_time, 2.5);
    assert!((stats.average_crossing_time() - 2.5).abs() < 0.001);
}

#[test]
fn record_vehicle_exit_tracks_min_and_max_across_vehicles() {
    let mut stats = Stats::new();
    stats.record_vehicle_exit(&sample_vehicle(0, 1.0, 200.0, 60.0));
    stats.record_vehicle_exit(&sample_vehicle(1, 3.0, 120.0, 80.0));

    assert_eq!(stats.total_passed, 2);
    assert_eq!(stats.max_speed, 200.0);
    assert_eq!(stats.min_speed, 60.0);
    assert_eq!(stats.max_crossing_time, 3.0);
    assert_eq!(stats.min_crossing_time, 1.0);
    assert!((stats.average_crossing_time() - 2.0).abs() < 0.001);
}

#[test]
fn record_vehicle_exit_ignores_zero_crossing_time_for_averages() {
    let mut stats = Stats::new();
    stats.record_vehicle_exit(&sample_vehicle(0, 0.0, MEDIUM_SPEED, MEDIUM_SPEED));
    assert_eq!(stats.total_passed, 1);
    assert_eq!(stats.average_crossing_time(), 0.0);
}

#[test]
fn close_call_pairs_are_counted_once() {
    let mut stats = Stats::new();
    stats.record_close_call_pair(1, 2);
    stats.record_close_call_pair(2, 1);
    stats.record_close_call_pair(1, 2);
    assert_eq!(stats.close_calls, 1);
}

#[test]
fn average_crossing_time_is_zero_with_no_vehicles() {
    let stats = Stats::new();
    assert_eq!(stats.average_crossing_time(), 0.0);
}
