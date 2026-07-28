#  Architecture Source de Vérité Unique (LightX)

Bienvenue dans le répertoire du modèle de données pur !

Ce dossier est **généré automatiquement** à la compilation par le framework LightX via une introspection avancée de votre base de données. Il agit comme le dictionnaire de données absolu (Single Source of Truth) de tout le cycle de vie de l'application.

##  AVERTISSEMENT CRITIQUE
**NE MODIFIEZ MANUELLEMENT AUCUN FICHIER DANS CE RÉPERTOIRE !**

Tous les fichiers de définition de structure (`.toml`) ici sont détruits puis regénérés de zéro à chaque appel de `cargo build`. Toute modification humaine sera définitivement et formellement perdue.

Si vous avez besoin de forcer des règles de validation sur mesure ou de corriger un type dégénéré, vous devez utiliser impérativement le dossier `../overrides/` à la place.