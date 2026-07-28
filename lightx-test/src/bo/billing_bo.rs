use lightx::core::AppError;

pub struct UserAge(pub u32);

pub struct CartTotal(pub f64);

impl CartTotal {
    pub fn apply_discount(&mut self, percentage: f64) {
        self.0 -= self.0 * (percentage / 100.0);
    }
}

pub struct BillingBo;

impl BillingBo {
    ///  Phase 1 : Contrôle des Règles de Gestion (Validation Métier Pures)
    ///
    /// Note Pédagogique : The user_age is now passed as an explicit typed parameter
    /// instead of being extracted from a TypeMap. This is zero-overhead and fully type-safe.
    pub async fn validate_discount_eligibility(user_age: &UserAge) -> Result<(), AppError> {
        // Règle de gestion stricte
        if user_age.0 <= 65 && user_age.0 >= 25 {
            lightx::bail_business_rule!(
                "age",
                "L'utilisateur n'est ni sénior ni jeune. Réduction non applicable."
            );
        }

        Ok(())
    }

    ///  Phase 2 : Traitement Principal (Mutation / Action)
    ///
    /// Note Pédagogique : All dependencies are now explicit typed parameters.
    /// No TypeMap, no Box<dyn Any>, no vtable dispatch. Pure zero-overhead.
    pub async fn execute_discount(
        user_age: &UserAge,
        cart_total: &mut CartTotal,
    ) -> Result<(), AppError> {
        // 1. On s'assure (Fail-Fast) que les règles de gestion sont respectées
        Self::validate_discount_eligibility(user_age).await?;

        // 2. Application de la logique de traitement (Zéro allocation)
        cart_total.apply_discount(10.0);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_apply_discount_for_senior() {
        // 1. Données typées explicites (pas de TypeMap)
        let user_age = UserAge(70);
        let mut cart_total = CartTotal(100.0);

        // 2. Appel du traitement principal avec paramètres explicites
        BillingBo::execute_discount(&user_age, &mut cart_total)
            .await
            .unwrap();

        // 3. Assertion pure (Vérification de la mutation)
        assert_eq!(cart_total.0, 90.0); // Le prix a bien été réduit de 10%
    }

    #[tokio::test]
    async fn test_no_discount_for_adult_fails() {
        let user_age = UserAge(40);
        let mut cart_total = CartTotal(100.0);

        // On s'attend à ce que l'exécution renvoie une erreur métier
        let result = BillingBo::execute_discount(&user_age, &mut cart_total).await;
        assert!(result.is_err());

        if let Err(AppError::BusinessError { msg, .. }) = result {
            assert_eq!(
                msg,
                "L'utilisateur n'est ni sénior ni jeune. Réduction non applicable."
            );
        } else {
            panic!("Expected a BusinessError!");
        }

        assert_eq!(cart_total.0, 100.0); // Pas de réduction pour les 40 ans
    }
}
