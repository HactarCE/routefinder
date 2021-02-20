use std::collections::BTreeSet;
use std::ops::Deref;

use crate::{Captures, Route, Segment};

#[derive(Debug)]
pub struct Matches<'router, 'path, T> {
    matches: BTreeSet<Match<'router, 'path, T>>,
}

impl<'router, 'path, T> Deref for Matches<'router, 'path, T> {
    type Target = BTreeSet<Match<'router, 'path, T>>;

    fn deref(&self) -> &Self::Target {
        &self.matches
    }
}

impl<'router, 'path, T> Matches<'router, 'path, T> {
    pub fn len(&self) -> usize {
        self.matches.len()
    }

    pub fn best(&self) -> Option<&Match<'router, 'path, T>> {
        self.matches.iter().last()
    }

    pub fn for_routes_and_path(routes: &'router [Route<T>], path: &'path str) -> Self {
        Self {
            matches: routes
                .iter()
                .filter_map(|route| route.is_match(path))
                .collect(),
        }
    }
}

#[derive(Debug)]
pub struct Match<'router, 'path, T> {
    path: &'path str,
    route: &'router Route<T>,
    captures: Vec<&'path str>,
}

impl<'router, 'path, T> Match<'router, 'path, T> {
    pub(crate) fn new(
        path: &'path str,
        route: &'router Route<T>,
        captures: Vec<&'path str>,
    ) -> Self {
        Self {
            path,
            route,
            captures,
        }
    }

    pub fn handler(&self) -> &'router T {
        self.route.handler()
    }

    pub fn into_route(self) -> &'router Route<T> {
        self.route
    }

    pub fn captures(&self) -> Captures {
        self.route
            .segments()
            .iter()
            .filter(|s| matches!(s, Segment::Param(_) | Segment::Wildcard))
            .zip(&self.captures)
            .fold(
                Captures::default(),
                |mut captures, (segment, capture)| match segment {
                    Segment::Param(name) => {
                        captures
                            .0
                            .push((String::from(*name), String::from(*capture)));
                        captures
                    }

                    Segment::Wildcard => {
                        captures.1 = Some(String::from(*capture));
                        captures
                    }
                    _ => captures,
                },
            )
    }
}

impl<'router, 'path, T> PartialEq for Match<'router, 'path, T> {
    fn eq(&self, other: &Self) -> bool {
        *other.route == *self.route
    }
}

impl<'router, 'path, T> Eq for Match<'router, 'path, T> {}

impl<'router, 'path, T> PartialOrd for Match<'router, 'path, T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'router, 'path, T> Ord for Match<'router, 'path, T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.route
            .segments()
            .iter()
            .zip(other.route.segments())
            .map(|(mine, theirs)| mine.cmp(theirs))
            .find(|c| *c != std::cmp::Ordering::Equal)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}