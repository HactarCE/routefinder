use std::collections::BTreeSet;
use std::convert::TryInto;

use crate::{Match, Matches, Route, RouteSpec};

/// a router represents an ordered set of routes which can be applied
/// to a given request path, and any handler T that is associated with
/// each route

pub struct Router<T> {
    routes: BTreeSet<Route<T>>,
}

impl<T> std::fmt::Debug for Router<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_set().entries(self.routes.iter()).finish()
    }
}

impl<T> Default for Router<T> {
    fn default() -> Self {
        Self {
            routes: BTreeSet::new(),
        }
    }
}

impl<T> Router<T> {
    /// Builds a new router
    ///
    /// ```rust
    /// let mut router = routefinder::Router::new();
    /// router.add("/", ()).unwrap(); // here we use () as the handler
    /// assert!(router.best_match("/").is_some());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a route to the router, accepting any type that implements TryInto<[`RouteSpec`]>. In most circumstances, this will be a &str or a String.
    ///
    /// ```rust
    /// let mut router = routefinder::Router::new();
    /// assert!(router.add("*named_wildcard", ()).is_err());
    /// assert!(router.add("*", ()).is_ok());
    /// assert!(router.add(format!("/dynamic/{}", "route"), ()).is_ok());
    /// ```
    pub fn add<R>(&mut self, route: R, handler: T) -> Result<(), <R as TryInto<RouteSpec>>::Error>
    where
        R: TryInto<RouteSpec>,
    {
        self.routes.insert(Route::new(route, handler)?);
        Ok(())
    }

    /// Returns _all_ of the matching routes for a given path. This is
    /// probably not what you want, as [`Router::best_match`] is more
    /// efficient. The primary reason you'd want to use `matches` is
    /// to implement different route precedence rules or for
    /// testing. See [`Matches`] for more details. Note that `Matches`
    /// can be empty.
    ///
    /// ```rust
    /// let mut router = routefinder::Router::new();
    /// router.add("*", ()).unwrap();
    /// router.add("/:param", ()).unwrap();
    /// router.add("/hello", ()).unwrap();
    /// assert!(!router.matches("/").is_empty());
    /// assert_eq!(router.matches("/hello").len(), 3);
    /// assert_eq!(router.matches("/hey").len(), 2);
    /// assert_eq!(router.matches("/hey/there").len(), 1);
    /// ```
    pub fn matches<'a, 'b>(&'a self, path: &'b str) -> Matches<'a, 'b, T> {
        Matches::for_routes_and_path(self.routes.iter(), path)
    }

    /// Returns the single best route match as defined by the sorting
    /// rules. To compare any two routes, step through each
    /// [`Segment`] and find the first pair that are not equal,
    /// according to: `Exact > Param > Wildcard > (dots and slashes)`
    /// As a result, `/hello` > `/:param` > `/*`.  Because we can sort
    /// the routes before encountering a path, we evaluate them from
    /// highest to lowest weight and an early return as soon as we
    /// find a match.
    pub fn best_match<'a, 'b>(&'a self, path: &'b str) -> Option<Match<'a, 'b, T>> {
        self.routes.iter().rev().find_map(|r| r.is_match(path))
    }
}
