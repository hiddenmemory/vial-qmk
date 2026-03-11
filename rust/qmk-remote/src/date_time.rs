/// solar_times.rs
///
/// This code was authored by Claude.code:
///
/// I wanted a quick way to get sunrise, sunset, dusk, dawn. Sunrise and sunset are straight
/// forward to get using `spa`, but dusk and dawn are more tricky. If this works, it will likely
/// get rewritten, so at the moment, consider it placeholder.
///
/// Calculates dawn, sunrise, sunset, and dusk as seconds since midnight
/// for a given latitude, longitude, and timezone.
///
use chrono::{DateTime, Duration, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Timelike};
use chrono_tz::Tz;
use spa::{StdFloatOps, SunriseAndSet, solar_position, sunrise_and_set};
use std::str::FromStr;

// Civil twilight: sun centre is 6° below horizon → zenith angle = 96°.
const CIVIL_TWILIGHT_ZENITH: f64 = 96.0;

// Binary-search tolerance: converge to within 10 seconds.
const TOLERANCE_SECS: i64 = 10;

/// Returns the solar zenith angle (degrees) at a given UTC instant and location.
fn zenith_at(utc: DateTime<chrono::Utc>, lat: f64, lon: f64) -> Result<f64, anyhow::Error> {
    solar_position::<StdFloatOps>(utc, lat, lon)
        .map(|pos| pos.zenith_angle)
        .map_err(|e| anyhow::anyhow!("spa solar_position error: {e:?}"))
}

/// Binary-search for the UTC instant between `lo` and `hi` at which the sun's
/// zenith angle crosses `threshold`.
///
/// The zenith must be on opposite sides of `threshold` at `lo` and `hi`.
fn bisect_zenith_crossing(
    lo: DateTime<chrono::Utc>,
    hi: DateTime<chrono::Utc>,
    threshold: f64,
    lat: f64,
    lon: f64,
) -> Result<DateTime<chrono::Utc>, anyhow::Error> {
    let mut lo = lo;
    let mut hi = hi;

    while (hi - lo).num_seconds().abs() > TOLERANCE_SECS {
        let mid = lo + Duration::seconds((hi - lo).num_seconds() / 2);
        let z_lo = zenith_at(lo, lat, lon)?;
        let z_mid = zenith_at(mid, lat, lon)?;

        // Stay in the half where the sign relative to threshold changes.
        if (z_lo - threshold).signum() == (z_mid - threshold).signum() {
            lo = mid;
        } else {
            hi = mid;
        }
    }

    Ok(lo + Duration::seconds((hi - lo).num_seconds() / 2))
}

/// Converts a UTC `DateTime` to seconds since local midnight in `tz`.
fn to_secs_since_midnight(utc_dt: DateTime<chrono::Utc>, tz: Tz) -> u32 {
    utc_dt.with_timezone(&tz).num_seconds_from_midnight()
}

