// ported from: https://github.com/Unity-Technologies/UnityCsReference/blob/master/Runtime/Export/Math/Mathf.cs

// Moves a value /current/ towards /target/.
pub fn move_towards(current: f32, target: f32, max_delta: f32) -> f32 {
    if f32::abs(target - current) <= max_delta {
        return target;
    }
    return current + f32::signum(target - current) * max_delta;
}

// Same as ::ref::MoveTowards but makes sure the values interpolate correctly when they wrap around 360 degrees.
pub fn move_towards_angle(current: f32, target: f32, max_delta: f32) -> f32 {
    let delta_angle = delta_angle(current, target);
    if -max_delta < delta_angle && delta_angle < max_delta {
        return target;
    }
    let new_target = current + delta_angle;
    return move_towards(current, new_target, max_delta);
}

// Calculates the shortest difference between two given angles.
pub fn delta_angle(current: f32, target: f32) -> f32 {
    let mut delta = repeat(target - current, 360.0f32);
    if delta > 180.0f32 {
        delta -= 360.0f32; // This is crap. Bevy is in Radians. TODO.
    }
    return delta;
}

// Loops the value t, so that it is never larger than length and never smaller than 0.
pub fn repeat(t: f32, length: f32) -> f32 {
    return f32::clamp(t - f32::floor(t / length) * length, 0.0f32, length);
}

pub fn round_to_nearest_multiple(f: f32, multiple: f32) -> f32 {
    return f32::round(f / multiple) * multiple;
}
