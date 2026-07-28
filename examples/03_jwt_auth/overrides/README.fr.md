#  LightX Overrides Strategy

Bienvenue dans le dossier d'overrides !

La philosophie "Database-First" de LightX écrase entièrement le dossier `schema/` à chaque compilation.
**Ne modifiez jamais les fichiers dans `schema/` car ils seront écrasés !**

Si vous souhaitez surcharger des règles de validation métier (ex: forcer `min_length = 5` sur une colonne SQL), vous devez reproduire l'arborescence de la table ici.

## Exemple
Pour surcharger la colonne `last_name` de la table `users` :
1. Créez le dossier `users/` ici.
2. Créez le fichier `users/last_name.toml` ici avec UNIQUEMENT les valeurs à écraser.

```toml
[min_length]
value = 5
message = "Le nom de famille doit faire au moins 5 caractères (Surcharge manuelle !)"
```
