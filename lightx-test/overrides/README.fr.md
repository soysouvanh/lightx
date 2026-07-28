#  Stratégie des Overrides LightX

Bienvenue dans le répertoire des surcharges (overrides) !

La philosophie "Database-First" de LightX écrase entièrement le dossier `schema/` à chaque cycle de compilation. 
**Ne modifiez jamais manuellement les fichiers dans `schema/` car toutes vos modifications seront détruites !**

Si vous souhaitez surcharger des règles de validation métier (ex: forcer `min_length = 5` sur une colonne SQL) ou lier de nouvelles métadonnées, vous devez reproduire l'arborescence exacte de la table base de données ici.

##  Exemple
Pour créer une redéfinition sur la colonne `last_name` de la table `users` :
1. Créez un répertoire `users/` ici.
2. Créez ensuite le fichier `users/last_name.toml` ici avec UNIQUEMENT les valeurs à écraser.

```toml
[min_length]
value = 5
message = "Le nom de famille doit faire au moins 5 caractères (Surcharge manuelle !)"
```