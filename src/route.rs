pub enum Route {
    CalendarStart,
    CalendarEnd,
    Events,
    Unhandled,
}

impl From<&str> for Route {
    fn from(s: &str) -> Self {
        match s {
            "/calendar_start" => Route::CalendarStart,
            "/calendar_end" => Route::CalendarEnd,
            "/events" => Route::Events,
            _ => Route::Unhandled,
        }
    }
}
