use crate::{RequestContext, UserRoles, Users};
use lightx::core::AppError;

pub struct UserBo;

impl UserBo {
    ///  Phase 1 : Contrôle de la Règle de Gestion
    ///
    /// Called by the AOP pipeline. Extracts the typed payload from raw_body
    /// using the generated check_parameters (zero-copy deserialization).
    pub async fn validate_admin_creation(
        _ctx: &RequestContext,
        payload: &crate::AdminCreationPayload,
    ) -> Result<(), AppError> {
        // Règle métier stricte : seuls les employés peuvent devenir admin
        if !payload.email.ends_with("@lightx.com") {
            lightx::bail_business_rule!(
                "email",
                "Seuls les domaines @lightx.com peuvent être administrateurs."
            );
        }

        // Règle métier virtuelle : l'AOP garantit un booléen, le BO vérifie sa sémantique
        if !payload.accept_terms {
            lightx::bail_business_rule!(
                "accept_terms",
                "Vous devez obligatoirement accepter les responsabilités système pour continuer."
            );
        }

        Ok(())
    }

    ///  Phase 2 : Traitement Principal (Multi-Insert Transactionnel)
    ///
    /// The payload is deserialized from ctx.raw_body via check_parameters.
    /// No TypeMap, no Box<dyn Any>. Pure zero-overhead typed extraction.
    pub async fn execute_admin_creation(
        ctx: &mut RequestContext,
        payload: &crate::AdminCreationPayload,
    ) -> Result<
        lightx::ext::hyper::Response<lightx::ext::http_body_util::Full<lightx::ext::bytes::Bytes>>,
        AppError,
    > {
        // 1. Instanciation du modèle utilisateur
        let new_user = Users {
            id: 0, // Ignoré par l'insert car auto_increment
            email: payload.email.clone(),
            first_name: Some(payload.first_name.clone()),
            last_name: payload.last_name.clone(),
            status: "active".to_string(),
            created_at: None, // Géré par CURRENT_TIMESTAMP en base
        };

        // 2. Première écriture (Déclenche le BEGIN TRANSACTION de manière transparente)
        let new_user_id = new_user.insert(ctx).await?;

        // 3. Instanciation de la table de liaison (UserRoles)
        let admin_role = UserRoles {
            user_id: new_user_id as i64,
            role_name: "admin".to_string(),
            assigned_at: None, // CURRENT_TIMESTAMP
        };

        // 4. Seconde écriture (Participe à la MÊME transaction via RequestContext)
        admin_role.insert(ctx).await?;

        let json = format!("{{\"user_id\":{}}}", new_user_id);

        // 5. Broadcast instantané "Pub/Sub" de la création
        let _ = ctx
            .global_state
            .send(lightx::ext::bytes::Bytes::from(json.clone()));

        let resp = lightx::ext::hyper::Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(lightx::ext::http_body_util::Full::new(
                lightx::ext::bytes::Bytes::from(json),
            ))
            .unwrap();

        Ok(resp)
    }
}
