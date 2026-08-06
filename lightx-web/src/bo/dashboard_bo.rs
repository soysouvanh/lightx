#![allow(clippy::collapsible_if)]
//! # Dashboard Business Object
//!
//! This module provides a complete End-to-End demonstrative CRUD (Create, Read, Update, Delete)
//! implementation for `Users` using LightX framework's powerful static routing and `Daox` ORM.
//!
//! ## Pedagogical Excellence
//! - Demonstrates safe extraction of parameters without heavy third-party dependencies.
//! - Uses `tmplx` for extremely fast static-size HTML template generation.
//! - Adheres to standard HTTP `Location` redirects (303) when resources are updated/deleted.

use lightx::core::AppError;
use lightx::ext::bytes::Bytes;
use lightx::ext::http_body_util::Full;
use lightx::ext::hyper::Response;
use std::collections::HashMap;

use crate::generated::{
    TMPLX_STATIC_SIZE_RENDER_DASHBOARD, TMPLX_STATIC_SIZE_RENDER_EDIT_USER,
    TMPLX_STATIC_SIZE_RENDER_VIEW_USER,
};
use crate::{Users, UsersPatch};

/// Data context explicitly strictly typed and passed to the `dashboard.html` template.
pub struct DashboardListViewData {
    pub message: String,
    pub users: Vec<Users>,
}

/// Data context passed to the `view_user.html` template.
pub struct DashboardViewUserData {
    pub user: Users,
}

/// Data context passed to the `edit_user.html` template.
pub struct DashboardEditUserData {
    pub user: Users,
}

/// The main Dashboard Controller grouping logic as static methods.
pub struct DashboardBo;

fn parse_urlencoded(query: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for pair in query.split('&') {
        let mut parts = pair.splitn(2, '=');
        let key = parts.next().unwrap_or("");
        let value = parts.next().unwrap_or("");

        let key = urlencoding_decode(key);
        let value = urlencoding_decode(value);
        if !key.is_empty() {
            map.insert(key, value);
        }
    }
    map
}

