use url::Url;

pub enum Route {
    CalendarStart,
    CalendarEnd,
    Events,
    Submit { submitter: String },
    CheckLastSubmission { submitter: String },
    Unhandled,
}

impl From<&Url> for Route {
    fn from(url: &Url) -> Route {
        if let Some(mut path_segments) = url.path_segments() {
            return match path_segments.next() {
                // /calendar_start
                Some("calendar_start") => Route::CalendarStart,
                // /calendar_end
                Some("calendar_end") => Route::CalendarEnd,
                // /events
                Some("events") => Route::Events,
                Some("submit") => match path_segments.next() {
                    Some("discord") => match path_segments.next() {
                        Some("last") => {
                            if let Some(submitter) = path_segments.next() {
                                // /submit/discord/last/:submitter
                                Route::CheckLastSubmission {
                                    submitter: submitter.to_string(),
                                }
                            } else {
                                Route::Unhandled
                            }
                        }
                        // /submit/discord/:submitter
                        Some(s) => Route::Submit {
                            submitter: s.to_string(),
                        },
                        None => Route::Unhandled,
                    },
                    _ => Route::Unhandled,
                },
                _ => Route::Unhandled,
            };
        }
        Route::Unhandled
    }
}