/// Compute solar event times for `date` at `(latitude, longitude)` expressed
/// as seconds since local midnight in `timezone`.
///
/// # Arguments
/// * `latitude`  – decimal degrees, positive = North
/// * `longitude` – decimal degrees, positive = East
/// * `timezone`  – IANA timezone string, e.g. `"Europe/London"` or `"America/New_York"`
/// * `date`      – optional `NaiveDate`; defaults to today in the local system timezone
///
/// # Errors
/// Returns a `String` error if the timezone is invalid, the sun never rises/sets
/// on the given date (polar day/night), or the `spa` crate returns an error.
pub fn solar_times(
    latitude: f64,
    longitude: f64,
    timezone: &str,
    date: Option<NaiveDate>,
) -> Result<hid_bridge::DateTime, anyhow::Error> {
    // --- Parse timezone ---
    let tz: Tz =
        Tz::from_str(timezone).map_err(|_| anyhow::anyhow!("Unknown timezone: '{timezone}'"))?;

    // --- Resolve date (default = today in the local system clock) ---
    let naive_date = date.unwrap_or_else(|| Local::now().date_naive());

    // Build UTC noon — avoids DST edge-cases at midnight and gives spa a
    // reference point near solar noon.
    let noon_local = tz
        .from_local_datetime(&NaiveDateTime::new(
            naive_date,
            NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
        ))
        .single()
        .ok_or_else(|| {
            anyhow::anyhow!("Ambiguous or invalid local time for {naive_date} in {timezone}")
        })?;
    let noon_utc: DateTime<chrono::Utc> = noon_local.with_timezone(&chrono::Utc);

    // --- Sunrise / Sunset via spa ---
    let (sunrise_utc, sunset_utc) =
        match sunrise_and_set::<StdFloatOps>(noon_utc, latitude, longitude)
            .map_err(|e| anyhow::anyhow!("spa error (sunrise/sunset): {e:?}"))?
        {
            SunriseAndSet::Daylight(rise, set) => (rise, set),
            SunriseAndSet::PolarDay => {
                return Err(anyhow::anyhow!(
                    "Polar day on {naive_date} at ({latitude}, {longitude}): sun never sets"
                ));
            }
            SunriseAndSet::PolarNight => {
                return Err(anyhow::anyhow!(
                    "Polar night on {naive_date} at ({latitude}, {longitude}): sun never rises"
                ));
            }
        };

    // --- Civil Dawn ---
    // Search between local midnight and sunrise.
    // At midnight the zenith is > 96° (sun deep below horizon).
    // At sunrise the zenith ≈ 90° (sun at horizon).
    // Dawn is where the zenith descends through 96°.
    let midnight_utc = noon_utc - Duration::hours(12);
    let dawn_utc = bisect_zenith_crossing(
        midnight_utc,
        sunrise_utc,
        CIVIL_TWILIGHT_ZENITH,
        latitude,
        longitude,
    )?;

    // --- Civil Dusk ---
    // Search between sunset and the following midnight.
    // After sunset the zenith climbs back through 96° — that crossing is dusk.
    let next_midnight_utc = noon_utc + Duration::hours(12);
    let dusk_utc = bisect_zenith_crossing(
        sunset_utc,
        next_midnight_utc,
        CIVIL_TWILIGHT_ZENITH,
        latitude,
        longitude,
    )?;

    // --- Current time as seconds since local midnight ---
    let current_secs = chrono::Utc::now()
        .with_timezone(&tz)
        .num_seconds_from_midnight();

    Ok(hid_bridge::DateTime {
        seconds_since_midnight: current_secs,
        dawn_secs: to_secs_since_midnight(dawn_utc, tz),
        sunrise_secs: to_secs_since_midnight(sunrise_utc, tz),
        sunset_secs: to_secs_since_midnight(sunset_utc, tz),
        dusk_secs: to_secs_since_midnight(dusk_utc, tz),
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_london_summer_order() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 21).unwrap();
        let t = solar_times(51.5074, -0.1278, "Europe/London", Some(date))
            .expect("should succeed for London in summer");

        assert!(t.dawn_secs < t.sunrise_secs, "dawn must be before sunrise");
        assert!(
            t.sunrise_secs < t.sunset_secs,
            "sunrise must be before sunset"
        );
        assert!(t.sunset_secs < t.dusk_secs, "sunset must be before dusk");
        assert!(t.dusk_secs < 86_400, "dusk must be within one day");
    }

    #[test]
    fn test_new_york_winter() {
        let date = NaiveDate::from_ymd_opt(2024, 12, 21).unwrap();
        let t = solar_times(40.7128, -74.0060, "America/New_York", Some(date))
            .expect("should succeed for New York in winter");

        assert!(t.dawn_secs < t.sunrise_secs);
        assert!(t.sunrise_secs < t.sunset_secs);
        assert!(t.sunset_secs < t.dusk_secs);
    }

    #[test]
    fn test_current_secs_in_range() {
        let t = solar_times(51.5074, -0.1278, "Europe/London", None)
            .expect("should succeed for London today");
        assert!(
            t.seconds_since_midnight < 86_400,
            "current_secs must be < 86400"
        );
    }

    #[test]
    fn test_invalid_timezone() {
        let result = solar_times(51.5, -0.1, "Not/ATimezone", None);
        assert!(result.is_err());
    }
}