fn urlencoding_decode(input: &str) -> String {
    let input = input.replace('+', " ");
    let mut out = String::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '%' && i + 2 < chars.len() {
            let hex = format!("{}{}", chars[i + 1], chars[i + 2]);
            if let Ok(b) = u8::from_str_radix(&hex, 16) {
                out.push(b as char);
            } else {
                out.push(chars[i]);
                out.push(chars[i + 1]);
                out.push(chars[i + 2]);
            }
            i += 3;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

#[allow(non_snake_case)]
#[allow(unused_variables)]
impl DashboardBo {
    // ----------------------------------------------------
    // 1. GET /dashboard
    // ----------------------------------------------------
    /// Lists all users from the Database.
    /// Uses cursor-based pagination provided automatically by `daox` to guarantee memory safety.
    pub async fn DashboardList(
        ctx: &mut crate::RequestContext,
    ) -> Result<Response<Full<Bytes>>, AppError> {
        let pool = &ctx.sqlite_pool;
        let users =
            Users::list_by_cursor(pool, -1, 1000)
                .await
                .map_err(|e| AppError::DatabaseError {
                    msg: e.to_string(),
                    file: file!(),
                    line: line!(),
                })?;

        let view_data = DashboardListViewData {
            message: String::new(),
            users,
        };

        let mut html = String::with_capacity(TMPLX_STATIC_SIZE_RENDER_DASHBOARD + 2000);
        render_dashboard!(&mut html, &view_data);

        Ok(Response::builder()
            .status(200)
            .header("Content-Type", "text/html; charset=utf-8")
            .body(Full::new(Bytes::from(html)))
            .unwrap())
    }

    // ----------------------------------------------------
    // 2. POST /dashboard/add
    // ----------------------------------------------------
    /// Captures URL-encoded `POST` payload data and securely inserts a new User natively.
    /// Redirects securely to `/dashboard` upon HTTP completion.
    pub async fn DashboardAdd(
        ctx: &mut crate::RequestContext,
    ) -> Result<Response<Full<Bytes>>, AppError> {
        let pool = &ctx.sqlite_pool;
        let body_str = String::from_utf8_lossy(&ctx.raw_body).to_string();
        let params = parse_urlencoded(&body_str);

        let pseudo = params.get("pseudo").cloned().unwrap_or_default();
        let email = params.get("email").cloned().unwrap_or_default();
        let is_admin = params.get("is_admin").map(|v| v == "true").unwrap_or(false);

        let new_user = Users {
            id: 0,
            pseudo,
            email,
            is_admin,
        };

        let _ = new_user.insert(pool).await;

        Ok(Response::builder()
            .status(303)
            .header("Location", "/dashboard")
            .body(Full::new(Bytes::from("")))
            .unwrap())
    }

    // ----------------------------------------------------
    // 3. GET /dashboard/view
    // ----------------------------------------------------
    /// Renders the detailed view of a single user.
    /// Manually extracts the `id` from the URL query parameters since this path doesn't use `/:id` path matching.
    pub async fn DashboardView(
        ctx: &mut crate::RequestContext,
    ) -> Result<Response<Full<Bytes>>, AppError> {
        let pool = &ctx.sqlite_pool;

        let mut id = 0;
        if let Some(req) = &ctx.raw_req {
            if let Some(query) = req.uri().query() {
                let params = parse_urlencoded(query);
                if let Some(id_str) = params.get("id") {
                    id = id_str.parse().unwrap_or(0);
                }
            }
        }

        let user_opt = Users::get_by_id(pool, id).await.unwrap_or(None);

        if let Some(user) = user_opt {
            let view_data = DashboardViewUserData { user };
            let mut html = String::with_capacity(TMPLX_STATIC_SIZE_RENDER_VIEW_USER + 500);
            render_view_user!(&mut html, &view_data);

            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "text/html; charset=utf-8")
                .body(Full::new(Bytes::from(html)))
                .unwrap())
        } else {
            Ok(Response::builder()
                .status(303)
                .header("Location", "/dashboard")
                .body(Full::new(Bytes::from("User not found")))
                .unwrap())
        }
    }

    // ----------------------------------------------------
    // 4. GET /dashboard/edit
    // ----------------------------------------------------
    /// Renders the edit form for a user, perfectly hydrated with the database data.
    /// If the `id` is missing or invalid, gracefully degrades to a 303 Redirect to `/dashboard`.
    pub async fn DashboardEditForm(
        ctx: &mut crate::RequestContext,
    ) -> Result<Response<Full<Bytes>>, AppError> {
        let pool = &ctx.sqlite_pool;

        let mut id = 0;
        if let Some(req) = &ctx.raw_req {
            if let Some(query) = req.uri().query() {
                let params = parse_urlencoded(query);
                if let Some(id_str) = params.get("id") {
                    id = id_str.parse().unwrap_or(0);
                }
            }
        }

        let user_opt = Users::get_by_id(pool, id).await.unwrap_or(None);

        if let Some(user) = user_opt {
            let view_data = DashboardEditUserData { user };

            let mut html = String::with_capacity(TMPLX_STATIC_SIZE_RENDER_EDIT_USER + 500);
            render_edit_user!(&mut html, &view_data);

            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "text/html; charset=utf-8")
                .body(Full::new(Bytes::from(html)))
                .unwrap())
        } else {
            Ok(Response::builder()
                .status(303)
                .header("Location", "/dashboard")
                .body(Full::new(Bytes::from("User not found")))
                .unwrap())
        }
    }

    // ----------------------------------------------------
    // 5. POST /dashboard/edit
    // ----------------------------------------------------
    /// Processes the URL-encoded POST payload when editing a user.
    /// Utilizes `UsersPatch` from Daox for highly optimized partial SQL updates.
    pub async fn DashboardEdit(
        ctx: &mut crate::RequestContext,
    ) -> Result<Response<Full<Bytes>>, AppError> {
        let pool = &ctx.sqlite_pool;
        let body_str = String::from_utf8_lossy(&ctx.raw_body).to_string();
        let params = parse_urlencoded(&body_str);

        if let Some(id_str) = params.get("id") {
            if let Ok(id) = id_str.parse::<i32>() {
                let pseudo = params.get("pseudo").cloned();
                let email = params.get("email").cloned();
                let is_admin = Some(params.get("is_admin").map(|v| v == "true").unwrap_or(false));

                let patch = UsersPatch {
                    pseudo,
                    email,
                    is_admin,
                };
                let _ = Users::update_partial_by_id(pool, id, &patch).await;
            }
        }

        Ok(Response::builder()
            .status(303)
            .header("Location", "/dashboard")
            .body(Full::new(Bytes::from("")))
            .unwrap())
    }

    // ----------------------------------------------------
    // 6. POST /dashboard/delete
    // ----------------------------------------------------
    /// Deletes a specific user using native fast SQL deletion, avoiding ORM allocations.
    pub async fn DashboardDelete(
        ctx: &mut crate::RequestContext,
    ) -> Result<Response<Full<Bytes>>, AppError> {
        let pool = &ctx.sqlite_pool;
        let body_str = String::from_utf8_lossy(&ctx.raw_body).to_string();
        let params = parse_urlencoded(&body_str);

        if let Some(id_str) = params.get("id") {
            if let Ok(id) = id_str.parse::<i32>() {
                let _ = Users::delete_by_id(pool, id).await;
            }
        }

        Ok(Response::builder()
            .status(303)
            .header("Location", "/dashboard")
            .body(Full::new(Bytes::from("")))
            .unwrap())
    }
}
