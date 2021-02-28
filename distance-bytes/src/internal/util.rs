use crate::internal::{Quaternion, Vector3, EMPTY_MARK};

// Based on Unity's `Mathf.Approximately()` function
pub(crate) trait ApproximatelyEquals {
    fn approximately_equals(&self, other: &Self) -> bool;
}

impl ApproximatelyEquals for f32 {
    fn approximately_equals(&self, other: &Self) -> bool {
        f32::abs(other - self) < (1E-6 * self.abs().max(other.abs())).max(f32::EPSILON * 8.0)
    }
}

impl ApproximatelyEquals for Vector3 {
    fn approximately_equals(&self, other: &Self) -> bool {
        (self.x).approximately_equals(&other.x)
            && (self.y).approximately_equals(&other.y)
            && (self.z).approximately_equals(&other.z)
    }
}

impl ApproximatelyEquals for Quaternion {
    fn approximately_equals(&self, other: &Self) -> bool {
        (self.v.x).approximately_equals(&other.v.x)
            && (self.v.y).approximately_equals(&other.v.y)
            && (self.v.z).approximately_equals(&other.v.z)
            && (self.s).approximately_equals(&other.s)
    }
}

pub(crate) fn scope_mark_string(scope_mark: i32) -> &'static str {
    match scope_mark {
        11111111 => "Array",
        12121212 => "Dictionary",
        22222222 => "SerialComponent",
        23232323 => "UnknownComponent",
        33333333 => "BuiltInComponent",
        44444444 => "General",
        55555555 => "Children",
        66666666 => "GameObject",
        88888888 => "LevelSettings",
        99999999 => "Level",
        n if n == EMPTY_MARK => "Empty",
        _ => "INVALID",
    }
}
