use url::Url;

pub enum Route {
    CalendarStart,
    CalendarEnd,
    Events,
    Submit,
    CheckLastSubmission,
    Unhandled,
}

impl From<&Url> for Route {
    fn from(url: &Url) -> Self {
        if let Some(mut path_segments) = url.path_segments() {
            match path_segments.next() {
                // /calendar_start
                Some("calendar_start") => return Route::CalendarStart,
                // /calendar_end
                Some("calendar_end") => return Route::CalendarEnd,
                // /events
                Some("events") => return Route::Events,
                Some("submit") => match path_segments.next() {
                    Some("discord") => match path_segments.next() {
                        // /submit/discord/last
                        Some("last") => Route::CheckLastSubmission,
                        Some(_) => Route::Unhandled,
                        // /submit/discord
                        None => Route::Submit,
                    },
                    _ => return Route::Unhandled,
                },
                _ => return Route::Unhandled,
            };
        }
        Route::Unhandled
    }
}
