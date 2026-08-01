use lightx::core::AppError;
use lightx::ext::bytes::Bytes;
use lightx::ext::http_body_util::Full;
use lightx::ext::hyper::Response;

pub async fn home_index(
    _ctx: &mut crate::RequestContext,
) -> Result<Response<Full<Bytes>>, AppError> {
    // Demo Showcase HTML Interface
    let html = r#"<!DOCTYPE html>
<html lang="fr">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>LightX Showcase</title>
    <style>
        body { font-family: 'Inter', sans-serif; background: #0f172a; color: white; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0; }
        .container { text-align: center; background: #1e293b; padding: 50px; border-radius: 12px; box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.4); max-width: 600px; }
        h1 { font-size: 3rem; margin-bottom: 0.5rem; background: linear-gradient(to right, #38bdf8, #818cf8); -webkit-background-clip: text; color: transparent; }
        p { font-size: 1.25rem; color: #94a3b8; }
        .links { margin-top: 30px; display: flex; gap: 15px; justify-content: center; }
        .links a { padding: 10px 20px; background: #3b82f6; color: white; text-decoration: none; border-radius: 6px; font-weight: bold; transition: all 0.2s; }
        .links a:hover { background: #2563eb; transform: scale(1.05); }
    </style>
</head>
<body>
    <div class="container">
        <h1>🚀 LightX Framework</h1>
        <p>Le serveur vitrine <strong>State of the Art</strong> ultra-performant. Routage O(1), AOP et Génération Static.</p>
        <div class="links">
            <a href="/docs">Lire la DOC</a>
            <a href="/api/health">Health API</a>
        </div>
    </div>
</body>
</html>"#;
    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(Full::new(Bytes::from(html)))
        .unwrap())
}
