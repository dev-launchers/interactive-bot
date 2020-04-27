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
            match path_segments.next() {
                // /calendar_start
                Some("calendar_start") => return Route::CalendarStart,
                // /calendar_end
                Some("calendar_end") => return Route::CalendarEnd,
                // /events
                Some("events") => return Route::Events,
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
                        Some(_) => Route::Unhandled,
                        None => {
                            if let Some(submitter) = path_segments.next() {
                                // /submit/discord/:submitter
                                Route::Submit {
                                    submitter: submitter.to_string(),
                                }
                            } else {
                                Route::Unhandled
                            }
                        }
                    },
                    _ => return Route::Unhandled,
                },
                _ => return Route::Unhandled,
            };
        }
        Route::Unhandled
    }
}
