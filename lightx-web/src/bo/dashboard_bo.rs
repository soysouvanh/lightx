use lightx::core::AppError;
use lightx::ext::bytes::Bytes;
use lightx::ext::http_body_util::Full;
use lightx::ext::hyper::Response;

use crate::Users;
use crate::generated::{
    TMPLX_STATIC_SIZE_RENDER_DASHBOARD, TMPLX_STATIC_SIZE_RENDER_EDIT_USER,
    TMPLX_STATIC_SIZE_RENDER_VIEW_USER,
};

pub struct DashboardListViewData {
    pub message: String,
    pub users: Vec<Users>,
}

pub struct DashboardViewUserData {
    pub user: Users,
}

pub struct DashboardEditUserData {
    pub user: Users,
}

pub struct DashboardBo;

#[allow(non_snake_case)]
#[allow(unused_variables)]
impl DashboardBo {
    // ----------------------------------------------------
    // 1. GET /dashboard
    // ----------------------------------------------------
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
    pub async fn DashboardAdd(
        _ctx: &mut crate::RequestContext,
    ) -> Result<Response<Full<Bytes>>, AppError> {
        Ok(Response::builder()
            .status(303)
            .header("Location", "/dashboard")
            .body(Full::new(Bytes::from("")))
            .unwrap())
    }

    // ----------------------------------------------------
    // 3. GET /dashboard/view
    // ----------------------------------------------------
    pub async fn DashboardView(
        ctx: &mut crate::RequestContext,
    ) -> Result<Response<Full<Bytes>>, AppError> {
        let pool = &ctx.sqlite_pool;

        let users = Users::list_by_cursor(pool, -1, 1000)
            .await
            .unwrap_or(vec![]);
        let user = users.into_iter().next().unwrap_or(Users {
            id: 0,
            pseudo: "Inconnu".into(),
            email: "inconnu@inconnu.com".into(),
            is_admin: false,
        });

        let view_data = DashboardViewUserData { user };
        let mut html = String::with_capacity(TMPLX_STATIC_SIZE_RENDER_VIEW_USER + 500);
        render_view_user!(&mut html, &view_data);

        Ok(Response::builder()
            .status(200)
            .header("Content-Type", "text/html; charset=utf-8")
            .body(Full::new(Bytes::from(html)))
            .unwrap())
    }

    // ----------------------------------------------------
    // 4. GET /dashboard/edit
    // ----------------------------------------------------
    pub async fn DashboardEditForm(
        _ctx: &mut crate::RequestContext,
    ) -> Result<Response<Full<Bytes>>, AppError> {
        let view_data = DashboardEditUserData {
            user: Users {
                id: 1,
                pseudo: "Mock".into(),
                email: "mock@mock.com".into(),
                is_admin: true,
            },
        };

        let mut html = String::with_capacity(TMPLX_STATIC_SIZE_RENDER_EDIT_USER + 500);
        render_edit_user!(&mut html, &view_data);

        Ok(Response::builder()
            .status(200)
            .header("Content-Type", "text/html; charset=utf-8")
            .body(Full::new(Bytes::from(html)))
            .unwrap())
    }

    // ----------------------------------------------------
    // 5. POST /dashboard/edit
    // ----------------------------------------------------
    pub async fn DashboardEdit(
        _ctx: &mut crate::RequestContext,
    ) -> Result<Response<Full<Bytes>>, AppError> {
        Ok(Response::builder()
            .status(303)
            .header("Location", "/dashboard")
            .body(Full::new(Bytes::from("")))
            .unwrap())
    }

    // ----------------------------------------------------
    // 6. POST /dashboard/delete
    // ----------------------------------------------------
    pub async fn DashboardDelete(
        _ctx: &mut crate::RequestContext,
    ) -> Result<Response<Full<Bytes>>, AppError> {
        Ok(Response::builder()
            .status(303)
            .header("Location", "/dashboard")
            .body(Full::new(Bytes::from("")))
            .unwrap())
    }
}
